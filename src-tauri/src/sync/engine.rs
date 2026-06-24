use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, Emitter};
use anyhow::{Result, Context, anyhow};
use chrono::{Utc, DateTime};
use rusqlite::Connection;
use crate::db::DbConn;
use crate::api::XtreamClient;

static IS_SYNCING: AtomicBool = AtomicBool::new(false);

#[derive(serde::Serialize, Clone)]
struct SyncProgressPayload {
    data_type: String,
    status: String, // "connecting", "downloading", "parsing", "writing", "done", "error"
}

#[derive(serde::Serialize, Clone)]
struct SyncStartedPayload {
    data_type: String,
}

#[derive(serde::Serialize, Clone)]
struct SyncDonePayload {
    data_type: String,
    count: usize,
}

#[derive(serde::Serialize, Clone)]
struct SyncErrorPayload {
    data_type: String,
    message: String,
}

pub async fn run_sync_all(app: AppHandle, force: bool) -> Result<()> {
    if IS_SYNCING.swap(true, Ordering::SeqCst) {
        tracing::warn!("Sync already in progress, ignoring request");
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::spawn(async move {
        if let Err(e) = do_sync(app_clone, force).await {
            tracing::error!(error = %e, "Sync execution failed");
        }
        IS_SYNCING.store(false, Ordering::SeqCst);
    });

    Ok(())
}

async fn do_sync(app: AppHandle, force: bool) -> Result<()> {
    let db_conn = app.state::<DbConn>();

    // 1. Get credentials
    let (url, username, password) = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        let url = crate::db::settings::get(&conn, "server_url")?;
        let username = crate::db::settings::get(&conn, "username")?;
        let password = crate::db::settings::get(&conn, "password")?;
        (url, username, password)
    };

    let (url, username, password) = match (url, username, password) {
        (Some(u), Some(user), Some(pass)) => (u, user, pass),
        _ => {
            tracing::error!("Credentials missing, aborting sync");
            let _ = app.emit("sync://error", "Credentials missing. Please configure settings first.");
            return Err(anyhow!("Credentials missing"));
        }
    };

    let client = XtreamClient::new(url, username, password);

    // 2. Perform sequential syncs
    let data_types = vec!["live_streams", "vod_streams", "series"];
    for dt in data_types {
        // Check if stale
        let is_stale = {
            let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
            match get_last_sync_time(&conn, 1, dt)? {
                Some(last_time) => {
                    let diff = Utc::now() - last_time;
                    diff.num_hours() >= 24
                }
                None => true,
            }
        };

        if !is_stale && !force {
            tracing::info!(data_type = dt, "Data is fresh, skipping sync");
            let item_count = {
                let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                let mut stmt = conn.prepare_cached(
                    "SELECT item_count FROM sync_log WHERE profile_id = 1 AND data_type = ?1"
                )?;
                let mut rows = stmt.query(rusqlite::params![dt])?;
                if let Some(row) = rows.next()? {
                    let ic: Option<i64> = row.get(0)?;
                    ic.unwrap_or(0) as usize
                } else {
                    0
                }
            };

            let _ = app.emit("sync://started", SyncStartedPayload {
                data_type: dt.to_string(),
            });
            let _ = app.emit("sync://progress", SyncProgressPayload {
                data_type: dt.to_string(),
                status: "done".to_string(),
            });
            let _ = app.emit("sync://done", SyncDonePayload {
                data_type: dt.to_string(),
                count: item_count,
            });
            continue;
        }

        tracing::info!(data_type = dt, "Starting sync");
        let _ = app.emit("sync://started", SyncStartedPayload {
            data_type: dt.to_string(),
        });

        let _ = app.emit("sync://progress", SyncProgressPayload {
            data_type: dt.to_string(),
            status: "connecting".to_string(),
        });

        // Perform fetches based on type
        match dt {
            "live_streams" => {
                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "downloading".to_string(),
                });

                let categories_res = fetch_with_retry(|| crate::api::live::fetch_categories(&client)).await;
                let streams_res = fetch_with_retry(|| crate::api::live::fetch_streams(&client)).await;

                let (categories, streams) = match (categories_res, streams_res) {
                    (Ok(c), Ok(s)) => (c, s),
                    (Err(e), _) | (_, Err(e)) => {
                        let err_msg = format!("Failed to fetch live streams: {}", e);
                        let _ = record_error(&db_conn, dt, &err_msg);
                        let _ = app.emit("sync://error", SyncErrorPayload {
                            data_type: dt.to_string(),
                            message: err_msg.clone(),
                        });
                        return Err(anyhow!(err_msg));
                    }
                };

                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "parsing".to_string(),
                });

                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "writing".to_string(),
                });

                {
                    let mut conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                    crate::db::live::upsert_categories(&mut conn, 1, &categories)?;
                    crate::db::live::upsert_streams(&mut conn, 1, &streams)?;
                    update_sync_log(&conn, 1, dt, Some(streams.len()), None)?;
                }

                let _ = app.emit("sync://done", SyncDonePayload {
                    data_type: dt.to_string(),
                    count: streams.len(),
                });
            }
            "vod_streams" => {
                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "downloading".to_string(),
                });

                let categories_res = fetch_with_retry(|| crate::api::vod::fetch_categories(&client)).await;
                let streams_res = fetch_with_retry(|| crate::api::vod::fetch_streams(&client)).await;

                let (categories, streams) = match (categories_res, streams_res) {
                    (Ok(c), Ok(s)) => (c, s),
                    (Err(e), _) | (_, Err(e)) => {
                        let err_msg = format!("Failed to fetch VOD streams: {}", e);
                        let _ = record_error(&db_conn, dt, &err_msg);
                        let _ = app.emit("sync://error", SyncErrorPayload {
                            data_type: dt.to_string(),
                            message: err_msg.clone(),
                        });
                        return Err(anyhow!(err_msg));
                    }
                };

                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "parsing".to_string(),
                });

                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "writing".to_string(),
                });

                {
                    let mut conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                    crate::db::vod::upsert_categories(&mut conn, 1, &categories)?;
                    crate::db::vod::upsert_streams(&mut conn, 1, &streams)?;
                    update_sync_log(&conn, 1, dt, Some(streams.len()), None)?;
                }

                let _ = app.emit("sync://done", SyncDonePayload {
                    data_type: dt.to_string(),
                    count: streams.len(),
                });
            }
            "series" => {
                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "downloading".to_string(),
                });

                let categories_res = fetch_with_retry(|| crate::api::series::fetch_categories(&client)).await;
                let series_res = fetch_with_retry(|| crate::api::series::fetch_series(&client)).await;

                let (categories, series_list) = match (categories_res, series_res) {
                    (Ok(c), Ok(s)) => (c, s),
                    (Err(e), _) | (_, Err(e)) => {
                        let err_msg = format!("Failed to fetch series: {}", e);
                        let _ = record_error(&db_conn, dt, &err_msg);
                        let _ = app.emit("sync://error", SyncErrorPayload {
                            data_type: dt.to_string(),
                            message: err_msg.clone(),
                        });
                        return Err(anyhow!(err_msg));
                    }
                };

                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "parsing".to_string(),
                });

                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "writing".to_string(),
                });

                {
                    let mut conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                    crate::db::series::upsert_categories(&mut conn, 1, &categories)?;
                    crate::db::series::upsert_series(&mut conn, 1, &series_list)?;
                    update_sync_log(&conn, 1, dt, Some(series_list.len()), None)?;
                }

                let _ = app.emit("sync://done", SyncDonePayload {
                    data_type: dt.to_string(),
                    count: series_list.len(),
                });
            }
            _ => {}
        }

        let _ = app.emit("sync://progress", SyncProgressPayload {
            data_type: dt.to_string(),
            status: "done".to_string(),
        });
    }

    Ok(())
}

