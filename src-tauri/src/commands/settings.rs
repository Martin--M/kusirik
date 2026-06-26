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
    client: State<'_, crate::api::XtreamClient>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::settings::set(&conn, &key, &value).map_err(|e| e.to_string())?;

    if key == "server_url" || key == "username" || key == "password" {
        let url = crate::db::settings::get(&conn, "server_url").map_err(|e| e.to_string())?.unwrap_or_default();
        let username = crate::db::settings::get(&conn, "username").map_err(|e| e.to_string())?.unwrap_or_default();
        let password = crate::db::settings::get(&conn, "password").map_err(|e| e.to_string())?.unwrap_or_default();
        client.update_credentials(url, username, password);
    }

    Ok(())
}
