use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn get_setting(
    state: State<'_, DbConn>,
    key: String,
) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::settings::get(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, DbConn>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::settings::set(&conn, &key, &value).map_err(|e| e.to_string())?;

    Ok(())
}
