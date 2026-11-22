use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpgEntry {
    pub profile_id: i64,
    pub channel_id: String,
    pub start: i64, // Unix Epoch Seconds
    pub stop: i64,  // Unix Epoch Seconds
    pub title: Option<String>,
    pub description: Option<String>,
    pub tz_offset: Option<i32>, // Offset in seconds (e.g. 7200 for UTC+2)
}

/// Bulk insert EPG entries using INSERT OR IGNORE to handle daily re-syncs
/// without accumulating duplicates (schema has UNIQUE on profile_id, channel_id, start).
pub fn bulk_insert(conn: &mut Connection, entries: &[EpgEntry]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR IGNORE INTO epg_entries (profile_id, channel_id, start, stop, title, description, tz_offset)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )?;

        for entry in entries {
            stmt.execute(params![
                entry.profile_id,
                entry.channel_id,
                entry.start,
                entry.stop,
                entry.title,
                entry.description,
                entry.tz_offset.unwrap_or(0),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Queries EPG entries for a specific channel overlapping the given time window.
/// A program overlaps if its start time is before the window's end time,
/// and its stop time is after the window's start time.
pub fn query_for_channel(
    conn: &Connection,
    profile_id: i64,
    channel_id: &str,
    from: i64,
    to: i64,
) -> Result<Vec<EpgEntry>> {
    let mut stmt = conn.prepare_cached(
        "SELECT profile_id, channel_id, start, stop, title, description, tz_offset 
         FROM epg_entries 
         WHERE profile_id = ?1 AND channel_id = ?2 AND start < ?3 AND stop > ?4
         ORDER BY start ASC"
    )?;

    let rows = stmt.query_map(params![profile_id, channel_id, to, from], |row| {
        Ok(EpgEntry {
            profile_id: row.get(0)?,
            channel_id: row.get(1)?,
            start: row.get(2)?,
            stop: row.get(3)?,
            title: row.get(4)?,
            description: row.get(5)?,
            tz_offset: row.get(6)?,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

/// Time window (in seconds) used to bound active EPG title searches.
/// Programs must start within `[now - 6h, now]` and end within `(now, now + 6h]`
/// so SQLite performs a tight indexed B-tree range scan on `idx_epg_time_window`.
pub const ACTIVE_EPG_SEARCH_WINDOW_SECS: i64 = 6 * 3600;

/// Escapes SQL LIKE wildcards (`%`, `_`, `\`) and wraps the trimmed query in `%...%`.
pub fn build_like_contains_pattern(query: &str) -> Option<String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut escaped = String::with_capacity(trimmed.len() + 2);
    escaped.push('%');
    for ch in trimmed.chars() {
        if matches!(ch, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped.push('%');
    Some(escaped)
}

/// Returns distinct `(profile_id, channel_id)` pairs whose currently airing EPG program
/// title contains `query` (case-insensitive), bounded to `[now - 6h, now]` for `start`
/// and `(now, now + 6h]` for `stop`.
pub fn search_active_channel_ids(
    conn: &Connection,
    profile_id: Option<i64>,
    query: &str,
    now: i64,
) -> Result<Vec<(i64, String)>> {
    let Some(pattern) = build_like_contains_pattern(query) else {
        return Ok(Vec::new());
    };

    let min_start = now - ACTIVE_EPG_SEARCH_WINDOW_SECS;
    let max_stop = now + ACTIVE_EPG_SEARCH_WINDOW_SECS;

    let mut stmt = conn.prepare_cached(
        "SELECT DISTINCT profile_id, channel_id
         FROM epg_entries
         WHERE (?1 IS NULL OR profile_id = ?1)
           AND start >= ?2
           AND start <= ?3
           AND stop > ?3
           AND stop <= ?4
           AND title IS NOT NULL
           AND title LIKE ?5 ESCAPE '\\'",
    )?;

    let rows = stmt.query_map(
        params![profile_id, min_start, now, max_stop, pattern],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
    )?;

    let mut matches = Vec::new();
    for row in rows {
        matches.push(row?);
    }
    Ok(matches)
}

/// Deletes EPG entries that ended before the given timestamp (Unix Epoch Seconds).
pub fn cleanup_old_entries(conn: &Connection, profile_id: i64, before_timestamp: i64) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM epg_entries WHERE profile_id = ?1 AND stop < ?2",
        params![profile_id, before_timestamp],
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();
        conn
    }

    fn entry(profile_id: i64, channel_id: &str, start: i64, stop: i64, title: &str) -> EpgEntry {
        EpgEntry {
            profile_id,
            channel_id: channel_id.to_string(),
            start,
            stop,
            title: Some(title.to_string()),
            description: Some(format!("Desc for {}", title)),
            tz_offset: Some(-18000),
        }
    }

    #[test]
    fn test_bulk_insert_deduplicates_and_handles_empty() {
        let mut conn = setup_conn();
        bulk_insert(&mut conn, &[]).unwrap();

        let batch = vec![
            entry(1, "ch1", 1000, 2000, "Original Show"),
            // Duplicate (profile_id, channel_id, start) should be ignored
            entry(1, "ch1", 1000, 2000, "Duplicate Show"),
            entry(1, "ch1", 2000, 3000, "Next Show"),
        ];
        bulk_insert(&mut conn, &batch).unwrap();

        let rows = query_for_channel(&conn, 1, "ch1", 0, 5000).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title.as_deref(), Some("Original Show"));
        assert_eq!(rows[0].tz_offset, Some(-18000));
        assert_eq!(rows[1].title.as_deref(), Some("Next Show"));
    }

    #[test]
    fn test_query_for_channel_window_boundaries_and_isolation() {
        let mut conn = setup_conn();
        let batch = vec![
            // Ends exactly at window start (1000) -> excluded (stop > from)
            entry(1, "ch1", 500, 1000, "Ended At Boundary"),
            // Overlaps start boundary
            entry(1, "ch1", 800, 1200, "Overlaps Start"),
            // Completely inside window [1000, 2000]
            entry(1, "ch1", 1200, 1800, "Inside Window"),
            // Overlaps end boundary
            entry(1, "ch1", 1800, 2500, "Overlaps End"),
            // Starts exactly at window end (2000) -> excluded (start < to)
            entry(1, "ch1", 2000, 2600, "Starts At Boundary"),
            // Different profile and different channel
            entry(2, "ch1", 1100, 1900, "Other Profile"),
            entry(1, "ch2", 1100, 1900, "Other Channel"),
        ];
        bulk_insert(&mut conn, &batch).unwrap();

        let rows = query_for_channel(&conn, 1, "ch1", 1000, 2000).unwrap();
        let titles: Vec<_> = rows.iter().filter_map(|e| e.title.as_deref()).collect();
        assert_eq!(titles, vec!["Overlaps Start", "Inside Window", "Overlaps End"]);
    }

    #[test]
    fn test_cleanup_old_entries_respects_profile_and_boundary() {
        let mut conn = setup_conn();
        bulk_insert(
            &mut conn,
            &[
                entry(1, "ch1", 100, 500, "Old"),
                entry(1, "ch1", 500, 1000, "At Cutoff"),
                entry(1, "ch1", 1000, 1500, "Active"),
                entry(2, "ch1", 100, 500, "Other Profile Old"),
            ],
        )
        .unwrap();

        let deleted = cleanup_old_entries(&conn, 1, 1000).unwrap();
        assert_eq!(deleted, 1);

        let remaining_p1 = query_for_channel(&conn, 1, "ch1", 0, 5000).unwrap();
        assert_eq!(remaining_p1.len(), 2);
        assert_eq!(remaining_p1[0].title.as_deref(), Some("At Cutoff"));

        // Profile 2 entries must remain untouched
        let remaining_p2 = query_for_channel(&conn, 2, "ch1", 0, 5000).unwrap();
        assert_eq!(remaining_p2.len(), 1);
    }

    #[test]
    fn test_search_active_channel_ids_respects_6h_bounds_and_like_escaping() {
        let mut conn = setup_conn();
        let now = 1_795_269_600i64;
        let hour = 3600i64;

        bulk_insert(
            &mut conn,
            &[
                // Active within [now - 6h, now] and (now, now + 6h] -> MATCH
                entry(1, "ch_match", now - hour, now + hour, "Evening News Live"),
                // Exact boundary: starts at now - 6h, ends at now + 6h -> MATCH
                entry(1, "ch_boundary", now - 6 * hour, now + 6 * hour, "News Marathon"),
                // Started > 6h ago (e.g. 24h loop) -> EXCLUDED
                entry(1, "ch_too_old_start", now - 6 * hour - 1, now + hour, "24h News Loop"),
                // Ends > 6h from now -> EXCLUDED
                entry(1, "ch_too_long_stop", now - hour, now + 6 * hour + 1, "Long News Special"),
                // Past program -> EXCLUDED
                entry(1, "ch_past", now - 2 * hour, now, "Past News"),
                // Future program -> EXCLUDED
                entry(1, "ch_future", now + 60, now + 2 * hour, "Upcoming News"),
                // Profile 2 match -> included only when profile_id is None or Some(2)
                entry(2, "ch_p2", now - hour, now + hour, "World News"),
                // Literal % and _ characters to verify LIKE escaping
                entry(1, "ch_percent", now - hour, now + hour, "100% Live_Show"),
            ],
        )
        .unwrap();

        let p1_hits = search_active_channel_ids(&conn, Some(1), "news", now).unwrap();
        let mut p1_channels: Vec<_> = p1_hits.iter().map(|(_, c)| c.as_str()).collect();
        p1_channels.sort();
        assert_eq!(p1_channels, vec!["ch_boundary", "ch_match"]);

        let all_hits = search_active_channel_ids(&conn, None, "news", now).unwrap();
        assert_eq!(all_hits.len(), 3);

        // Escaped wildcard search should only match literal "100%" and not every title
        let pct_hits = search_active_channel_ids(&conn, Some(1), "100%", now).unwrap();
        assert_eq!(pct_hits, vec![(1, "ch_percent".to_string())]);

        assert!(search_active_channel_ids(&conn, Some(1), "   ", now).unwrap().is_empty());
    }
}
