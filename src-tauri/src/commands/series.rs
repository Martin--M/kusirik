use tauri::State;
use crate::db::DbConn;
use crate::api::series::SeriesApi;
use crate::api::common::CategoryApi;
use crate::api::XtreamClient;
use chrono::Utc;

#[tauri::command]
pub fn get_series_categories(
    state: State<'_, DbConn>,
    profile_id: i64,
) -> Result<Vec<CategoryApi>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::series::query_categories(&conn, profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_series(
    state: State<'_, DbConn>,
    profile_id: i64,
    category_id: Option<String>,
    offset: u32,
    limit: u32,
) -> Result<Vec<SeriesApi>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::series::query_series(
        &conn,
        profile_id,
        category_id.as_deref(),
        offset,
        limit,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_series_info(
    state: State<'_, DbConn>,
    profile_id: i64,
    series_id: i64,
) -> Result<serde_json::Value, String> {
    // 1. Check DB Cache
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        if let Some((info_json, _fetched_at)) = crate::db::series::get_series_info(&conn, profile_id, series_id).map_err(|e| e.to_string())? {
            let parsed: serde_json::Value = serde_json::from_str(&info_json).map_err(|e| e.to_string())?;
            return Ok(parsed);
        }
    }

    // 2. Fetch on-demand from Xtream API
    let (url, username, password) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let url = crate::db::settings::get(&conn, "server_url").map_err(|e| e.to_string())?;
        let username = crate::db::settings::get(&conn, "username").map_err(|e| e.to_string())?;
        let password = crate::db::settings::get(&conn, "password").map_err(|e| e.to_string())?;
        (url, username, password)
    };

    let (url, username, password) = match (url, username, password) {
        (Some(u), Some(user), Some(pass)) => (u, user, pass),
        _ => return Err("Credentials missing".to_string()),
    };

    let client = XtreamClient::new(url, username, password);
    let info_val = crate::api::series::fetch_series_info(&client, series_id).await.map_err(|e| e.to_string())?;

    // 3. Cache in database
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let info_str = serde_json::to_string(&info_val).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();
        crate::db::series::upsert_series_info(&conn, profile_id, series_id, &info_str, &now).map_err(|e| e.to_string())?;
    }

    Ok(info_val)
}
