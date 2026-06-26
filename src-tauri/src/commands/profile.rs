use tauri::State;
use crate::db::DbConn;
use crate::api::XtreamClient;
use crate::db::profile::Profile;
use chrono::Utc;
use serde_json::json;

#[tauri::command]
pub async fn test_connection(
    url: String,
    username: String,
    password: String,
) -> Result<serde_json::Value, String> {
    let client = XtreamClient::new(url, username, password);
    match crate::api::auth::test_connection(&client).await {
        Ok(res) => Ok(json!({
            "status": "success",
            "allowed_formats": res.user_info.and_then(|u| u.allowed_output_formats).unwrap_or_default(),
        })),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(serde::Deserialize)]
pub struct SaveProfilePayload {
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub password: Option<String>,
    pub epg_mode: Option<String>,
}

#[tauri::command]
pub async fn save_profile(
    state: State<'_, DbConn>,
    client: State<'_, XtreamClient>,
    payload: SaveProfilePayload,
) -> Result<serde_json::Value, String> {
    let conn_guard = state.0.lock().map_err(|e| e.to_string())?;
    let conn = &*conn_guard;

    let epg_mode = payload.epg_mode.unwrap_or_else(|| "xmltv".to_string());
    let created_at = Utc::now().to_rfc3339();

    let profile = Profile {
        id: 1,
        name: payload.name.clone(),
        server_url: payload.server_url.clone(),
        username: payload.username.clone(),
        epg_mode: epg_mode.clone(),
        created_at: created_at.clone(),
    };

    // Save profile metadata
    let _ = crate::db::profile::delete(conn, 1); // clear existing if any
    crate::db::profile::insert(conn, &profile).map_err(|e| e.to_string())?;

    // Save settings
    crate::db::settings::set(conn, "server_url", &payload.server_url).map_err(|e| e.to_string())?;
    crate::db::settings::set(conn, "username", &payload.username).map_err(|e| e.to_string())?;
    if let Some(ref pass) = payload.password {
        crate::db::settings::set(conn, "password", pass).map_err(|e| e.to_string())?;
    }
    crate::db::settings::set(conn, "epg_mode", &epg_mode).map_err(|e| e.to_string())?;

    // Update managed client config
    client.update_credentials(
        payload.server_url.clone(),
        payload.username.clone(),
        payload.password.clone().unwrap_or_default(),
    );

    Ok(json!({
        "id": 1,
        "name": payload.name,
        "server_url": payload.server_url,
        "username": payload.username,
        "epg_mode": epg_mode,
        "created_at": created_at,
    }))
}

#[tauri::command]
pub fn get_profile(
    state: State<'_, DbConn>,
    id: i64,
) -> Result<Option<serde_json::Value>, String> {
    let conn_guard = state.0.lock().map_err(|e| e.to_string())?;
    let conn = &*conn_guard;

    match crate::db::profile::get(conn, id) {
        Ok(Some(p)) => Ok(Some(json!({
            "id": p.id,
            "name": p.name,
            "server_url": p.server_url,
            "username": p.username,
            "epg_mode": p.epg_mode,
            "created_at": p.created_at,
        }))),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn delete_profile(
    state: State<'_, DbConn>,
    client: State<'_, XtreamClient>,
    id: i64,
) -> Result<(), String> {
    let conn_guard = state.0.lock().map_err(|e| e.to_string())?;
    let conn = &*conn_guard;

    crate::db::profile::delete(conn, id).map_err(|e| e.to_string())?;
    // Clear credentials settings
    let _ = crate::db::settings::set(conn, "server_url", "");
    let _ = crate::db::settings::set(conn, "username", "");
    let _ = crate::db::settings::set(conn, "password", "");

    // Clear managed client config
    client.update_credentials("".to_string(), "".to_string(), "".to_string());

    Ok(())
}
