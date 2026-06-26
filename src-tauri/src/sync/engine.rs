use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, Emitter};
use anyhow::{Result, Context, anyhow};
use chrono::{Utc, DateTime, NaiveDateTime};
use rusqlite::Connection;
use crate::db::DbConn;
use crate::api::XtreamClient;
use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::XmlVersion;

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
    let data_types = vec!["live_streams", "vod_streams", "series", "epg"];
    for dt in data_types {
        // Check if stale
        let is_stale = {
            let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
            match get_last_sync_time(&conn, 1, dt)? {
                Some(last_time) => {
                    let diff = Utc::now() - last_time;
                    if dt == "epg" {
                        diff.num_hours() >= 12
                    } else {
                        diff.num_hours() >= 24
                    }
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
            "epg" => {
                if let Err(e) = sync_epg_internal(app.clone(), &client).await {
                    let err_msg = format!("Failed to sync EPG: {}", e);
                    let _ = record_error(&db_conn, dt, &err_msg);
                    let _ = app.emit("sync://error", SyncErrorPayload {
                        data_type: dt.to_string(),
                        message: err_msg.clone(),
                    });
                    return Err(anyhow!(err_msg));
                }
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

async fn sync_epg_internal(app: AppHandle, client: &XtreamClient) -> Result<usize> {
    let db_conn = app.state::<DbConn>();

    let epg_mode = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        let profile = crate::db::profile::get(&conn, 1)?;
        profile.map(|p| p.epg_mode).unwrap_or_else(|| "xmltv".to_string())
    };

    if epg_mode != "xmltv" {
        tracing::info!("EPG mode is not XMLTV, skipping EPG sync");
        return Ok(0);
    }

    let _ = app.emit("sync://progress", SyncProgressPayload {
        data_type: "epg".to_string(),
        status: "downloading".to_string(),
    });

    let xmltv_url = client.get_url(None).replace("player_api.php", "xmltv.php");
    tracing::info!(url = %xmltv_url, "Fetching XMLTV EPG data");

    let res = reqwest::Client::new()
        .get(&xmltv_url)
        .send()
        .await
        .context("Failed to fetch XMLTV stream")?;

    if !res.status().is_success() {
        return Err(anyhow!("HTTP request failed with status: {}", res.status()));
    }

    let _ = app.emit("sync://progress", SyncProgressPayload {
        data_type: "epg".to_string(),
        status: "parsing".to_string(),
    });

    let body_bytes = res.bytes().await.context("Failed to read XMLTV body bytes")?;
    let mut reader = Reader::from_reader(std::io::Cursor::new(&body_bytes));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut entries = Vec::new();
    let mut current_entry: Option<crate::db::epg::EpgEntry> = None;
    let mut inside_title = false;
    let mut inside_desc = false;
    let mut count = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"programme" => {
                        let mut start = String::new();
                        let mut stop = String::new();
                        let mut channel = String::new();
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"start" => {
                                    start = attr.decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())?.into_owned();
                                }
                                b"stop" => {
                                    stop = attr.decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())?.into_owned();
                                }
                                b"channel" => {
                                    channel = attr.decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())?.into_owned();
                                }
                                _ => {}
                            }
                        }

                        let start_normalized = parse_xmltv_date_to_utc(&start).unwrap_or(start);
                        let stop_normalized = parse_xmltv_date_to_utc(&stop).unwrap_or(stop);

                        current_entry = Some(crate::db::epg::EpgEntry {
                            profile_id: 1,
                            channel_id: channel,
                            start: start_normalized,
                            stop: stop_normalized,
                            title: None,
                            description: None,
                        });
                    }
                    b"title" if current_entry.is_some() => {
                        inside_title = true;
                    }
                    b"desc" if current_entry.is_some() => {
                        inside_desc = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if inside_title {
                    if let Some(ref mut entry) = current_entry {
                        let title = e.decode()?.into_owned();
                        entry.title = Some(title);
                    }
                } else if inside_desc {
                    if let Some(ref mut entry) = current_entry {
                        let desc = e.decode()?.into_owned();
                        entry.description = Some(desc);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"programme" => {
                        if let Some(entry) = current_entry.take() {
                            entries.push(entry);
                            count += 1;

                            if entries.len() >= 2000 {
                                let mut conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                                crate::db::epg::bulk_insert(&mut conn, &entries)?;
                                entries.clear();
                                let _ = app.emit("sync://progress", SyncProgressPayload {
                                    data_type: "epg".to_string(),
                                    status: format!("writing ({} items)", count),
                                });
                            }
                        }
                    }
                    b"title" => {
                        inside_title = false;
                    }
                    b"desc" => {
                        inside_desc = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    if !entries.is_empty() {
        let mut conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        crate::db::epg::bulk_insert(&mut conn, &entries)?;
    }

    {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        update_sync_log(&conn, 1, "epg", Some(count), None)?;
    }

    let _ = app.emit("sync://done", SyncDonePayload {
        data_type: "epg".to_string(),
        count,
    });

    Ok(count)
}

fn parse_xmltv_date_to_utc(s: &str) -> Option<String> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%d%H%M%S %z") {
        return Some(dt.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%d%H%M%S%z") {
        return Some(dt.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y%m%d%H%M%S") {
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        return Some(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
    None
}

pub async fn run_startup_tasks(app: AppHandle) -> Result<()> {
    let db_conn = app.state::<DbConn>();

    // 1. Cleanup old entries (ended > 48h ago)
    {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        let forty_eight_hours_ago = Utc::now() - chrono::Duration::hours(48);
        let before_timestamp = forty_eight_hours_ago.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let deleted = crate::db::epg::cleanup_old_entries(&conn, 1, &before_timestamp)?;
        tracing::info!(deleted_count = deleted, "EPG startup cleanup complete (removed items older than 48h)");
    }

    // 2. Check EPG Sync (24-hour limit on startup)
    let needs_epg_sync = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        match get_last_sync_time(&conn, 1, "epg")? {
            Some(last_time) => {
                let diff = Utc::now() - last_time;
                diff.num_hours() >= 24
            }
            None => true,
        }
    };

    if needs_epg_sync {
        tracing::info!("EPG sync is needed on startup. Starting background sync...");
        tauri::async_runtime::spawn(async move {
            if let Err(e) = run_epg_sync_startup_background(app).await {
                tracing::error!(error = %e, "EPG startup background sync failed");
            }
        });
    } else {
        tracing::info!("EPG guide is fresh (last synced < 24h ago). Skipping startup sync.");
    }

    Ok(())
}

async fn run_epg_sync_startup_background(app: AppHandle) -> Result<()> {
    let db_conn = app.state::<DbConn>();
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
            tracing::warn!("Credentials missing on startup, skipping EPG sync");
            return Ok(());
        }
    };

    let client = XtreamClient::new(url, username, password);
    let _ = app.emit("sync://started", SyncStartedPayload { data_type: "epg".to_string() });

    match sync_epg_internal(app.clone(), &client).await {
        Ok(count) => {
            tracing::info!(count, "EPG startup background sync completed successfully");
        }
        Err(e) => {
            let err_msg = format!("Failed to sync EPG: {}", e);
            let _ = record_error(&db_conn, "epg", &err_msg);
            let _ = app.emit("sync://error", SyncErrorPayload {
                data_type: "epg".to_string(),
                message: err_msg,
            });
            return Err(e);
        }
    }

    Ok(())
}
