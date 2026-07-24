use tauri::State;
use crate::db::DbConn;
use crate::db::epg::EpgEntry;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

type EpgFetchCache = HashMap<(i64, String), DateTime<Utc>>;
static LAST_FETCH_TIMES: OnceLock<Mutex<EpgFetchCache>> = OnceLock::new();

#[tauri::command]
pub async fn get_epg_for_channel(
    app: tauri::AppHandle,
    state: State<'_, DbConn>,
    profile_id: i64,
    channel_id: String,
    from: String,
    to: String,
) -> Result<Vec<EpgEntry>, String> {
    use tauri::Manager;
    // 1. Query the local database first
    let db_conn = state;
    let list = {
        let conn = db_conn.read().map_err(|e| e.to_string())?;
        crate::db::epg::query_for_channel(&conn, profile_id, &channel_id, &from, &to)
            .map_err(|e| e.to_string())?
    };

    if !list.is_empty() {
        return Ok(list);
    }

    // Check in-memory rate limit for on-demand fetches (cap to once per 2 hours per channel)
    let should_fetch = {
        let cache = LAST_FETCH_TIMES.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = cache.lock().map_err(|e| e.to_string())?;
        let now = Utc::now();
        let key = (profile_id, channel_id.clone());
        if let Some(&last_time) = map.get(&key) {
            if now - last_time < chrono::Duration::hours(2) {
                false
            } else {
                map.insert(key, now);
                true
            }
        } else {
            map.insert(key, now);
            true
        }
    };

    if !should_fetch {
        return Ok(vec![]);
    }

    // 2. Fallback: If empty, do on-demand fetch if epg_channel_id maps to a stream_id in live_streams
    let stream_id: Option<i64> = {
        let conn = db_conn.read().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT stream_id FROM live_streams WHERE profile_id = ?1 AND epg_channel_id = ?2 LIMIT 1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(rusqlite::params![profile_id, channel_id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let id: i64 = row.get(0).map_err(|e| e.to_string())?;
            Some(id)
        } else {
            None
        }
    };

    let stream_id = match stream_id {
        Some(id) => id,
        None => return Ok(vec![]),
    };

    // Check if the profile is public IPTV (which doesn't support on-demand Xtream short_epg)
    let is_public = {
        let conn = db_conn.read().map_err(|e| e.to_string())?;
        let p = crate::db::profile::get(&conn, profile_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Profile not found".to_string())?;
        p.profile_type == "public_iptv"
    };

    if is_public {
        return Ok(vec![]);
    }

    // Fetch EPG listings on demand
    let registry = app.state::<crate::api::ClientRegistry>();
    let client = {
        let conn = db_conn.read().map_err(|e| e.to_string())?;
        registry.get_or_create(profile_id, &conn)?
    };

    let listings_res = crate::api::epg::fetch_short_epg(&client, stream_id).await;
    let listings = match listings_res {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(error = %e, "On-demand short EPG fetch failed");
            return Ok(vec![]);
        }
    };

    // Convert API programs to EPG entries and insert into database
    let mut epg_entries = Vec::new();
    for item in listings {
        let mut start_iso = None;
        let mut stop_iso = None;
        let mut tz_offset = None;

        if let Some(ref start_ts) = item.start_timestamp {
            if let Ok(ts) = start_ts.parse::<i64>() {
                if let Some(dt) = DateTime::from_timestamp(ts, 0) {
                    start_iso = Some(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
                }
            }
        }
        if let Some(ref stop_ts) = item.stop_timestamp {
            if let Ok(ts) = stop_ts.parse::<i64>() {
                if let Some(dt) = DateTime::from_timestamp(ts, 0) {
                    stop_iso = Some(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
                }
            }
        }

        // Infer timezone offset by comparing start_timestamp with start local date string
        if let (Some(ref start_ts), Some(ref start_local_str)) = (&item.start_timestamp, &item.start) {
            if let (Ok(ts), Ok(naive_local)) = (
                start_ts.parse::<i64>(),
                chrono::NaiveDateTime::parse_from_str(start_local_str.trim(), "%Y-%m-%d %H:%M:%S")
            ) {
                let local_ts = naive_local.and_utc().timestamp();
                let diff_secs = ts - local_ts;
                let hours = diff_secs / 3600;
                let minutes = (diff_secs % 3600).abs() / 60;
                let sign = if diff_secs >= 0 { "+" } else { "-" };
                tz_offset = Some(format!("{}{:02}:{:02}", sign, hours.abs(), minutes));
            }
        }

        let start_iso = start_iso.or_else(|| item.start.as_ref().and_then(|s| crate::api::epg::parse_api_date_to_utc(s)));
        let stop_iso = stop_iso.or_else(|| item.end.as_ref().and_then(|s| crate::api::epg::parse_api_date_to_utc(s)));

        let (start, stop) = match (start_iso, stop_iso) {
            (Some(s), Some(e)) => (s, e),
            _ => continue,
        };

        // Clean base64 strings if present (such as in TSN sample epg listing)
        let title_clean = item.title.map(|t| crate::api::epg::maybe_decode_base64(&t));
        let desc_clean = item.description.map(|d| crate::api::epg::maybe_decode_base64(&d));

        epg_entries.push(EpgEntry {
            profile_id,
            channel_id: channel_id.clone(),
            start,
            stop,
            title: title_clean,
            description: desc_clean,
            tz_offset,
        });
    }

    if !epg_entries.is_empty() {
        let mut conn = db_conn.writer.lock().map_err(|e| e.to_string())?;
        crate::db::epg::bulk_insert(&mut conn, &epg_entries).map_err(|e| e.to_string())?;
    }

    // Re-query from DB with a wider window (now-1h to now+24h) so that entries
    // returned by the short EPG API (which are upcoming) are always included,
    // regardless of how narrow the original `from`/`to` window was.
    let wider_from = (Utc::now() - chrono::Duration::hours(1))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let wider_to = (Utc::now() + chrono::Duration::hours(24))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let re_queried = {
        let conn = db_conn.read().map_err(|e| e.to_string())?;
        crate::db::epg::query_for_channel(&conn, profile_id, &channel_id, &wider_from, &wider_to)
            .map_err(|e| e.to_string())?
    };

    Ok(re_queried)
}

#[derive(serde::Serialize)]
pub struct GuideChannel {
    pub stream_id: i64,
    pub name: Option<String>,
    pub stream_icon: Option<String>,
    pub epg_channel_id: Option<String>,
    pub tv_archive: i64,
    pub tv_archive_duration: i64,
    pub profile_id: i64,
    pub countries: Option<String>,
    pub epg_entries: Vec<crate::db::epg::EpgEntry>,
}

#[tauri::command]
pub async fn get_epg_guide(
    state: State<'_, DbConn>,
    profile_id: Option<i64>,
    from: String,
    to: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<GuideChannel>, String> {
    let conn = state.read().map_err(|e| e.to_string())?;

    let limit_val = limit.unwrap_or(50);
    let offset_val = offset.unwrap_or(0);

    // 1. Fetch channels that have active EPG listings in the timeline window (paginated & index-backed)
    let channel_rows = if let Some(p_id) = profile_id {
        let mut channel_stmt = conn.prepare_cached(
            "SELECT ls.stream_id, ls.name, ls.stream_icon, ls.epg_channel_id, ls.tv_archive, ls.tv_archive_duration, ls.profile_id, ls.countries
             FROM live_streams ls
             INNER JOIN epg_entries ee 
               ON ee.profile_id = ls.profile_id 
              AND ee.channel_id = ls.epg_channel_id
             WHERE ls.profile_id = ?1
               AND ls.epg_channel_id IS NOT NULL 
               AND ls.epg_channel_id != ''
               AND ee.start < ?2 
               AND ee.stop > ?3
             GROUP BY ls.profile_id, ls.epg_channel_id
             ORDER BY ls.name ASC
             LIMIT ?4 OFFSET ?5"
        ).map_err(|e| e.to_string())?;
        
        let rows = channel_stmt.query_map(rusqlite::params![p_id, &to, &from, limit_val, offset_val], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        }).map_err(|e| e.to_string())?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r.map_err(|e| e.to_string())?);
        }

        res
    } else {
        let mut channel_stmt = conn.prepare_cached(
            "SELECT ls.stream_id, ls.name, ls.stream_icon, ls.epg_channel_id, ls.tv_archive, ls.tv_archive_duration, ls.profile_id, ls.countries
             FROM live_streams ls
             INNER JOIN epg_entries ee 
               ON ee.profile_id = ls.profile_id 
              AND ee.channel_id = ls.epg_channel_id
             WHERE ls.epg_channel_id IS NOT NULL 
               AND ls.epg_channel_id != ''
               AND ee.start < ?1 
               AND ee.stop > ?2
             GROUP BY ls.profile_id, ls.epg_channel_id
             ORDER BY ls.name ASC
             LIMIT ?3 OFFSET ?4"
        ).map_err(|e| e.to_string())?;

        let rows = channel_stmt.query_map(rusqlite::params![&to, &from, limit_val, offset_val], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        }).map_err(|e| e.to_string())?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r.map_err(|e| e.to_string())?);
        }

        res
    };

    let mut channels = Vec::new();
    let mut channel_ids = Vec::new();
    for row in channel_rows {
        let (stream_id, name, stream_icon, epg_channel_id, tv_archive, tv_archive_duration, p_id, countries) = row;
        if let Some(ref ch_id) = epg_channel_id {
            channel_ids.push(ch_id.clone());
        }
        channels.push(GuideChannel {
            stream_id,
            name,
            stream_icon,
            epg_channel_id,
            tv_archive,
            tv_archive_duration,
            profile_id: p_id,
            countries,
            epg_entries: Vec::new(),
        });
    }

    if channel_ids.is_empty() {
        return Ok(channels);
    }

    // 2. Fetch EPG entries in the timeframe matching ONLY the paginated channel IDs
    let channel_ids_json = serde_json::to_string(&channel_ids).map_err(|e| e.to_string())?;
    let epg_rows = if let Some(p_id) = profile_id {
        let mut epg_stmt = conn.prepare_cached(
            "SELECT profile_id, channel_id, start, stop, title, description, tz_offset 
             FROM epg_entries 
             WHERE profile_id = ?1
               AND start < ?2 
               AND stop > ?3
               AND channel_id IN (SELECT value FROM json_each(?4))
             ORDER BY start ASC"
        ).map_err(|e| e.to_string())?;
        
        let rows = epg_stmt.query_map(rusqlite::params![p_id, &to, &from, channel_ids_json], |row| {
            Ok(EpgEntry {
                profile_id: row.get(0)?,
                channel_id: row.get(1)?,
                start: row.get(2)?,
                stop: row.get(3)?,
                title: row.get(4)?,
                description: row.get(5)?,
                tz_offset: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r.map_err(|e| e.to_string())?);
        }
        res
    } else {
        let mut epg_stmt = conn.prepare_cached(
            "SELECT profile_id, channel_id, start, stop, title, description, tz_offset 
             FROM epg_entries 
             WHERE start < ?1 
               AND stop > ?2
               AND channel_id IN (SELECT value FROM json_each(?3))
             ORDER BY start ASC"
        ).map_err(|e| e.to_string())?;
        
        let rows = epg_stmt.query_map(rusqlite::params![&to, &from, channel_ids_json], |row| {
            Ok(EpgEntry {
                profile_id: row.get(0)?,
                channel_id: row.get(1)?,
                start: row.get(2)?,
                stop: row.get(3)?,
                title: row.get(4)?,
                description: row.get(5)?,
                tz_offset: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r.map_err(|e| e.to_string())?);
        }
        res
    };

    let mut entries_by_channel: HashMap<String, Vec<EpgEntry>> = HashMap::new();
    for entry in epg_rows {
        entries_by_channel.entry(entry.channel_id.clone()).or_default().push(entry);
    }

    for ch in &mut channels {
        if let Some(ref ch_id) = ch.epg_channel_id {
            ch.epg_entries = entries_by_channel.remove(ch_id).unwrap_or_default();
        }
    }

    Ok(channels)
}
