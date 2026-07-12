use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpgEntry {
    pub profile_id: i64,
    pub channel_id: String,
    pub start: String, // UTC ISO 8601
    pub stop: String,  // UTC ISO 8601
    pub title: Option<String>,
    pub description: Option<String>,
    pub tz_offset: Option<String>,
}

/// Bulk insert EPG entries using INSERT OR REPLACE to handle daily re-syncs
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
                entry.tz_offset.as_deref().unwrap_or("+00:00"),
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
    from: &str,
    to: &str,
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

/// Deletes EPG entries that ended before the given timestamp (UTC ISO 8601).
pub fn cleanup_old_entries(conn: &Connection, profile_id: i64, before_utc_iso: &str) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM epg_entries WHERE profile_id = ?1 AND stop < ?2",
        params![profile_id, before_utc_iso],
    )?;
    Ok(count)
}
