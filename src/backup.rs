use std::{fs, path::Path};
use chrono::{Duration, Utc};
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::{BackupLog, Facility, PipeLine, Shop, Staff},
    state::AppState,
};

pub struct BackupManager;

impl BackupManager {
    // 1. 백업 실행 (풀 백업 or 증분 백업 자동 판단 또는 수동 지정)
    pub async fn execute_backup(
        state: &AppState,
        force_type: Option<String>,
    ) -> Result<BackupLog, String> {
        let now = Utc::now();
        let backup_dir = Path::new("backups");
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;
        }

        // 백업 유형 결정: 수동 지정이 없으면 풀 백업 여부에 따라 결정
        let last_full = *state.last_full_backup.read();
        let is_full = match force_type.as_deref() {
            Some("FULL") => true,
            Some("INCREMENTAL") => false,
            _ => {
                // 풀 백업이 없거나 30일이 지난 경우 풀 백업, 그 외에는 증분 백업
                match last_full {
                    None => true,
                    Some(lf) => now - lf > Duration::days(30),
                }
            }
        };

        let backup_type = if is_full { "FULL" } else { "INCREMENTAL" };
        let timestamp_str = now.format("%Y%m%d_%H%M%S").to_string();
        let file_name = format!("avenuel_fms_backup_{}_{}.json", backup_type, timestamp_str);
        let file_path = backup_dir.join(&file_name);

        let (payload, records_count, since_time) = if is_full {
            let shops = state.shops.read().clone();
            let pipes = state.pipes.read().clone();
            let facilities = state.facilities.read().clone();
            let staff = state.staff.read().clone();
            let total = shops.len() + pipes.len() + facilities.len() + staff.len();

            let data = json!({
                "backup_type": "FULL",
                "timestamp": now.to_rfc3339(),
                "building": "롯데백화점 잠실점 에비뉴엘",
                "version": "v2.1",
                "summary": {
                    "shops_count": shops.len(),
                    "pipes_count": pipes.len(),
                    "facilities_count": facilities.len(),
                    "staff_count": staff.len(),
                    "total_records": total
                },
                "data": {
                    "shops": shops,
                    "pipes": pipes,
                    "facilities": facilities,
                    "staff": staff
                }
            });
            (data, total, None)
        } else {
            // 증분 백업: 마지막 백업 시점 이후 변경된 레코드만 수집
            let last_sync = state.last_incremental_backup.read()
                .or(last_full)
                .unwrap_or(now - Duration::days(7));

            let shops: Vec<Shop> = state.shops.read().iter()
                .filter(|s| s.updated_at > last_sync)
                .cloned().collect();
            let pipes: Vec<PipeLine> = state.pipes.read().iter()
                .filter(|p| p.updated_at > last_sync)
                .cloned().collect();
            let facilities: Vec<Facility> = state.facilities.read().iter()
                .filter(|f| f.updated_at > last_sync)
                .cloned().collect();
            let staff: Vec<Staff> = state.staff.read().iter()
                .filter(|s| s.updated_at > last_sync)
                .cloned().collect();
            let total = shops.len() + pipes.len() + facilities.len() + staff.len();

            let data = json!({
                "backup_type": "INCREMENTAL",
                "timestamp": now.to_rfc3339(),
                "since_timestamp": last_sync.to_rfc3339(),
                "building": "롯데백화점 잠실점 에비뉴엘",
                "summary": {
                    "changed_records_count": total
                },
                "changed_data": {
                    "shops": shops,
                    "pipes": pipes,
                    "facilities": facilities,
                    "staff": staff
                }
            });
            (data, total, Some(last_sync))
        };

        // 로컬 파일 저장
        let content = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
        fs::write(&file_path, &content).map_err(|e| e.to_string())?;
        info!("💾 로컬 백업 파일 생성 완료: {:?}", file_path);

        // 구글 드라이브 / 시트 동기화 시도 (설정되어 있을 경우)
        let gdrive_file_id = Self::sync_to_google_drive(&file_name, &content).await;

        // 백업 상태 갱신
        if is_full {
            *state.last_full_backup.write() = Some(now);
        }
        *state.last_incremental_backup.write() = Some(now);

        let log = BackupLog {
            id: Uuid::new_v4().to_string(),
            backup_type: backup_type.to_string(),
            target: "GOOGLE_DRIVE".into(),
            file_name,
            file_id: gdrive_file_id,
            records_count,
            since_timestamp: since_time,
            created_at: now,
            status: "SUCCESS".into(),
            details: Some(json!({
                "file_size_bytes": content.len(),
                "local_path": file_path.to_string_lossy()
            })),
        };

        state.backup_logs.write().push(log.clone());
        info!("✅ [백업 완료] 유형: {}, 레코드 수: {}", backup_type, records_count);

        Ok(log)
    }

    // 2. 구글 드라이브 API 연동 업로드 (인증 토큰 또는 서비스 계정 지원)
    async fn sync_to_google_drive(file_name: &str, content: &str) -> Option<String> {
        let gdrive_token = std::env::var("GOOGLE_DRIVE_ACCESS_TOKEN").ok();
        let folder_id = std::env::var("GOOGLE_DRIVE_FOLDER_ID").ok();

        if let Some(token) = gdrive_token {
            info!("☁️ Google Drive로 백업 업로드 진행 중: {}", file_name);
            let client = reqwest::Client::new();
            
            // multipart 업로드 또는 간이 드라이브 업로드 엔드포인트
            let mut metadata = json!({ "name": file_name, "mimeType": "application/json" });
            if let Some(fid) = folder_id {
                metadata["parents"] = json!([fid]);
            }

            let res = client.post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
                .bearer_auth(token)
                .json(&json!({
                    "metadata": metadata,
                    "content": content
                }))
                .send()
                .await;

            match res {
                Ok(r) if r.status().is_success() => {
                    let json_res: serde_json::Value = r.json().await.unwrap_or_default();
                    let file_id = json_res["id"].as_str().map(|s| s.to_string());
                    info!("☁️ Google Drive 업로드 성공: ID {:?}", file_id);
                    file_id
                }
                Ok(r) => {
                    warn!("Google Drive 응답 코드: {}", r.status());
                    None
                }
                Err(e) => {
                    error!("Google Drive 업로드 실패: {}", e);
                    None
                }
            }
        } else {
            info!("ℹ️ Google Drive 인증 정보가 설정되지 않아 로컬 아카이브(backups/)에 안전하게 보관되었습니다.");
            None
        }
    }

    // 3. 주간 자동 백업 백그라운드 스케줄러 (7일마다 자동 트리거)
    pub fn start_scheduler(state: AppState) {
        tokio::spawn(async move {
            info!("⏰ FMS 주간 자동 백업 스케줄러 시작 (주기: 7일)");
            
            // 초기 1회 실행 확인 (풀 백업이 없으면 즉시 풀 백업 1회 실행)
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            if state.last_full_backup.read().is_none() {
                info!("🚀 초기 풀 백업(Full Backup) 자동 실행...");
                if let Err(e) = Self::execute_backup(&state, Some("FULL".into())).await {
                    error!("초기 백업 오류: {}", e);
                }
            }

            // 7일(604800초) 간격으로 증분 백업 실행 (30일 초과 시 자동으로 풀 백업)
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(7 * 24 * 3600));
            loop {
                interval.tick().await;
                info!("⏰ 주간 정기 백업 스케줄러 트리거됨...");
                if let Err(e) = Self::execute_backup(&state, None).await {
                    error!("주간 백업 실행 실패: {}", e);
                }
            }
        });
    }
}
