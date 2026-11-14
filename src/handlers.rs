use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    backup::BackupManager,
    models::{BackupTriggerRequest, Facility, LoginRequest, LoginResponse, PipeLine, Staff},
    state::AppState,
    supabase::SupabaseClient,
};

// 1. PIN 인증 핸들러 (4자리 PIN)
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let pin = payload.pin.trim();
    let staff_list = state.staff.read();
    
    if let Some(user) = staff_list.iter().find(|s| s.pin == pin) {
        (
            StatusCode::OK,
            Json(LoginResponse {
                success: true,
                role: Some(user.role.clone()),
                name: Some(user.name.clone()),
                message: format!("환영합니다, {} 님.", user.name),
            }),
        )
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                success: false,
                role: None,
                name: None,
                message: "등록되지 않은 PIN 번호입니다. 관리자에게 문의하세요.".into(),
            }),
        )
    }
}

// 2. 매장 목록 핸들러
pub async fn get_shops_handler(State(state): State<AppState>) -> impl IntoResponse {
    let shops = state.shops.read().clone();
    (StatusCode::OK, Json(shops))
}

// 3. 배관 및 전기 라인 조회 핸들러 (쿼리 파라미터 ?shop=샤넬 지원)
#[derive(Debug, Deserialize)]
pub struct PipeQuery {
    pub shop: Option<String>,
}

pub async fn get_pipes_handler(
    State(state): State<AppState>,
    Query(query): Query<PipeQuery>,
) -> impl IntoResponse {
    let pipes = state.pipes.read();
    let result: Vec<PipeLine> = if let Some(shop_name) = query.shop {
        pipes.iter().filter(|p| p.shop == shop_name).cloned().collect()
    } else {
        pipes.clone()
    };
    (StatusCode::OK, Json(result))
}

// 4. 새 배관/전기 라인 등록 (관리자 권한)
pub async fn create_pipe_handler(
    State(state): State<AppState>,
    Json(mut new_pipe): Json<PipeLine>,
) -> impl IntoResponse {
    if new_pipe.id.is_empty() {
        new_pipe.id = format!("P-{}", Uuid::new_v4().simple().to_string()[..8].to_uppercase());
    }
    new_pipe.updated_at = Utc::now();

    // 매장 선로 개수 갱신
    {
        let mut shops = state.shops.write();
        if let Some(shop) = shops.iter_mut().find(|s| s.name == new_pipe.shop) {
            shop.pipe_count += 1;
            shop.updated_at = Utc::now();
        }
    }

    let saved = new_pipe.clone();
    state.pipes.write().push(new_pipe);

    // Supabase 주 DB 비동기 동기화
    if let Some(sb) = SupabaseClient::from_env() {
        let p = saved.clone();
        tokio::spawn(async move {
            let _ = sb.upsert_pipe(&p).await;
        });
    }

    (StatusCode::CREATED, Json(saved))
}

// 5. 배관/전기 라인 삭제
pub async fn delete_pipe_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut pipes = state.pipes.write();
    if let Some(pos) = pipes.iter().position(|p| p.id == id) {
        let removed = pipes.remove(pos);
        // 매장 선로 개수 감소
        let mut shops = state.shops.write();
        if let Some(shop) = shops.iter_mut().find(|s| s.name == removed.shop) {
            if shop.pipe_count > 0 {
                shop.pipe_count -= 1;
                shop.updated_at = Utc::now();
            }
        }

        // Supabase 삭제 동기화
        if let Some(sb) = SupabaseClient::from_env() {
            let pid = id.clone();
            tokio::spawn(async move {
                let _ = sb.delete_pipe(&pid).await;
            });
        }

        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// 6. 시설물 목록 조회
pub async fn get_facilities_handler(State(state): State<AppState>) -> impl IntoResponse {
    let list = state.facilities.read().clone();
    (StatusCode::OK, Json(list))
}

// 7. 시설물 등록
pub async fn create_facility_handler(
    State(state): State<AppState>,
    Json(mut fac): Json<Facility>,
) -> impl IntoResponse {
    if fac.id.is_empty() {
        fac.id = format!("F-{}", Uuid::new_v4().simple().to_string()[..8].to_uppercase());
    }
    fac.updated_at = Utc::now();
    let saved = fac.clone();
    state.facilities.write().push(fac);

    if let Some(sb) = SupabaseClient::from_env() {
        let f = saved.clone();
        tokio::spawn(async move {
            let _ = sb.upsert_facility(&f).await;
        });
    }

    (StatusCode::CREATED, Json(saved))
}

// 8. 시설물 삭제
pub async fn delete_facility_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut facs = state.facilities.write();
    if let Some(pos) = facs.iter().position(|f| f.id == id) {
        facs.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// 9. 직원 목록 조회
pub async fn get_staff_handler(State(state): State<AppState>) -> impl IntoResponse {
    let staff = state.staff.read().clone();
    (StatusCode::OK, Json(staff))
}

// 10. 직원 등록
pub async fn create_staff_handler(
    State(state): State<AppState>,
    Json(mut member): Json<Staff>,
) -> impl IntoResponse {
    if member.id.is_empty() {
        member.id = format!("ST-{}", Uuid::new_v4().simple().to_string()[..6].to_uppercase());
    }
    member.updated_at = Utc::now();
    let saved = member.clone();
    state.staff.write().push(member);
    (StatusCode::CREATED, Json(saved))
}

// 11. 직원 삭제
pub async fn delete_staff_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut staff = state.staff.write();
    if let Some(pos) = staff.iter().position(|s| s.id == id) {
        staff.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// 12. 백업 즉시 실행 핸들러 (수동 트리거: FULL 또는 INCREMENTAL)
pub async fn trigger_backup_handler(
    State(state): State<AppState>,
    Json(payload): Json<BackupTriggerRequest>,
) -> impl IntoResponse {
    match BackupManager::execute_backup(&state, payload.backup_type).await {
        Ok(log) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": format!("{} 백업이 성공적으로 완료되었습니다.", log.backup_type),
            "log": log
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "success": false,
            "message": format!("백업 실행 실패: {}", e)
        })))
    }
}

// 13. 백업 이력 조회 핸들러
pub async fn get_backup_logs_handler(State(state): State<AppState>) -> impl IntoResponse {
    let logs = state.backup_logs.read().clone();
    (StatusCode::OK, Json(logs))
}
