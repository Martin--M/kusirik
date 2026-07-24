use tauri::State;
use crate::db::DbConn;
use crate::api::live::LiveStreamDto;
use crate::api::vod::VodStreamApi;
use crate::api::series::SeriesApi;
use serde::Serialize;

#[derive(Serialize)]
pub struct SearchResults {
    pub live: Vec<LiveStreamDto>,
    pub vod: Vec<VodStreamApi>,
    pub series: Vec<SeriesApi>,
}

#[tauri::command]
pub async fn search_all_media(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
    query: String,
) -> Result<SearchResults, String> {
    let conn = state.read().map_err(|e| e.to_string())?;
    let sql_query = format!("%{}%", query.trim());

    let live = crate::db::live::search_streams(&conn, profile_id, &sql_query, 30)
        .map_err(|e| e.to_string())?;

    let vod = crate::db::vod::search_streams(&conn, profile_id, &sql_query, 30)
        .map_err(|e| e.to_string())?;

    let series = crate::db::series::search_series(&conn, profile_id, &sql_query, 30)
        .map_err(|e| e.to_string())?;

    Ok(SearchResults { live, vod, series })
}