fn get_last_sync_time(conn: &Connection, profile_id: i64, data_type: &str) -> Result<Option<DateTime<Utc>>> {
    let mut stmt = conn.prepare_cached(
        "SELECT fetched_at, last_error FROM sync_log WHERE profile_id = ?1 AND data_type = ?2"
    )?;
    let mut rows = stmt.query(rusqlite::params![profile_id, data_type])?;
    if let Some(row) = rows.next()? {
        let last_error: Option<String> = row.get(1)?;
        if last_error.is_some() {
            // The last sync failed, so we treat it as if there was no successful sync
            return Ok(None);
        }
        let val: String = row.get(0)?;
        let dt = DateTime::parse_from_rfc3339(&val)
            .map(|dt| dt.with_timezone(&Utc))
            .context("Failed to parse ISO 8601 string")?;
        Ok(Some(dt))
    } else {
        Ok(None)
    }
}

fn update_sync_log(
    conn: &Connection,
    profile_id: i64,
    data_type: &str,
    item_count: Option<usize>,
    error: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO sync_log (profile_id, data_type, fetched_at, item_count, last_error)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            profile_id,
            data_type,
            now,
            item_count.map(|c| c as i64),
            error
        ],
    )?;
    Ok(())
}

fn record_error(db_conn: &DbConn, data_type: &str, error: &str) -> Result<()> {
    let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
    update_sync_log(&conn, 1, data_type, None, Some(error))?;
    Ok(())
}

async fn fetch_with_retry<T, F, Fut>(fetch_fn: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    fetch_fn().await
}
