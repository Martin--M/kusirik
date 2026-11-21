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
    country: Option<String>,
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
    profile_id: i64,
    data_type: String,
    status: String, // "connecting", "downloading", "parsing", "writing", "done", "error"
}

#[derive(serde::Serialize, Clone)]
struct SyncStartedPayload {
    profile_id: i64,
    data_type: String,
}

#[derive(serde::Serialize, Clone)]
struct SyncDonePayload {
    profile_id: i64,
    data_type: String,
    count: usize,
}

#[derive(serde::Serialize, Clone)]
struct SyncErrorPayload {
    profile_id: i64,
    data_type: String,
    message: String,
}


fn emit_started(app: &AppHandle, profile_id: i64, data_type: &str) {
    let _ = app.emit("sync://started", SyncStartedPayload {
        profile_id,
        data_type: data_type.to_string(),
    });
}

fn emit_progress(app: &AppHandle, profile_id: i64, data_type: &str, status: &str) {
    let _ = app.emit("sync://progress", SyncProgressPayload {
        profile_id,
        data_type: data_type.to_string(),
        status: status.to_string(),
    });
}

fn emit_done(app: &AppHandle, profile_id: i64, data_type: &str, count: usize) {
    let _ = app.emit("sync://done", SyncDonePayload {
        profile_id,
        data_type: data_type.to_string(),
        count,
    });
}

