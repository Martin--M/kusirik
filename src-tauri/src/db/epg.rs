// Stub — implemented in P3 (EPG sync phase)
use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EpgEntry {
    pub profile_id: i64,
    pub channel_id: String,
    pub start: String, // UTC ISO 8601
    pub stop: String,  // UTC ISO 8601
    pub title: Option<String>,
    pub description: Option<String>,
}

/// Bulk insert EPG entries using INSERT OR REPLACE to handle daily re-syncs
/// without accumulating duplicates (schema has UNIQUE on profile_id, channel_id, start).
pub fn bulk_insert(_conn: &Connection, _entries: &[EpgEntry]) -> Result<()> {
    // Implemented in P3
    Ok(())
}

pub fn query_for_channel(
    _conn: &Connection,
    _profile_id: i64,
    _channel_id: &str,
    _from: &str,
    _to: &str,
) -> Result<Vec<EpgEntry>> {
    // Implemented in P3
    Ok(vec![])
}
