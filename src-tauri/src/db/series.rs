// Stub — implemented in P5
use anyhow::Result;
use rusqlite::Connection;

pub fn upsert_categories(_conn: &Connection, _profile_id: i64, _categories: &[serde_json::Value]) -> Result<()> {
    Ok(())
}

pub fn upsert_series(_conn: &Connection, _profile_id: i64, _series: &[serde_json::Value]) -> Result<()> {
    Ok(())
}