fn emit_error(app: &AppHandle, profile_id: i64, data_type: &str, message: &str) {
    let _ = app.emit("sync://error", SyncErrorPayload {
        profile_id,
        data_type: data_type.to_string(),
        message: message.to_string(),
    });
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
        let conn = db_conn.read()?;
        crate::db::profile::get(&conn, profile_id)?
            .ok_or_else(|| anyhow!("Profile not found"))?
    };

    if profile.profile_type == "public_iptv" {
        return do_sync_public_iptv(app, profile_id, force).await;
    }

    let registry = app.state::<crate::api::ClientRegistry>();
    let client = {
        let conn = db_conn.read()?;
        registry.get_or_create(profile_id, &conn).map_err(|e| anyhow!(e))?
    };

    // 2. Perform sequential syncs
    let data_types = vec!["live_streams", "vod_streams", "series", "epg"];
    for dt in data_types {
        // Check if stale
        let is_stale = {
            let conn = db_conn.read()?;
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
                let conn = db_conn.read()?;
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

            emit_started(&app, profile_id, dt);
            emit_progress(&app, profile_id, dt, "done");
            emit_done(&app, profile_id, dt, item_count);
            continue;
        }

        tracing::info!(data_type = dt, "Starting sync");
        emit_started(&app, profile_id, dt);
        emit_progress(&app, profile_id, dt, "connecting");

        // Perform fetches based on type
        match dt {
            "live_streams" => {
                emit_progress(&app, profile_id, dt, "downloading");

                let categories_res = fetch_with_retry(|| crate::api::live::fetch_categories(&client)).await;
                let streams_res = fetch_with_retry(|| crate::api::live::fetch_streams(&client)).await;

                let (categories, streams) = match (categories_res, streams_res) {
                    (Ok(c), Ok(s)) => (c, s),
                    (Err(e), _) | (_, Err(e)) => {
                        let err_msg = format!("Failed to fetch live streams: {}", e);
                        let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                        emit_error(&app, profile_id, dt, &err_msg);
                        return Err(anyhow!(err_msg));
                    }
                };

                emit_progress(&app, profile_id, dt, "parsing");
                emit_progress(&app, profile_id, dt, "writing");

                {
                    let mut conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                    crate::db::live::upsert_categories(&mut conn, profile_id, &categories)?;
                    crate::db::live::upsert_streams(&mut conn, profile_id, &streams)?;
                    update_sync_log(&conn, profile_id, dt, Some(streams.len()), None)?;
                }

                emit_done(&app, profile_id, dt, streams.len());
            }
            "vod_streams" => {
                emit_progress(&app, profile_id, dt, "downloading");

                let categories_res = fetch_with_retry(|| crate::api::vod::fetch_categories(&client)).await;
                let streams_res = fetch_with_retry(|| crate::api::vod::fetch_streams(&client)).await;

                let (categories, streams) = match (categories_res, streams_res) {
                    (Ok(c), Ok(s)) => (c, s),
                    (Err(e), _) | (_, Err(e)) => {
                        let err_msg = format!("Failed to fetch VOD streams: {}", e);
                        let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                        emit_error(&app, profile_id, dt, &err_msg);
                        return Err(anyhow!(err_msg));
                    }
                };

                emit_progress(&app, profile_id, dt, "parsing");
                emit_progress(&app, profile_id, dt, "writing");

                {
                    let mut conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                    crate::db::vod::upsert_categories(&mut conn, profile_id, &categories)?;
                    crate::db::vod::upsert_streams(&mut conn, profile_id, &streams)?;
                    update_sync_log(&conn, profile_id, dt, Some(streams.len()), None)?;
                }

                emit_done(&app, profile_id, dt, streams.len());
            }
            "series" => {
                emit_progress(&app, profile_id, dt, "downloading");

                let categories_res = fetch_with_retry(|| crate::api::series::fetch_categories(&client)).await;
                let series_res = fetch_with_retry(|| crate::api::series::fetch_series(&client)).await;

                let (categories, series_list) = match (categories_res, series_res) {
                    (Ok(c), Ok(s)) => (c, s),
                    (Err(e), _) | (_, Err(e)) => {
                        let err_msg = format!("Failed to fetch series: {}", e);
                        let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                        emit_error(&app, profile_id, dt, &err_msg);
                        return Err(anyhow!(err_msg));
                    }
                };

                emit_progress(&app, profile_id, dt, "parsing");
                emit_progress(&app, profile_id, dt, "writing");

                {
                    let mut conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                    crate::db::series::upsert_categories(&mut conn, profile_id, &categories)?;
                    crate::db::series::upsert_series(&mut conn, profile_id, &series_list)?;
                    update_sync_log(&conn, profile_id, dt, Some(series_list.len()), None)?;
                }

                emit_done(&app, profile_id, dt, series_list.len());
            }
            "epg" => {
                if let Err(e) = sync_epg_internal(app.clone(), profile_id, &client).await {
                    let err_msg = format!("Failed to sync EPG: {}", e);
                    let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                    emit_error(&app, profile_id, dt, &err_msg);
                    return Err(anyhow!(err_msg));
                }
            }
            _ => {}
        }

        emit_progress(&app, profile_id, dt, "done");
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
        let ts: i64 = row.get(0)?;
        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
            Ok(Some(dt))
        } else {
            Ok(None)
        }
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
    let now = Utc::now().timestamp();
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
    let conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
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
        let conn = db_conn.read()?;
        let profile = crate::db::profile::get(&conn, profile_id)?;
        profile.map(|p| p.epg_mode).unwrap_or_else(|| "xmltv".to_string())
    };

    if epg_mode != "xmltv" {
        tracing::info!("EPG mode is not XMLTV, skipping EPG sync");
        return Ok(0);
    }

    emit_progress(&app, profile_id, "epg", "downloading");

    let xmltv_url = client.get_url(None).replace("player_api.php", "xmltv.php");
    tracing::info!(url = %xmltv_url, "Fetching XMLTV EPG data");

    let res = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?
        .get(&xmltv_url)
        .send()
        .await
        .context("Failed to fetch XMLTV stream")?;

    if !res.status().is_success() {
        return Err(anyhow!("HTTP request failed with status: {}", res.status()));
    }

    emit_progress(&app, profile_id, "epg", "parsing");

    let body_bytes = res.bytes().await.context("Failed to read XMLTV body bytes")?.to_vec();
    let count = 0;
    let app_clone = app.clone();
    let db_conn_clone = db_conn.inner().clone();
    let count = tokio::task::spawn_blocking(move || {
        parse_and_insert_epg_xml(app_clone, db_conn_clone, profile_id, body_bytes, count)
    }).await??;

    {
        let conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        update_sync_log(&conn, profile_id, "epg", Some(count), None)?;
    }

    emit_done(&app, profile_id, "epg", count);

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
            let conn = db_conn.read()?;
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
                let conn = db_conn.read()?;
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

            emit_started(&app, profile_id, dt);
            emit_progress(&app, profile_id, dt, "done");
            emit_done(&app, profile_id, dt, item_count);
            continue;
        }

        emit_started(&app, profile_id, dt);

        match dt {
            "live_streams" => {
                if let Err(e) = sync_public_iptv_streams(app.clone(), profile_id).await {
                    let err_msg = format!("Failed to sync public IPTV streams: {}", e);
                    let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                    emit_error(&app, profile_id, dt, &err_msg);
                    return Err(anyhow!(err_msg));
                }
            }
            "epg" => {
                if let Err(e) = sync_public_iptv_epg(app.clone(), profile_id).await {
                    let err_msg = format!("Failed to sync public IPTV EPG: {}", e);
                    let _ = record_error(&db_conn, profile_id, dt, &err_msg);
                    emit_error(&app, profile_id, dt, &err_msg);
                    return Err(anyhow!(err_msg));
                }
            }
            _ => {
                let conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
                update_sync_log(&conn, profile_id, dt, Some(0), None)?;
                emit_progress(&app, profile_id, dt, "done");
                emit_done(&app, profile_id, dt, 0);
            }
        }
    }
    Ok(())
}

