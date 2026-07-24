use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub async fn trigger_sync(
    app: tauri::AppHandle,
    profile_id: i64,
    _data_type: Option<String>,
    force: Option<bool>,
) -> Result<(), String> {
    crate::sync::engine::run_sync_all(app, profile_id, force.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_status(
    state: State<'_, DbConn>,
    profile_id: i64,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = state.read().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT data_type, fetched_at, item_count, last_error FROM sync_log WHERE profile_id = ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![profile_id], |row| {
            Ok(serde_json::json!({
                "data_type": row.get::<_, String>(0)?,
                "fetched_at": row.get::<_, String>(1)?,
                "item_count": row.get::<_, Option<i64>>(2)?,
                "last_error": row.get::<_, Option<String>>(3)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let mut list = vec![];
    for item in rows.flatten() {
        list.push(item);
    }

    Ok(list)
}
