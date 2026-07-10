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

#[derive(serde::Deserialize)]
struct IptvCategory {
    id: String,
    name: String,
}

#[derive(serde::Deserialize)]
struct IptvChannel {
    id: String,
    name: String,
    categories: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
struct IptvLogo {
    channel: String,
    url: String,
}

#[derive(serde::Deserialize)]
struct IptvStream {
    channel: Option<String>,
    title: String,
    url: String,
}

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

pub async fn run_sync_all(app: AppHandle, profile_id: i64, force: bool) -> Result<()> {
    if IS_SYNCING.swap(true, Ordering::SeqCst) {
        tracing::warn!("Sync already in progress, ignoring request");
        return Ok(());
    }

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = do_sync(app_clone, profile_id, force).await {
            tracing::error!(error = %e, "Sync execution failed");
        }
        IS_SYNCING.store(false, Ordering::SeqCst);
    });

    Ok(())
}

async fn do_sync(app: AppHandle, profile_id: i64, force: bool) -> Result<()> {
    use tauri::Manager;
    let db_conn = app.state::<DbConn>();

    // Load profile to check type
    let profile = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        crate::db::profile::get(&conn, profile_id)?
            .ok_or_else(|| anyhow!("Profile not found"))?
    };

    if profile.profile_type == "public_iptv" {
        return do_sync_public_iptv(app, profile_id, force).await;
    }

    let registry = app.state::<crate::api::ClientRegistry>();
    let client = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        registry.get_or_create(profile_id, &conn).map_err(|e| anyhow!(e))?
    };

    // 2. Perform sequential syncs
    let data_types = vec!["live_streams", "vod_streams", "series", "epg"];
    for dt in data_types {
        // Check if stale
        let is_stale = {
            let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
            match get_last_sync_time(&conn, profile_id, dt)? {
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
                    "SELECT item_count FROM sync_log WHERE profile_id = ?1 AND data_type = ?2"
                )?;
                let mut rows = stmt.query(rusqlite::params![profile_id, dt])?;
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
                        let _ = record_error(&db_conn, profile_id, dt, &err_msg);
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
                    crate::db::live::upsert_categories(&mut conn, profile_id, &categories)?;
                    crate::db::live::upsert_streams(&mut conn, profile_id, &streams)?;
                    update_sync_log(&conn, profile_id, dt, Some(streams.len()), None)?;
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
                        let _ = record_error(&db_conn, profile_id, dt, &err_msg);
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
                    crate::db::vod::upsert_categories(&mut conn, profile_id, &categories)?;
                    crate::db::vod::upsert_streams(&mut conn, profile_id, &streams)?;
                    update_sync_log(&conn, profile_id, dt, Some(streams.len()), None)?;
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
                        let _ = record_error(&db_conn, profile_id, dt, &err_msg);
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
                    crate::db::series::upsert_categories(&mut conn, profile_id, &categories)?;
                    crate::db::series::upsert_series(&mut conn, profile_id, &series_list)?;
                    update_sync_log(&conn, profile_id, dt, Some(series_list.len()), None)?;
                }

                let _ = app.emit("sync://done", SyncDonePayload {
                    data_type: dt.to_string(),
                    count: series_list.len(),
                });
            }
            "epg" => {
                if let Err(e) = sync_epg_internal(app.clone(), profile_id, &client).await {
                    let err_msg = format!("Failed to sync EPG: {}", e);
                    let _ = record_error(&db_conn, profile_id, dt, &err_msg);
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

fn record_error(db_conn: &DbConn, profile_id: i64, data_type: &str, error: &str) -> Result<()> {
    let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
    update_sync_log(&conn, profile_id, data_type, None, Some(error))?;
    Ok(())
}

async fn fetch_with_retry<T, F, Fut>(fetch_fn: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    fetch_fn().await
}

async fn sync_epg_internal(app: AppHandle, profile_id: i64, client: &XtreamClient) -> Result<usize> {
    let db_conn = app.state::<DbConn>();

    let epg_mode = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        let profile = crate::db::profile::get(&conn, profile_id)?;
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

    let res = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .context("Failed to build HTTP client")?
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
    let mut count = 0;
    parse_and_insert_epg_xml(&app, &db_conn, profile_id, &body_bytes, &mut count)?;

    {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        update_sync_log(&conn, profile_id, "epg", Some(count), None)?;
    }

    let _ = app.emit("sync://done", SyncDonePayload {
        data_type: "epg".to_string(),
        count,
    });

    Ok(count)
}

fn hash_str_to_i64(s: &str) -> i64 {
    let mut hash = 2166136261u64;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(16777619);
    }
    (hash & 0x001FFFFFFFFFFFFF) as i64
}

async fn do_sync_public_iptv(app: AppHandle, profile_id: i64, force: bool) -> Result<()> {
    use tauri::Manager;
    let db_conn = app.state::<DbConn>();
    let data_types = vec!["live_streams", "vod_streams", "series", "epg"];

    for dt in data_types {
        let is_stale = {
            let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
            match get_last_sync_time(&conn, profile_id, dt)? {
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
            let item_count = {
                let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                let mut stmt = conn.prepare_cached(
                    "SELECT item_count FROM sync_log WHERE profile_id = ?1 AND data_type = ?2"
                )?;
                let mut rows = stmt.query(rusqlite::params![profile_id, dt])?;
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

        let _ = app.emit("sync://started", SyncStartedPayload {
            data_type: dt.to_string(),
        });

        match dt {
            "live_streams" => {
                if let Err(e) = sync_public_iptv_streams(app.clone(), profile_id).await {
                    let err_msg = format!("Failed to sync public IPTV streams: {}", e);
                    let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                    let _ = app.emit("sync://error", SyncErrorPayload {
                        data_type: dt.to_string(),
                        message: err_msg.clone(),
                    });
                    return Err(anyhow!(err_msg));
                }
            }
            "epg" => {
                if let Err(e) = sync_public_iptv_epg(app.clone(), profile_id).await {
                    let err_msg = format!("Failed to sync public IPTV EPG: {}", e);
                    let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                    let _ = app.emit("sync://error", SyncErrorPayload {
                        data_type: dt.to_string(),
                        message: err_msg.clone(),
                    });
                    return Err(anyhow!(err_msg));
                }
            }
            _ => {
                let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                update_sync_log(&conn, profile_id, dt, Some(0), None)?;
                let _ = app.emit("sync://progress", SyncProgressPayload {
                    data_type: dt.to_string(),
                    status: "done".to_string(),
                });
                let _ = app.emit("sync://done", SyncDonePayload {
                    data_type: dt.to_string(),
                    count: 0,
                });
            }
        }
    }
    Ok(())
}

async fn sync_public_iptv_streams(app: AppHandle, profile_id: i64) -> Result<usize> {
    use tauri::Manager;
    let db_conn = app.state::<DbConn>();

    let _ = app.emit("sync://progress", SyncProgressPayload {
        data_type: "live_streams".to_string(),
        status: "downloading".to_string(),
    });

    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
        
    let cats: Vec<IptvCategory> = http.get("https://iptv-org.github.io/api/categories.json").send().await?.json().await?;
    let channels: Vec<IptvChannel> = http.get("https://iptv-org.github.io/api/channels.json").send().await?.json().await?;
    let logos: Vec<IptvLogo> = http.get("https://iptv-org.github.io/api/logos.json").send().await?.json().await?;
    let streams: Vec<IptvStream> = http.get("https://iptv-org.github.io/api/streams.json").send().await?.json().await?;

    let _ = app.emit("sync://progress", SyncProgressPayload {
        data_type: "live_streams".to_string(),
        status: "parsing".to_string(),
    });

    let mut channel_map = std::collections::HashMap::new();
    for ch in channels {
        channel_map.insert(ch.id.clone(), ch);
    }

    let mut logo_map = std::collections::HashMap::new();
    for l in logos {
        logo_map.insert(l.channel.clone(), l.url);
    }

    let mut db_categories = Vec::new();
    for cat in cats {
        db_categories.push(crate::api::common::CategoryApi {
            category_id: Some(cat.id),
            category_name: cat.name,
            profile_id: Some(profile_id),
        });
    }

    let mut conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
    conn.execute("DELETE FROM live_streams WHERE profile_id = ?1", rusqlite::params![profile_id])?;
    conn.execute("DELETE FROM live_categories WHERE profile_id = ?1", rusqlite::params![profile_id])?;
    crate::db::live::upsert_categories(&mut conn, profile_id, &db_categories)?;

    let _ = app.emit("sync://progress", SyncProgressPayload {
        data_type: "live_streams".to_string(),
        status: "writing".to_string(),
    });

    let tx = conn.transaction()?;
    let count = streams.len();

    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO live_streams (
                profile_id, stream_id, name, stream_icon, epg_channel_id,
                category_id, tv_archive, tv_archive_duration, added, url
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(profile_id, stream_id) DO UPDATE SET
                name = excluded.name,
                stream_icon = excluded.stream_icon,
                epg_channel_id = excluded.epg_channel_id,
                category_id = excluded.category_id,
                tv_archive = excluded.tv_archive,
                tv_archive_duration = excluded.tv_archive_duration,
                added = excluded.added,
                url = excluded.url",
        )?;

        for stream in &streams {
            let stream_id = hash_str_to_i64(&stream.url);
            let ch_id = stream.channel.clone().unwrap_or_default();
            
            let (name, category_id) = if let Some(ch) = channel_map.get(&ch_id) {
                let cat_id = ch.categories.as_ref().and_then(|c| c.first().cloned()).unwrap_or_else(|| "0".to_string());
                (ch.name.clone(), cat_id)
            } else {
                (stream.title.clone(), "0".to_string())
            };

            let logo = logo_map.get(&ch_id).cloned();

            stmt.execute(rusqlite::params![
                profile_id,
                stream_id,
                name,
                logo,
                ch_id,
                category_id,
                0,
                0,
                "",
                stream.url,
            ])?;
        }
    }

    tx.commit()?;

    update_sync_log(&conn, profile_id, "live_streams", Some(count), None)?;

    let _ = app.emit("sync://done", SyncDonePayload {
        data_type: "live_streams".to_string(),
        count,
    });

    Ok(count)
}

async fn sync_public_iptv_epg(app: AppHandle, profile_id: i64) -> Result<usize> {
    let db_conn = app.state::<DbConn>();
    let _ = app.emit("sync://progress", SyncProgressPayload {
        data_type: "epg".to_string(),
        status: "downloading".to_string(),
    });

    let epg_urls = vec!["https://iptv-epg.org/files/epg-co.xml"];
    let mut total_count = 0;

    for url in epg_urls {
        tracing::info!(url = %url, "Fetching public EPG XML");
        let res = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?
            .get(url)
            .send()
            .await?;

        if !res.status().is_success() {
            tracing::warn!("Failed to fetch public EPG from {}, status: {}", url, res.status());
            continue;
        }

        let _ = app.emit("sync://progress", SyncProgressPayload {
            data_type: "epg".to_string(),
            status: "parsing".to_string(),
        });

        let body_bytes = res.bytes().await?;
        parse_and_insert_epg_xml(&app, &db_conn, profile_id, &body_bytes, &mut total_count)?;
    }

    {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        update_sync_log(&conn, profile_id, "epg", Some(total_count), None)?;
    }

    let _ = app.emit("sync://done", SyncDonePayload {
        data_type: "epg".to_string(),
        count: total_count,
    });

    Ok(total_count)
}

fn parse_and_insert_epg_xml(
    app: &AppHandle,
    db_conn: &DbConn,
    profile_id: i64,
    body_bytes: &[u8],
    accumulated_count: &mut usize,
) -> Result<()> {
    let mut reader = Reader::from_reader(std::io::Cursor::new(body_bytes));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut entries = Vec::new();
    let mut current_entry: Option<crate::db::epg::EpgEntry> = None;
    let mut inside_title = false;
    let mut inside_desc = false;

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

                        let (start_normalized, tz_offset) = match parse_xmltv_date_to_utc_and_offset(&start) {
                            Some((utc, off)) => (utc, Some(off)),
                            None => (start, None),
                        };
                        let stop_normalized = parse_xmltv_date_to_utc_and_offset(&stop)
                            .map(|(utc, _)| utc)
                            .unwrap_or(stop);

                        current_entry = Some(crate::db::epg::EpgEntry {
                            profile_id,
                            channel_id: channel,
                            start: start_normalized,
                            stop: stop_normalized,
                            title: None,
                            description: None,
                            tz_offset,
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
                            *accumulated_count += 1;

                            if entries.len() >= 2000 {
                                let mut conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                                crate::db::epg::bulk_insert(&mut conn, &entries)?;
                                entries.clear();
                                let _ = app.emit("sync://progress", SyncProgressPayload {
                                    data_type: "epg".to_string(),
                                    status: format!("writing ({} items)", *accumulated_count),
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

    Ok(())
}

fn parse_xmltv_date_to_utc_and_offset(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%d%H%M%S %z") {
        let utc = dt.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let secs = dt.offset().local_minus_utc();
        let hours = secs / 3600;
        let minutes = (secs % 3600).abs() / 60;
        let sign = if secs >= 0 { "+" } else { "-" };
        let offset = format!("{}{:02}:{:02}", sign, hours.abs(), minutes);
        return Some((utc, offset));
    }
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%d%H%M%S%z") {
        let utc = dt.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let secs = dt.offset().local_minus_utc();
        let hours = secs / 3600;
        let minutes = (secs % 3600).abs() / 60;
        let sign = if secs >= 0 { "+" } else { "-" };
        let offset = format!("{}{:02}:{:02}", sign, hours.abs(), minutes);
        return Some((utc, offset));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y%m%d%H%M%S") {
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        let utc = dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        return Some((utc, "+00:00".to_string()));
    }
    None
}

async fn run_epg_sync_startup_background_for_profile(app: AppHandle, profile: crate::db::profile::Profile) -> Result<()> {
    use tauri::Manager;
    let db_conn = app.state::<DbConn>();
    let _ = app.emit("sync://started", SyncStartedPayload { data_type: "epg".to_string() });

    if profile.profile_type == "public_iptv" {
        match sync_public_iptv_epg(app.clone(), profile.id).await {
            Ok(count) => {
                tracing::info!(profile_id = profile.id, count, "EPG startup background sync completed successfully");
            }
            Err(e) => {
                let err_msg = format!("Failed to sync EPG for profile {}: {}", profile.id, e);
                let _ = record_error(&db_conn, profile.id, "epg", &err_msg);
                let _ = app.emit("sync://error", SyncErrorPayload {
                    data_type: "epg".to_string(),
                    message: err_msg,
                });
                return Err(e);
            }
        }
        return Ok(());
    }

    let registry = app.state::<crate::api::ClientRegistry>();
    let client = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        registry.get_or_create(profile.id, &conn).map_err(|e| anyhow!(e))?
    };
    match sync_epg_internal(app.clone(), profile.id, &client).await {
        Ok(count) => {
            tracing::info!(profile_id = profile.id, count, "EPG startup background sync completed successfully");
        }
        Err(e) => {
            let err_msg = format!("Failed to sync EPG for profile {}: {}", profile.id, e);
            let _ = record_error(&db_conn, profile.id, "epg", &err_msg);
            let _ = app.emit("sync://error", SyncErrorPayload {
                data_type: "epg".to_string(),
                message: err_msg,
            });
            return Err(e);
        }
    }

    Ok(())
}

pub async fn run_startup_tasks(app: AppHandle) -> Result<()> {
    let db_conn = app.state::<DbConn>();

    let profiles = {
        let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        crate::db::profile::get_all(&conn)?
    };

    for profile in profiles {
        // 1. Cleanup old entries (ended > 48h ago)
        {
            let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
            let forty_eight_hours_ago = Utc::now() - chrono::Duration::hours(48);
            let before_timestamp = forty_eight_hours_ago.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let deleted = crate::db::epg::cleanup_old_entries(&conn, profile.id, &before_timestamp)?;
            tracing::info!(profile_id = profile.id, deleted_count = deleted, "EPG startup cleanup complete (removed items older than 48h)");
        }

        // 2. Check EPG Sync (24-hour limit on startup)
        let needs_epg_sync = {
            let conn = db_conn.0.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
            match get_last_sync_time(&conn, profile.id, "epg")? {
                Some(last_time) => {
                    let diff = Utc::now() - last_time;
                    diff.num_hours() >= 24
                }
                None => true,
            }
        };

        if needs_epg_sync {
            tracing::info!(profile_id = profile.id, "EPG sync is needed on startup. Starting background sync...");
            let app_clone = app.clone();
            let profile_clone = crate::db::profile::Profile {
                id: profile.id,
                name: profile.name.clone(),
                server_url: profile.server_url.clone(),
                username: profile.username.clone(),
                password: profile.password.clone(),
                epg_mode: profile.epg_mode.clone(),
                created_at: profile.created_at.clone(),
                profile_type: profile.profile_type.clone(),
            };
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_epg_sync_startup_background_for_profile(app_clone, profile_clone).await {
                    tracing::error!(profile_id = profile.id, error = %e, "EPG startup background sync failed");
                }
            });
        } else {
            tracing::info!(profile_id = profile.id, "EPG guide is fresh (last synced < 24h ago). Skipping startup sync.");
        }
    }

    Ok(())
}