async fn sync_public_iptv_streams(app: AppHandle, profile_id: i64) -> Result<usize> {
    use tauri::Manager;
    let db_conn = app.state::<DbConn>();

    emit_progress(&app, profile_id, "live_streams", "downloading");

    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
        
    let cats: Vec<IptvCategory> = http.get("https://iptv-org.github.io/api/categories.json").send().await?.json().await?;
    let channels: Vec<IptvChannel> = http.get("https://iptv-org.github.io/api/channels.json").send().await?.json().await?;
    let logos: Vec<IptvLogo> = http.get("https://iptv-org.github.io/api/logos.json").send().await?.json().await?;
    let streams: Vec<IptvStream> = http.get("https://iptv-org.github.io/api/streams.json").send().await?.json().await?;

    emit_progress(&app, profile_id, "live_streams", "parsing");

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

    let mut conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
    conn.execute("DELETE FROM live_streams WHERE profile_id = ?1", rusqlite::params![profile_id])?;
    conn.execute("DELETE FROM live_categories WHERE profile_id = ?1", rusqlite::params![profile_id])?;
    crate::db::live::upsert_categories(&mut conn, profile_id, &db_categories)?;

    emit_progress(&app, profile_id, "live_streams", "writing");

    let tx = conn.transaction()?;
    let count = streams.len();

    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO live_streams (
                profile_id, stream_id, name, stream_icon, epg_channel_id,
                category_id, tv_archive, tv_archive_duration, added, url, countries
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(profile_id, stream_id) DO UPDATE SET
                name = excluded.name,
                stream_icon = excluded.stream_icon,
                epg_channel_id = excluded.epg_channel_id,
                category_id = excluded.category_id,
                tv_archive = excluded.tv_archive,
                tv_archive_duration = excluded.tv_archive_duration,
                added = excluded.added,
                url = excluded.url,
                countries = excluded.countries",
        )?;

        for stream in &streams {
            let stream_id = hash_str_to_i64(&stream.url);
            let ch_id = stream.channel.clone().unwrap_or_default();
            
            let (name, category_id, countries) = if let Some(ch) = channel_map.get(&ch_id) {
                let cat_id = ch.categories.as_ref().and_then(|c| c.first().cloned()).unwrap_or_else(|| "0".to_string());
                let countrs = ch.country.clone();
                (ch.name.clone(), cat_id, countrs)
            } else {
                (stream.title.clone(), "0".to_string(), None)
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
                Option::<i64>::None,
                stream.url,
                countries,
            ])?;
        }
    }

    tx.commit()?;

    update_sync_log(&conn, profile_id, "live_streams", Some(count), None)?;

    emit_done(&app, profile_id, "live_streams", count);

    Ok(count)
}

