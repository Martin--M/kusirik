#![allow(unused_variables)]

use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn get_epg_for_channel(
    state: State<'_, DbConn>,
    profile_id: i64,
    channel_id: String,
    from: String,
    to: String,
) -> Result<Vec<serde_json::Value>, String> {
    // Implemented in P3
    Ok(vec![])
}
