#![allow(unused_variables)]

use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn save_profile(
    state: State<'_, DbConn>,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    // Implemented in P1
    Ok(serde_json::Value::Null)
}

#[tauri::command]
pub fn get_profile(
    state: State<'_, DbConn>,
    id: i64,
) -> Result<Option<serde_json::Value>, String> {
    // Implemented in P1
    Ok(None)
}

#[tauri::command]
pub fn delete_profile(
    state: State<'_, DbConn>,
    id: i64,
) -> Result<(), String> {
    // Implemented in P1
    Ok(())
}
