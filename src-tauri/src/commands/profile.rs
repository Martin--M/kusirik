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
    pub id: Option<i64>,
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub password: Option<String>,
    pub epg_mode: Option<String>,
    pub profile_type: Option<String>,
}

#[tauri::command]
pub async fn save_profile(
    app: tauri::AppHandle,
    state: State<'_, DbConn>,
    payload: SaveProfilePayload,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;
    let conn_guard = state.writer.lock().map_err(|e| e.to_string())?;
    let conn = &*conn_guard;

    let epg_mode = payload.epg_mode.unwrap_or_else(|| "xmltv".to_string());
    let created_at = Utc::now().timestamp();
    let profile_type = payload.profile_type.unwrap_or_else(|| "xtream".to_string());

    let profile = Profile {
        id: payload.id.unwrap_or(0),
        name: payload.name.clone(),
        server_url: payload.server_url.clone(),
        username: payload.username.clone(),
        password: payload.password.clone().unwrap_or_default(),
        epg_mode: epg_mode.clone(),
        created_at,
        profile_type: profile_type.clone(),
    };

    // Save profile metadata
    let saved_id = crate::db::profile::insert(conn, &profile).map_err(|e| e.to_string())?;

    // Update client cache in registry
    let registry = app.state::<crate::api::ClientRegistry>();
    registry.update(saved_id, payload.server_url.clone(), payload.username.clone(), payload.password.clone().unwrap_or_default());

    Ok(json!({
        "id": saved_id,
        "name": payload.name,
        "server_url": payload.server_url,
        "username": payload.username,
        "password": payload.password.clone().unwrap_or_default(),
        "epg_mode": epg_mode,
        "created_at": created_at,
        "profile_type": profile_type,
    }))
}

#[tauri::command]
pub async fn add_public_iptv_profile(
    _app: tauri::AppHandle,
    state: State<'_, DbConn>,
) -> Result<serde_json::Value, String> {
    let conn_guard = state.writer.lock().map_err(|e| e.to_string())?;
    let conn = &*conn_guard;

    let existing = crate::db::profile::get_all(conn).map_err(|e| e.to_string())?;
    if let Some(p) = existing.iter().find(|p| p.profile_type == "public_iptv") {
        return Ok(json!({
            "id": p.id,
            "name": p.name,
            "server_url": p.server_url,
            "username": p.username,
            "password": p.password,
            "epg_mode": p.epg_mode,
            "created_at": p.created_at,
            "profile_type": p.profile_type,
        }));
    }

    let created_at = Utc::now().timestamp();
    let profile = Profile {
        id: 0,
        name: "Public IPTV".to_string(),
        server_url: "https://iptv-org.github.io/api".to_string(),
        username: "public".to_string(),
        password: "".to_string(),
        epg_mode: "xmltv".to_string(),
        created_at,
        profile_type: "public_iptv".to_string(),
    };

    let saved_id = crate::db::profile::insert(conn, &profile).map_err(|e| e.to_string())?;

    Ok(json!({
        "id": saved_id,
        "name": profile.name,
        "server_url": profile.server_url,
        "username": profile.username,
        "password": profile.password,
        "epg_mode": profile.epg_mode,
        "created_at": created_at,
        "profile_type": profile.profile_type,
    }))
}

#[tauri::command]
pub fn get_profiles(state: State<'_, DbConn>) -> Result<Vec<serde_json::Value>, String> {
    let conn = state.read().map_err(|e| e.to_string())?;

    match crate::db::profile::get_all(&conn) {
        Ok(profiles) => {
            let mapped = profiles.into_iter().map(|p| json!({
                "id": p.id,
                "name": p.name,
                "server_url": p.server_url,
                "username": p.username,
                "password": p.password,
                "epg_mode": p.epg_mode,
                "created_at": p.created_at,
                "profile_type": p.profile_type,
            })).collect();
            Ok(mapped)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_profile(
    state: State<'_, DbConn>,
    id: i64,
) -> Result<Option<serde_json::Value>, String> {
    let conn = state.read().map_err(|e| e.to_string())?;

    match crate::db::profile::get(&conn, id) {
        Ok(Some(p)) => Ok(Some(json!({
            "id": p.id,
            "name": p.name,
            "server_url": p.server_url,
            "username": p.username,
            "password": p.password,
            "epg_mode": p.epg_mode,
            "created_at": p.created_at,
            "profile_type": p.profile_type,
        }))),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn delete_profile(
    app: tauri::AppHandle,
    state: State<'_, DbConn>,
    id: i64,
) -> Result<(), String> {
    use tauri::Manager;
    let conn_guard = state.writer.lock().map_err(|e| e.to_string())?;
    let conn = &*conn_guard;



    // Cascade delete related profile data
    let tables = [
        "live_categories",
        "live_streams",
        "vod_categories",
        "vod_streams",
        "vod_info",
        "series_categories",
        "series",
        "series_info",
        "epg_entries",
        "playback_history",
        "sync_log",
    ];
    for table in tables {
        let query = format!("DELETE FROM {} WHERE profile_id = ?1", table);
        let _ = conn.execute(&query, rusqlite::params![id]);
    }

    // Delete profile metadata
    crate::db::profile::delete(conn, id).map_err(|e| e.to_string())?;

    // Remove from client registry
    let registry = app.state::<crate::api::ClientRegistry>();
    registry.remove(id);

    Ok(())
}