async fn sync_public_iptv_epg(app: AppHandle, profile_id: i64) -> Result<usize> {
    let db_conn = app.state::<DbConn>().inner().clone();

    // Prune old EPG entries for public IPTV profile (keeping only starting from -24h to align with source data and optimize inserts)
    {
        let conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        let prune_cutoff = (Utc::now() - chrono::Duration::hours(24)).timestamp();
        match crate::db::epg::cleanup_old_entries(&conn, profile_id, prune_cutoff) {
            Ok(count) => tracing::info!(count, profile_id, cutoff = %prune_cutoff, "Pruned old EPG entries for public IPTV profile"),
            Err(e) => tracing::error!(error = ?e, "Failed to prune old EPG entries"),
        }
    }

    emit_progress(&app, profile_id, "epg", "downloading");

    let epg_urls = vec![
        "https://iptv-epg.org/files/epg-co.xml",
        "https://iptv-epg.org/files/epg-ar.xml",
        "https://iptv-epg.org/files/epg-be.xml",
        "https://iptv-epg.org/files/epg-au.xml",
        "https://iptv-epg.org/files/epg-ca.xml",
        "https://iptv-epg.org/files/epg-cl.xml",
        "https://iptv-epg.org/files/epg-fr.xml",
        "https://iptv-epg.org/files/epg-it.xml",
        "https://iptv-epg.org/files/epg-mx.xml",
        "https://iptv-epg.org/files/epg-kr.xml",
        "https://iptv-epg.org/files/epg-es.xml",
        "https://iptv-epg.org/files/epg-gb.xml",
        "https://iptv-epg.org/files/epg-us.xml",
        "https://iptv-epg.org/files/epg-uy.xml",
        "https://iptv-epg.org/files/epg-ve.xml",
    ];
    let mut total_count = 0;

    for url in epg_urls {
        tracing::info!(url = %url, "Fetching public EPG XML");
        let res = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()?
            .get(url)
            .send()
            .await?;

        if !res.status().is_success() {
            tracing::warn!("Failed to fetch public EPG from {}, status: {}", url, res.status());
            continue;
        }

        emit_progress(&app, profile_id, "epg", "parsing");

        let body_bytes = res.bytes().await?.to_vec();
        let app_clone = app.clone();
        let db_conn_clone = db_conn.clone();
        
        let count = tokio::task::spawn_blocking(move || {
            parse_and_insert_epg_xml(app_clone, db_conn_clone, profile_id, body_bytes, total_count)
        }).await??;
        
        total_count = count;
    }

    {
        let conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        update_sync_log(&conn, profile_id, "epg", Some(total_count), None)?;
    }

    emit_done(&app, profile_id, "epg", total_count);

    Ok(total_count)
}

fn parse_epg_xml_with_callback<F>(
    profile_id: i64,
    body_bytes: &[u8],
    batch_size: usize,
    mut on_batch: F,
) -> Result<usize>
where
    F: FnMut(&[crate::db::epg::EpgEntry], usize) -> Result<()>,
{
    let mut reader = Reader::from_reader(std::io::Cursor::new(body_bytes));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut entries = Vec::new();
    let mut current_entry: Option<crate::db::epg::EpgEntry> = None;
    let mut inside_title = false;
    let mut inside_desc = false;
    let mut parsed_count = 0usize;

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

                        let (start_normalized, tz_offset) = match parse_xmltv_date_to_timestamp_and_offset(&start) {
                            Some((ts, off)) => (ts, Some(off)),
                            None => continue,
                        };
                        let stop_normalized = match parse_xmltv_date_to_timestamp_and_offset(&stop) {
                            Some((ts, _)) => ts,
                            None => continue,
                        };

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
                            parsed_count += 1;

                            if entries.len() >= batch_size {
                                on_batch(&entries, parsed_count)?;
                                entries.clear();
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
        on_batch(&entries, parsed_count)?;
    }

    Ok(parsed_count)
}

fn parse_and_insert_epg_xml(
    app: AppHandle,
    db_conn: DbConn,
    profile_id: i64,
    body_bytes: Vec<u8>,
    accumulated_count: usize,
) -> Result<usize> {
    let added = parse_epg_xml_with_callback(profile_id, &body_bytes, 2000, |batch, parsed_so_far| {
        let mut conn = db_conn.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
        crate::db::epg::bulk_insert(&mut conn, batch)?;
        if batch.len() >= 2000 {
            emit_progress(
                &app,
                profile_id,
                "epg",
                &format!("writing ({} items)", accumulated_count + parsed_so_far),
            );
        }
        Ok(())
    })?;

    Ok(accumulated_count + added)
}

