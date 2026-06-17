#![allow(unused_variables)]

use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn get_vod_categories(
    state: State<'_, DbConn>,
    profile_id: i64,
) -> Result<Vec<serde_json::Value>, String> {
    // Implemented in P4
    Ok(vec![])
}

#[tauri::command]
pub fn get_vod_streams(
    state: State<'_, DbConn>,
    profile_id: i64,
    category_id: Option<String>,
    offset: u32,
    limit: u32,
) -> Result<Vec<serde_json::Value>, String> {
    // Implemented in P4
    Ok(vec![])
}

#[tauri::command]
pub fn get_vod_info(
    state: State<'_, DbConn>,
    profile_id: i64,
    stream_id: i64,
) -> Result<serde_json::Value, String> {
    // Implemented in P4
    Ok(serde_json::Value::Null)
}
