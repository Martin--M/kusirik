use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub epg_mode: String,
    pub created_at: String,
    pub profile_type: String,
}

pub fn insert(conn: &Connection, profile: &Profile) -> Result<i64> {
    if profile.id > 0 {
        conn.execute(
            "INSERT OR REPLACE INTO profiles (id, name, server_url, username, password, epg_mode, created_at, profile_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                profile.id,
                &profile.name,
                &profile.server_url,
                &profile.username,
                &profile.password,
                &profile.epg_mode,
                &profile.created_at,
                &profile.profile_type
            ],
        )?;
        Ok(profile.id)
    } else {
        conn.execute(
            "INSERT INTO profiles (name, server_url, username, password, epg_mode, created_at, profile_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                &profile.name,
                &profile.server_url,
                &profile.username,
                &profile.password,
                &profile.epg_mode,
                &profile.created_at,
                &profile.profile_type
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

pub fn get_all(conn: &Connection) -> Result<Vec<Profile>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, server_url, username, password, epg_mode, created_at, profile_type
         FROM profiles ORDER BY id ASC",
    )?;
    let profile_iter = stmt.query_map([], |row| {
        Ok(Profile {
            id: row.get(0)?,
            name: row.get(1)?,
            server_url: row.get(2)?,
            username: row.get(3)?,
            password: row.get(4)?,
            epg_mode: row.get(5)?,
            created_at: row.get(6)?,
            profile_type: row.get(7)?,
        })
    })?;
    
    let mut list = Vec::new();
    for p in profile_iter {
        list.push(p?);
    }
    Ok(list)
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<Profile>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, server_url, username, password, epg_mode, created_at, profile_type
         FROM profiles WHERE id = ?1",
     )?;
     let mut rows = stmt.query(rusqlite::params![id])?;
     if let Some(row) = rows.next()? {
         Ok(Some(Profile {
             id: row.get(0)?,
             name: row.get(1)?,
             server_url: row.get(2)?,
             username: row.get(3)?,
             password: row.get(4)?,
             epg_mode: row.get(5)?,
             created_at: row.get(6)?,
             profile_type: row.get(7)?,
         }))
     } else {
         Ok(None)
     }
 }

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM profiles WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
