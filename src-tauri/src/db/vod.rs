use anyhow::Result;
use rusqlite::Connection;
use crate::api::vod::VodStreamApi;
use crate::api::common::CategoryApi;

pub fn upsert_categories(conn: &mut Connection, profile_id: i64, categories: &[CategoryApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO vod_categories (profile_id, category_id, category_name)
             VALUES (?1, ?2, ?3)",
        )?;

        for cat in categories {
            let cat_id = cat.category_id.as_deref().unwrap_or("0");
            stmt.execute(rusqlite::params![profile_id, cat_id, cat.category_name])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn upsert_streams(conn: &mut Connection, profile_id: i64, streams: &[VodStreamApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO vod_streams (
                profile_id, stream_id, name, stream_icon, category_id,
                rating, container_extension, added, release_date
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(profile_id, stream_id) DO UPDATE SET
                name = excluded.name,
                stream_icon = excluded.stream_icon,
                category_id = excluded.category_id,
                rating = excluded.rating,
                container_extension = excluded.container_extension,
                added = excluded.added,
                release_date = CASE
                    WHEN ?10 = 1 THEN excluded.release_date
                    WHEN vod_streams.release_date IS NOT NULL
                         AND strftime('%m-%d', vod_streams.release_date, 'unixepoch') != '01-01'
                         THEN vod_streams.release_date
                    ELSE COALESCE(excluded.release_date, vod_streams.release_date)
                END",
        )?;

        for stream in streams {
            let cat_id = stream.category_id.as_deref().unwrap_or("0");

            // Resolve release date into Unix Epoch timestamp (seconds):
            // 1. If stream.release_date is already provided by the API, mark as precise (is_precise = 1)
            // 2. Otherwise try extracting "(YYYY)" from movie title (Jan 1st fallback, is_precise = 0)
            let (rel_timestamp, is_precise): (Option<i64>, i32) = match stream.release_date {
                Some(ts) if ts > 0 => (Some(ts), 1),
                _ => {
                    let fallback = stream.name.as_deref().and_then(|name| {
                        let bytes = name.as_bytes();
                        if bytes.len() >= 6 {
                            for i in 0..=bytes.len() - 6 {
                                if bytes[i] == b'('
                                    && bytes[i + 5] == b')'
                                    && bytes[i + 1..i + 5].iter().all(|b| b.is_ascii_digit())
                                {
                                    if let Ok(year) = name[i + 1..i + 5].parse::<i32>() {
                                        if let Some(ndt) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) {
                                            if let Some(dt) = ndt.and_hms_opt(0, 0, 0) {
                                                return Some(dt.and_utc().timestamp());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None
                    });
                    (fallback, 0)
                }
            };

            stmt.execute(rusqlite::params![
                profile_id,
                stream.stream_id,
                stream.name,
                stream.stream_icon,
                cat_id,
                stream.rating,
                stream.container_extension,
                stream.added,
                rel_timestamp,
                is_precise
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Extracts a release date Unix timestamp (seconds) from a `get_vod_info` JSON payload.
pub fn extract_release_date_from_vod_info(info_val: &serde_json::Value) -> Option<i64> {
    let info_obj = info_val.get("info").unwrap_or(info_val);
    for key in ["releasedate", "release_date", "releaseDate"] {
        if let Some(val) = info_obj.get(key) {
            let parsed = match val {
                serde_json::Value::String(s) => crate::api::client::parse_date_or_timestamp(s),
                serde_json::Value::Number(n) => n.as_i64(),
                _ => None,
            };
            if let Some(ts) = parsed {
                if ts > 0 {
                    return Some(ts);
                }
            }
        }
    }
    None
}

/// Refines a movie's cached `release_date` in `vod_streams` (e.g. from `get_vod_info`).
pub fn update_release_date(
    conn: &Connection,
    profile_id: i64,
    stream_id: i64,
    release_date: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE vod_streams SET release_date = ?1 WHERE profile_id = ?2 AND stream_id = ?3",
        rusqlite::params![release_date, profile_id, stream_id],
    )?;
    Ok(())
}

pub fn upsert_vod_info(
    conn: &Connection,
    profile_id: i64,
    stream_id: i64,
    info_json: &str,
    fetched_at: i64,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO vod_info (profile_id, stream_id, info_json, fetched_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![profile_id, stream_id, info_json, fetched_at],
    )?;
    Ok(())
}

pub fn get_vod_info(conn: &Connection, profile_id: i64, stream_id: i64) -> Result<Option<(String, i64)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT info_json, fetched_at FROM vod_info WHERE profile_id = ?1 AND stream_id = ?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![profile_id, stream_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?)))
    } else {
        Ok(None)
    }
}

pub fn query_categories(conn: &Connection, profile_id: Option<i64>) -> Result<Vec<CategoryApi>> {
    crate::db::common::query_categories_generic(conn, "vod_categories", profile_id)
}

pub fn query_streams(
    conn: &Connection,
    profile_id: Option<i64>,
    category_id: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<Vec<VodStreamApi>> {
    let (db_category, is_uncategorized) = match category_id {
        None | Some("all") => (None, false),
        Some("0") | Some("") | Some("uncategorized") => (None, true),
        Some(cat) => (Some(cat), false),
    };

    let sql = "SELECT stream_id, name, stream_icon, category_id, rating, container_extension, added, is_favorite, profile_id, release_date
               FROM vod_streams
               WHERE (?1 IS NULL OR profile_id = ?1)
                 AND (
                   (?2 IS NULL AND ?3 = 0) OR
                   (?3 = 1 AND (category_id = '0' OR category_id = '' OR category_id IS NULL)) OR
                   (category_id = ?2)
                 )
               ORDER BY name ASC
               LIMIT ?4 OFFSET ?5";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, db_category, is_uncategorized, limit, offset], |row| {
        Ok(VodStreamApi {
            stream_id: row.get(0)?,
            name: row.get(1)?,
            stream_icon: row.get(2)?,
            category_id: row.get(3)?,
            rating: row.get(4)?,
            container_extension: row.get(5)?,
            added: row.get(6)?,
            is_favorite: row.get(7)?,
            profile_id: Some(row.get(8)?),
            release_date: row.get(9)?,
        })
    })?;

    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

pub fn search_streams(
    conn: &Connection,
    profile_id: Option<i64>,
    query: &str,
    limit: u32,
) -> Result<Vec<VodStreamApi>> {
    let sanitized = crate::commands::search::sanitize_fts5_query(query);
    if sanitized.is_empty() {
        return Ok(Vec::new());
    }

    let sql = "SELECT s.stream_id, s.name, s.stream_icon, s.category_id, s.rating, s.container_extension, s.added, s.is_favorite, s.profile_id, s.release_date
               FROM vod_streams s
               JOIN fts_vod fts ON s.stream_id = fts.rowid
               WHERE (?1 IS NULL OR s.profile_id = ?1) AND fts_vod MATCH ?2
               ORDER BY bm25(fts_vod) ASC
               LIMIT ?3";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, sanitized, limit], |row| {
        Ok(VodStreamApi {
            stream_id: row.get(0)?,
            name: row.get(1)?,
            stream_icon: row.get(2)?,
            category_id: row.get(3)?,
            rating: row.get(4)?,
            container_extension: row.get(5)?,
            added: row.get(6)?,
            is_favorite: row.get(7)?,
            profile_id: Some(row.get(8)?),
            release_date: row.get(9)?,
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vod_stream_api_deserializes_string_release_dates() {
        let json = r#"{
            "stream_id": 42,
            "name": "Example Movie (2026)",
            "releasedate": "2026-11-20",
            "added": "1795179001"
        }"#;
        let parsed: VodStreamApi = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.release_date, Some(1795132800));
        assert_eq!(parsed.added, Some(1795179001));
    }

    #[test]
    fn test_refine_release_date_from_vod_info_preserved_across_resync() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        // 1. Initial sync without API release_date falls back to Jan 1 of title year
        let initial = vec![VodStreamApi {
            stream_id: 100,
            name: Some("Steps (2026)".to_string()),
            stream_icon: None,
            category_id: Some("1".to_string()),
            rating: Some("7.5".to_string()),
            container_extension: Some("mp4".to_string()),
            added: Some(1795179001),
            release_date: None,
            is_favorite: Some(0),
            profile_id: Some(1),
        }];
        upsert_streams(&mut conn, 1, &initial).unwrap();

        let rows = query_streams(&conn, Some(1), None, 0, 10).unwrap();
        assert_eq!(rows[0].release_date, Some(1767225600)); // 2026-01-01

        // 2. Refine release_date from vod_info JSON payload
        let info_json = serde_json::json!({
            "info": {
                "releasedate": "2026-11-20"
            }
        });
        let refined_ts = extract_release_date_from_vod_info(&info_json).unwrap();
        assert_eq!(refined_ts, 1795132800); // 2026-11-20
        update_release_date(&conn, 1, 100, refined_ts).unwrap();

        // 3. Re-sync without API release_date must preserve the refined Nov 20 date
        upsert_streams(&mut conn, 1, &initial).unwrap();
        let rows_after_resync = query_streams(&conn, Some(1), None, 0, 10).unwrap();
        assert_eq!(rows_after_resync[0].release_date, Some(1795132800));

        // 4. Re-sync WITH an explicit API release_date overrides it
        let mut with_api_date = initial.clone();
        with_api_date[0].release_date = Some(1795219200); // 2026-11-21
        upsert_streams(&mut conn, 1, &with_api_date).unwrap();
        let rows_after_override = query_streams(&conn, Some(1), None, 0, 10).unwrap();
        assert_eq!(rows_after_override[0].release_date, Some(1795219200));
    }
}


