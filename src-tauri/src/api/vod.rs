use serde::{Deserialize, Serialize};
use anyhow::Result;
use super::client::{XtreamClient, deserialize_option_string};
use super::common::CategoryApi;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VodStreamApi {
    pub stream_id: i64,
    pub name: Option<String>,
    pub stream_icon: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub category_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub rating: Option<String>,
    pub container_extension: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub added: Option<String>,
    #[serde(default)]
    pub is_favorite: Option<i32>,
    #[serde(default)]
    pub profile_id: Option<i64>,
}

pub async fn fetch_categories(client: &XtreamClient) -> Result<Vec<CategoryApi>> {
    client.fetch(Some("get_vod_categories")).await
}

pub async fn fetch_streams(client: &XtreamClient) -> Result<Vec<VodStreamApi>> {
    client.fetch(Some("get_vod_streams")).await
}

pub async fn fetch_vod_info(client: &XtreamClient, stream_id: i64) -> Result<serde_json::Value> {
    let action = format!("get_vod_info&vod_id={}", stream_id);
    client.fetch(Some(&action)).await
}
