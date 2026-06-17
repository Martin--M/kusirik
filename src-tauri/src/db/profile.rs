use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub epg_mode: String,
    pub created_at: String,
}

pub fn insert(conn: &Connection, profile: &Profile) -> Result<()> {
    conn.execute(
        "INSERT INTO profiles (id, name, server_url, username, epg_mode, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            profile.id,
            profile.name,
            profile.server_url,
            profile.username,
            profile.epg_mode,
            profile.created_at
        ],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<Profile>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, server_url, username, epg_mode, created_at
         FROM profiles WHERE id = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Profile {
            id: row.get(0)?,
            name: row.get(1)?,
            server_url: row.get(2)?,
            username: row.get(3)?,
            epg_mode: row.get(4)?,
            created_at: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM profiles WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
