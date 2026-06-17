#![allow(unused_variables)]

use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn trigger_sync(data_type: &str) -> Result<(), String> {
    // Implemented in P1
    Ok(())
}

#[tauri::command]
pub fn get_sync_status(
    state: State<'_, DbConn>,
) -> Result<Vec<serde_json::Value>, String> {
    // Implemented in P1
    Ok(vec![])
}