fn parse_xmltv_date_to_timestamp_and_offset(s: &str) -> Option<(i64, i32)> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%d%H%M%S %z") {
        let ts = dt.timestamp();
        let secs = dt.offset().local_minus_utc();
        return Some((ts, secs));
    }
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%d%H%M%S%z") {
        let ts = dt.timestamp();
        let secs = dt.offset().local_minus_utc();
        return Some((ts, secs));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y%m%d%H%M%S") {
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        return Some((dt.timestamp(), 0));
    }
    None
}

async fn run_epg_sync_startup_background_for_profile(app: AppHandle, profile: crate::db::profile::Profile) -> Result<()> {
    use tauri::Manager;
    let db_conn = app.state::<DbConn>();
    emit_started(&app, profile.id, "epg");

    if profile.profile_type == "public_iptv" {
        match sync_public_iptv_epg(app.clone(), profile.id).await {
            Ok(count) => {
                tracing::info!(profile_id = profile.id, count, "EPG startup background sync completed successfully");
            }
            Err(e) => {
                let err_msg = format!("Failed to sync EPG for profile {}: {}", profile.id, e);
                let _ = record_error(&db_conn, profile.id, "epg", &err_msg);
                emit_error(&app, profile.id, "epg", &err_msg);
                return Err(e);
            }
        }
        return Ok(());
    }

    let registry = app.state::<crate::api::ClientRegistry>();
    let client = {
        let conn = db_conn.read()?;
        registry.get_or_create(profile.id, &conn).map_err(|e| anyhow!(e))?
    };
    match sync_epg_internal(app.clone(), profile.id, &client).await {
        Ok(count) => {
            tracing::info!(profile_id = profile.id, count, "EPG startup background sync completed successfully");
        }
        Err(e) => {
            let err_msg = format!("Failed to sync EPG for profile {}: {}", profile.id, e);
            let _ = record_error(&db_conn, profile.id, "epg", &err_msg);
            emit_error(&app, profile.id, "epg", &err_msg);
            return Err(e);
        }
    }

    Ok(())
}

