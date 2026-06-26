use tauri::State;
use crate::db::DbConn;
use crate::api::vod::VodStreamApi;
use crate::api::common::CategoryApi;
use crate::api::XtreamClient;
use chrono::Utc;

#[tauri::command]
pub fn get_vod_categories(
    state: State<'_, DbConn>,
    profile_id: i64,
) -> Result<Vec<CategoryApi>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::vod::query_categories(&conn, profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_vod_streams(
    state: State<'_, DbConn>,
    profile_id: i64,
    category_id: Option<String>,
    offset: u32,
    limit: u32,
) -> Result<Vec<VodStreamApi>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::vod::query_streams(
        &conn,
        profile_id,
        category_id.as_deref(),
        offset,
        limit,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_vod_info(
    state: State<'_, DbConn>,
    client: State<'_, XtreamClient>,
    profile_id: i64,
    stream_id: i64,
) -> Result<serde_json::Value, String> {
    // 1. Check DB Cache
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        if let Some((info_json, _fetched_at)) = crate::db::vod::get_vod_info(&conn, profile_id, stream_id).map_err(|e| e.to_string())? {
            let parsed: serde_json::Value = serde_json::from_str(&info_json).map_err(|e| e.to_string())?;
            return Ok(parsed);
        }
    }

    // 2. Fetch on-demand from Xtream API
    let info_val = crate::api::vod::fetch_vod_info(&client, stream_id).await.map_err(|e| e.to_string())?;

    // 3. Cache in database
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let info_str = serde_json::to_string(&info_val).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();
        crate::db::vod::upsert_vod_info(&conn, profile_id, stream_id, &info_str, &now).map_err(|e| e.to_string())?;
    }

    Ok(info_val)
}
