mod backup;
mod handlers;
mod models;
mod state;
mod supabase;

use std::net::SocketAddr;
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    backup::BackupManager,
    handlers::{
        create_facility_handler, create_pipe_handler, create_staff_handler,
        delete_facility_handler, delete_pipe_handler, delete_staff_handler,
        get_backup_logs_handler, get_facilities_handler, get_pipes_handler,
        get_shops_handler, get_staff_handler, login_handler, trigger_backup_handler,
    },
    state::AppState,
};

#[tokio::main]
async fn main() {
    // 1. 로깅 초기화
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "building_fms=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. 전역 상태 생성 (잠실점 에비뉴엘 2F 데이터)
    let state = AppState::new();

    // 3. 주간 구글 드라이브 자동 백업 스케줄러 시작 (첫째 주 FULL 백업, 이후 주간 INCREMENTAL 증분 백업)
    BackupManager::start_scheduler(state.clone());

    // 4. CORS 레이어 설정
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 5. API 라우터 정의
    let api_router = Router::new()
        // 인증
        .route("/auth/login", post(login_handler))
        // 매장
        .route("/shops", get(get_shops_handler))
        // 배관 및 전기 라인
        .route("/pipes", get(get_pipes_handler).post(create_pipe_handler))
        .route("/pipes/:id", delete(delete_pipe_handler))
        // 시설물
        .route("/facilities", get(get_facilities_handler).post(create_facility_handler))
        .route("/facilities/:id", delete(delete_facility_handler))
        // 직원 및 PIN
        .route("/staff", get(get_staff_handler).post(create_staff_handler))
        .route("/staff/:id", delete(delete_staff_handler))
        // 데이터베이스 백업 및 이력
        .route("/backup/trigger", post(trigger_backup_handler))
        .route("/backup/logs", get(get_backup_logs_handler))
        .with_state(state);

    // 6. 프론트엔드 정적 파일 서빙 (`./web` 디렉토리)
    let serve_dir = ServeDir::new("web");

    // 7. 전체 앱 라우터 조립
    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(serve_dir)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // 8. 서버 구동 (0.0.0.0으로 바인딩하여 안드로이드 태블릿/모바일 접속 허용)
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("🚀 잠실점 에비뉴엘 시설물 관제 서버 시작: http://0.0.0.0:8080 (로컬 접속: http://127.0.0.1:8080)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
