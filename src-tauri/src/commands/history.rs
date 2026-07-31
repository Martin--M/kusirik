use tauri::State;
use crate::db::DbConn;
use crate::commands::search::SearchResults;

#[tauri::command]
pub async fn record_playback_history(
    state: State<'_, DbConn>,
    profile_id: i64,
    media_type: String,
    stream_id: i64,
    last_episode_id: Option<i64>,
) -> Result<(), String> {
    let conn = state.writer.lock().map_err(|e| e.to_string())?;
    crate::db::history::record_playback(&conn, profile_id, &media_type, stream_id, last_episode_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_playback_history(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
) -> Result<SearchResults, String> {
    let conn = state.read().map_err(|e| e.to_string())?;
    crate::db::history::query_history(&conn, profile_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_from_playback_history(
    state: State<'_, DbConn>,
    profile_id: i64,
    media_type: String,
    stream_id: i64,
) -> Result<(), String> {
    let conn = state.writer.lock().map_err(|e| e.to_string())?;
    crate::db::history::delete_playback(&conn, profile_id, &media_type, stream_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_playback_history(
    state: State<'_, DbConn>,
    profile_id: i64,
) -> Result<(), String> {
    let conn = state.writer.lock().map_err(|e| e.to_string())?;
    crate::db::history::clear_history(&conn, profile_id)
        .map_err(|e| e.to_string())
}