pub async fn run_startup_tasks(app: AppHandle) -> Result<()> {
    let db_conn = app.state::<DbConn>().inner().clone();

    // 1. Get all profiles (blocking DB read, offloaded)
    let db_conn_clone = db_conn.clone();
    let profiles = tokio::task::spawn_blocking(move || {
        let conn = db_conn_clone.read()?;
        crate::db::profile::get_all(&conn)
    }).await??;

    for profile in profiles {
        let profile_id = profile.id;

        // 2. Cleanup old entries (ended > 48h ago) - offloaded to spawn_blocking
        let db_conn_clone = db_conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db_conn_clone.writer.lock().map_err(|e| anyhow!("DB lock error: {}", e))?;
            let forty_eight_hours_ago = Utc::now() - chrono::Duration::hours(48);
            let before_timestamp = forty_eight_hours_ago.timestamp();
            let deleted = crate::db::epg::cleanup_old_entries(&conn, profile_id, before_timestamp)?;
            tracing::info!(profile_id, deleted_count = deleted, "EPG startup cleanup complete (removed items older than 48h)");
            Ok::<(), anyhow::Error>(())
        }).await??;

        // 3. Check EPG Sync (24-hour limit on startup) - offloaded to spawn_blocking
        let db_conn_clone = db_conn.clone();
        let needs_epg_sync = tokio::task::spawn_blocking(move || {
            let conn = db_conn_clone.read()?;
            let needs = match get_last_sync_time(&conn, profile_id, "epg")? {
                Some(last_time) => {
                    let diff = Utc::now() - last_time;
                    diff.num_hours() >= 24
                }
                None => true,
            };
            Ok::<bool, anyhow::Error>(needs)
        }).await??;

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
            // Spawn detached Tokio task so startup task loop finishes immediately without awaiting network downloads
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_epg_sync_startup_background_for_profile(app_clone, profile_clone).await {
                    tracing::error!(profile_id = profile_id, error = %e, "EPG startup background sync failed");
                }
            });
        } else {
            tracing::info!(profile_id = profile.id, "EPG guide is fresh (last synced < 24h ago). Skipping startup sync.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_str_to_i64_is_stable_and_js_safe() {
        const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
        let h1 = hash_str_to_i64("https://stream.example.com/live/ch1.m3u8");
        let h2 = hash_str_to_i64("https://stream.example.com/live/ch1.m3u8");
        let h3 = hash_str_to_i64("https://stream.example.com/live/ch2.m3u8");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert!(h1 >= 0 && h1 <= MAX_SAFE_INT);
        assert!(h3 >= 0 && h3 <= MAX_SAFE_INT);
    }

    #[test]
    fn test_parse_xmltv_date_to_timestamp_and_offset() {
        // Space-separated positive offset (+0200 = +7200s): 14:00+02:00 == 12:00 UTC
        assert_eq!(
            parse_xmltv_date_to_timestamp_and_offset("20261121140000 +0200"),
            Some((1795262400, 7200))
        );
        // Negative offset (-0500 = -18000s): 07:00-05:00 == 12:00 UTC
        assert_eq!(
            parse_xmltv_date_to_timestamp_and_offset("20261121070000 -0500"),
            Some((1795262400, -18000))
        );
        // Compact offset without space
        assert_eq!(
            parse_xmltv_date_to_timestamp_and_offset("20261121120000+0000"),
            Some((1795262400, 0))
        );
        // Naive UTC without offset
        assert_eq!(
            parse_xmltv_date_to_timestamp_and_offset("20261121120000"),
            Some((1795262400, 0))
        );
        assert_eq!(parse_xmltv_date_to_timestamp_and_offset("invalid"), None);
    }

    #[test]
    fn test_sync_log_tracks_success_and_error_states() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        assert!(get_last_sync_time(&conn, 1, "epg").unwrap().is_none());

        // Failed sync should be treated as no valid sync timestamp
        update_sync_log(&conn, 1, "epg", None, Some("network timeout")).unwrap();
        assert!(get_last_sync_time(&conn, 1, "epg").unwrap().is_none());

        // Successful sync records a valid timestamp
        update_sync_log(&conn, 1, "epg", Some(500), None).unwrap();
        assert!(get_last_sync_time(&conn, 1, "epg").unwrap().is_some());
    }

    #[test]
    fn test_parse_epg_xml_with_callback_parses_batches_and_skips_invalid() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <tv>
          <programme start="20261121120000 +0000" stop="20261121130000 +0000" channel="ch1">
            <title>Morning News</title>
            <desc>Daily headlines</desc>
          </programme>
          <programme start="bad-date" stop="20261121140000 +0000" channel="ch1">
            <title>Skipped Invalid</title>
          </programme>
          <programme start="20261121080000 -0500" stop="20261121090000 -0500" channel="ch2">
            <title>Sports Live</title>
          </programme>
        </tv>"#;

        let mut collected = Vec::new();
        let mut batch_calls = 0usize;
        let count = parse_epg_xml_with_callback(7, xml, 1, |batch, _| {
            batch_calls += 1;
            collected.extend_from_slice(batch);
            Ok(())
        })
        .unwrap();

        assert_eq!(count, 2);
        assert_eq!(batch_calls, 2);
        assert_eq!(collected.len(), 2);

        assert_eq!(collected[0].profile_id, 7);
        assert_eq!(collected[0].channel_id, "ch1");
        assert_eq!(collected[0].start, 1795262400);
        assert_eq!(collected[0].stop, 1795266000);
        assert_eq!(collected[0].title.as_deref(), Some("Morning News"));
        assert_eq!(collected[0].description.as_deref(), Some("Daily headlines"));
        assert_eq!(collected[0].tz_offset, Some(0));

        assert_eq!(collected[1].channel_id, "ch2");
        assert_eq!(collected[1].start, 1795266000);
        assert_eq!(collected[1].stop, 1795269600);
        assert_eq!(collected[1].title.as_deref(), Some("Sports Live"));
        assert_eq!(collected[1].description, None);
        assert_eq!(collected[1].tz_offset, Some(-18000));
    }

    #[test]
    fn test_parse_epg_xml_with_callback_errors_on_malformed_xml() {
        let bad_xml = b"<tv><programme><title>Unclosed</tv>";
        let res = parse_epg_xml_with_callback(1, bad_xml, 10, |_, _| Ok(()));
        assert!(res.is_err());
    }
}
