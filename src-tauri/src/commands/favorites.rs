use tauri::State;
use crate::db::DbConn;
use crate::commands::search::SearchResults;

#[tauri::command]
pub async fn toggle_favorite(
    state: State<'_, DbConn>,
    profile_id: i64,
    media_type: String,
    stream_id: i64,
) -> Result<bool, String> {
    let conn = state.writer.lock().map_err(|e| e.to_string())?;
    crate::db::favorites::toggle_favorite(&conn, profile_id, &media_type, stream_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_favorites(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
) -> Result<SearchResults, String> {
    let conn = state.read().map_err(|e| e.to_string())?;
    crate::db::favorites::query_favorites(&conn, profile_id)
        .map_err(|e| e.to_string())
}
