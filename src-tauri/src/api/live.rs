use serde::{Deserialize, Serialize};
use anyhow::Result;
use super::client::{XtreamClient, deserialize_option_string, deserialize_option_i32};
use super::common::CategoryApi;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiveStreamApi {
    pub stream_id: i64,
    pub name: Option<String>,
    pub stream_icon: Option<String>,
    pub epg_channel_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub category_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_i32")]
    pub tv_archive: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_i32")]
    pub tv_archive_duration: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub added: Option<String>,
    #[serde(default)]
    pub is_favorite: Option<i32>,
    #[serde(default)]
    pub profile_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiveStreamDto {
    #[serde(flatten)]
    pub stream: LiveStreamApi,
    pub current_title: Option<String>,
}

pub async fn fetch_categories(client: &XtreamClient) -> Result<Vec<CategoryApi>> {
    client.fetch(Some("get_live_categories")).await
}

pub async fn fetch_streams(client: &XtreamClient) -> Result<Vec<LiveStreamApi>> {
    client.fetch(Some("get_live_streams")).await
}
