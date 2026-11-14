use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FacilityCategory {
    Fire,     // 소방 (스프링클러, 소화전)
    Plumbing, // 위생, 급수, 오수 배관
    Hvac,     // 공조, 환기, 덕트
    Elec,     // 전기, 분전반, 간선
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,  // 관리자: 등록/수정/삭제, 직원관리, 도면작성 전체 권한
    Viewer, // 직원: 뷰어 전용 (층/매장/설비 조회만 가능)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub id: String,
    pub name: String,         // 예: 샤넬, 에르메스
    pub code: String,         // 예: CHANEL, HERMES
    pub floor: String,        // 예: 2F
    pub description: String,  // 설명 및 인입 제원
    pub pipe_count: usize,    // 연결 선로 수
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeLine {
    pub id: String,
    pub name: String,               // 예: 샤넬 인입 전용 급수관 50A
    pub shop: String,               // 연결 매장 (예: 샤넬)
    pub category: FacilityCategory, // PLUMBING, FIRE, ELEC, HVAC
    pub spec: String,               // 50A 스테인리스, F-CV 35sq 등
    pub length: String,             // 예: 18.4m
    pub points: String,             // SVG 좌표 목록 "x1,y1 x2,y2 ..."
    pub color: String,              // Hex 색상코드
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facility {
    pub id: String,
    pub name: String,               // 예: 샤넬 옥내소화전 2-1
    pub shop: String,               // 위치 매장 (예: 샤넬)
    pub category: FacilityCategory, // FIRE, ELEC 등
    pub floor: String,              // 2F
    pub x: f64,                     // 도면 X 좌표
    pub y: f64,                     // 도면 Y 좌표
    pub status: String,             // "정상", "점검필요"
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staff {
    pub id: String,
    pub name: String,
    pub pin: String,         // 4자리 PIN
    pub role: UserRole,      // admin | viewer
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub role: Option<UserRole>,
    pub name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupLog {
    pub id: String,
    pub backup_type: String, // "FULL" | "INCREMENTAL"
    pub target: String,      // "GOOGLE_DRIVE"
    pub file_name: String,
    pub file_id: Option<String>,
    pub records_count: usize,
    pub since_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BackupTriggerRequest {
    pub backup_type: Option<String>, // "FULL" | "INCREMENTAL"
}
