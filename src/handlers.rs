use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{Facility, LoginRequest, LoginResponse, PipeLine, Staff},
    state::AppState,
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

    // 매장 선로 개수 갱신
    {
        let mut shops = state.shops.write();
        if let Some(shop) = shops.iter_mut().find(|s| s.name == new_pipe.shop) {
            shop.pipe_count += 1;
        }
    }

    let saved = new_pipe.clone();
    state.pipes.write().push(new_pipe);

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
            }
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
    let saved = fac.clone();
    state.facilities.write().push(fac);
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
