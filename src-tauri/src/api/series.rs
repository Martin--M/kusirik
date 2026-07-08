use serde::{Deserialize, Serialize};
use anyhow::Result;
use super::client::{XtreamClient, deserialize_option_string};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeriesCategoryApi {
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub category_id: Option<String>,
    pub category_name: String,
    #[serde(default)]
    pub profile_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeriesApi {
    pub series_id: i64,
    pub name: Option<String>,
    pub cover: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub category_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub rating: Option<String>,
    pub plot: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub is_favorite: Option<i32>,
    #[serde(default)]
    pub profile_id: Option<i64>,
}

pub async fn fetch_categories(client: &XtreamClient) -> Result<Vec<SeriesCategoryApi>> {
    client.fetch(Some("get_series_categories")).await
}

pub async fn fetch_series(client: &XtreamClient) -> Result<Vec<SeriesApi>> {
    client.fetch(Some("get_series")).await
}

pub async fn fetch_series_info(client: &XtreamClient, series_id: i64) -> Result<serde_json::Value> {
    let action = format!("get_series_info&series_id={}", series_id);
    client.fetch(Some(&action)).await
}
