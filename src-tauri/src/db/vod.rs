// Stub — implemented in P4
use anyhow::Result;
use rusqlite::Connection;

pub fn upsert_categories(_conn: &Connection, _profile_id: i64, _categories: &[serde_json::Value]) -> Result<()> {
    Ok(())
}

pub fn upsert_streams(_conn: &Connection, _profile_id: i64, _streams: &[serde_json::Value]) -> Result<()> {
    Ok(())
}
