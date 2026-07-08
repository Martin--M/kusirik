use tauri::State;
use crate::db::DbConn;
use crate::api::live::LiveStreamDto;
use crate::api::common::CategoryApi;

#[tauri::command]
pub fn get_live_categories(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
) -> Result<Vec<CategoryApi>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::live::query_categories(&conn, profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_live_streams(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
    category_id: Option<String>,
    offset: u32,
    limit: u32,
) -> Result<Vec<LiveStreamDto>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::live::query_streams(
        &conn,
        profile_id,
        category_id.as_deref(),
        offset,
        limit,
    ).map_err(|e| e.to_string())
}
