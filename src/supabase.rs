use std::env;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use tracing::{info, warn};

use crate::models::{Facility, PipeLine, Shop, Staff};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SupabaseClient {
    pub url: String,
    pub api_key: String,
    pub client: reqwest::Client,
}

#[allow(dead_code)]
impl SupabaseClient {
    pub fn from_env() -> Option<Self> {
        let url = env::var("SUPABASE_URL").ok()?;
        let api_key = env::var("SUPABASE_ANON_KEY")
            .or_else(|_| env::var("SUPABASE_SERVICE_KEY"))
            .or_else(|_| env::var("SUPABASE_KEY"))
            .ok()?;

        if url.is_empty() || api_key.is_empty() {
            return None;
        }

        let mut headers = HeaderMap::new();
        let auth_val = format!("Bearer {}", api_key);
        if let Ok(hv) = HeaderValue::from_str(&auth_val) {
            headers.insert(AUTHORIZATION, hv);
        }
        if let Ok(hv) = HeaderValue::from_str(&api_key) {
            headers.insert("apikey", hv);
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Prefer", HeaderValue::from_static("return=representation"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .ok()?;

        info!("⚡ Supabase 클라이언트 연결 설정 완료: {}", url);
        Some(Self { url, api_key, client })
    }

    // 1. 매장 목록 가져오기
    pub async fn get_shops(&self) -> Result<Vec<Shop>, reqwest::Error> {
        let endpoint = format!("{}/rest/v1/shops?select=*&order=id.asc", self.url);
        let res = self.client.get(&endpoint).send().await?;
        res.json::<Vec<Shop>>().await
    }

    // 2. 배관 목록 가져오기
    pub async fn get_pipes(&self) -> Result<Vec<PipeLine>, reqwest::Error> {
        let endpoint = format!("{}/rest/v1/pipes?select=*&order=id.asc", self.url);
        let res = self.client.get(&endpoint).send().await?;
        res.json::<Vec<PipeLine>>().await
    }

    // 3. 배관 삽입 / 갱신
    pub async fn upsert_pipe(&self, pipe: &PipeLine) -> Result<(), reqwest::Error> {
        let endpoint = format!("{}/rest/v1/pipes", self.url);
        let res = self.client.post(&endpoint)
            .header("Prefer", "resolution=merge-duplicates")
            .json(pipe)
            .send()
            .await?;
        if !res.status().is_success() {
            warn!("Supabase pipe upsert 실패: status {}", res.status());
        }
        Ok(())
    }

    // 4. 배관 삭제
    pub async fn delete_pipe(&self, id: &str) -> Result<(), reqwest::Error> {
        let endpoint = format!("{}/rest/v1/pipes?id=eq.{}", self.url, id);
        self.client.delete(&endpoint).send().await?;
        Ok(())
    }

    // 5. 시설물 목록 가져오기
    pub async fn get_facilities(&self) -> Result<Vec<Facility>, reqwest::Error> {
        let endpoint = format!("{}/rest/v1/facilities?select=*&order=id.asc", self.url);
        let res = self.client.get(&endpoint).send().await?;
        res.json::<Vec<Facility>>().await
    }

    // 6. 시설물 삽입 / 갱신
    pub async fn upsert_facility(&self, facility: &Facility) -> Result<(), reqwest::Error> {
        let endpoint = format!("{}/rest/v1/facilities", self.url);
        self.client.post(&endpoint)
            .header("Prefer", "resolution=merge-duplicates")
            .json(facility)
            .send()
            .await?;
        Ok(())
    }

    // 7. 직원 목록 가져오기
    pub async fn get_staff(&self) -> Result<Vec<Staff>, reqwest::Error> {
        let endpoint = format!("{}/rest/v1/staff?select=*&order=id.asc", self.url);
        let res = self.client.get(&endpoint).send().await?;
        res.json::<Vec<Staff>>().await
    }

    // 8. 백업용: 특정 시점 이후 변경된 레코드 추출 (증분 백업용)
    pub async fn get_changes_since(&self, since: DateTime<Utc>) -> Result<serde_json::Value, reqwest::Error> {
        let iso_time = since.to_rfc3339();
        
        let pipes_url = format!("{}/rest/v1/pipes?updated_at=gt.{}&select=*", self.url, iso_time);
        let facs_url = format!("{}/rest/v1/facilities?updated_at=gt.{}&select=*", self.url, iso_time);
        let shops_url = format!("{}/rest/v1/shops?updated_at=gt.{}&select=*", self.url, iso_time);
        let staff_url = format!("{}/rest/v1/staff?updated_at=gt.{}&select=*", self.url, iso_time);

        let pipes: Vec<PipeLine> = self.client.get(&pipes_url).send().await?.json().await.unwrap_or_default();
        let facs: Vec<Facility> = self.client.get(&facs_url).send().await?.json().await.unwrap_or_default();
        let shops: Vec<Shop> = self.client.get(&shops_url).send().await?.json().await.unwrap_or_default();
        let staff: Vec<Staff> = self.client.get(&staff_url).send().await?.json().await.unwrap_or_default();

        Ok(serde_json::json!({
            "since": iso_time,
            "pipes": pipes,
            "facilities": facs,
            "shops": shops,
            "staff": staff,
            "total_changed_count": pipes.len() + facs.len() + shops.len() + staff.len()
        }))
    }
}
