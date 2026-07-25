use tauri::State;
use crate::db::DbConn;
use crate::api::vod::VodStreamApi;
use crate::api::common::CategoryApi;
use chrono::Utc;

#[tauri::command]
pub fn get_vod_categories(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
) -> Result<Vec<CategoryApi>, String> {
    let conn = state.read().map_err(|e| e.to_string())?;
    crate::db::vod::query_categories(&conn, profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_vod_streams(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
    category_id: Option<String>,
    offset: u32,
    limit: u32,
) -> Result<Vec<VodStreamApi>, String> {
    let conn = state.read().map_err(|e| e.to_string())?;
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
    app: tauri::AppHandle,
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
    stream_id: i64,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;
    // 1. Resolve profile_id if not provided
    let p_id = {
        let conn = state.read().map_err(|e| e.to_string())?;
        match profile_id {
            Some(id) => id,
            None => {
                conn.query_row(
                    "SELECT profile_id FROM vod_streams WHERE stream_id = ?1 LIMIT 1",
                    rusqlite::params![stream_id],
                    |row| row.get(0),
                ).map_err(|e| e.to_string())?
            }
        }
    };

    // 2. Check DB Cache
    {
        let conn = state.read().map_err(|e| e.to_string())?;
        if let Some((info_json, _fetched_at)) = crate::db::vod::get_vod_info(&conn, p_id, stream_id).map_err(|e| e.to_string())? {
            let parsed: serde_json::Value = serde_json::from_str(&info_json).map_err(|e| e.to_string())?;
            return Ok(parsed);
        }
    }

    // 3. Fetch on-demand from Xtream API
    let registry = app.state::<crate::api::ClientRegistry>();
    let client = {
        let conn = state.read().map_err(|e| e.to_string())?;
        registry.get_or_create(p_id, &conn)?
    };

    let info_val = crate::api::vod::fetch_vod_info(&client, stream_id).await.map_err(|e| e.to_string())?;

    // 4. Cache in database
    {
        let conn = state.writer.lock().map_err(|e| e.to_string())?;
        let info_str = serde_json::to_string(&info_val).map_err(|e| e.to_string())?;
        let now = Utc::now().timestamp();
        crate::db::vod::upsert_vod_info(&conn, p_id, stream_id, &info_str, now).map_err(|e| e.to_string())?;
    }

    Ok(info_val)
}
