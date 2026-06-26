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
    state: State<'_, DbConn>,
    client: State<'_, crate::api::XtreamClient>,
    profile_id: i64,
    channel_id: String,
    from: String,
    to: String,
) -> Result<Vec<EpgEntry>, String> {
    // 1. Query the local database first
    let db_conn = state;
    let list = {
        let conn = db_conn.0.lock().map_err(|e| e.to_string())?;
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
        let conn = db_conn.0.lock().map_err(|e| e.to_string())?;
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

    // Fetch EPG listings on demand
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
        });
    }

    if !epg_entries.is_empty() {
        let mut conn = db_conn.0.lock().map_err(|e| e.to_string())?;
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
        let conn = db_conn.0.lock().map_err(|e| e.to_string())?;
        crate::db::epg::query_for_channel(&conn, profile_id, &channel_id, &wider_from, &wider_to)
            .map_err(|e| e.to_string())?
    };

    Ok(re_queried)
}
