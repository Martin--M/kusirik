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
    pub created_at: i64,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();
        conn
    }

    #[test]
    fn test_profile_crud_lifecycle() {
        let conn = setup_conn();
        assert!(get_all(&conn).unwrap().is_empty());
        assert!(get(&conn, 1).unwrap().is_none());

        let p1 = Profile {
            id: 0,
            name: "Provider A".to_string(),
            server_url: "http://a.example.com".to_string(),
            username: "user_a".to_string(),
            password: "secret".to_string(),
            epg_mode: "xmltv".to_string(),
            created_at: 1700000001,
            profile_type: "xtream".to_string(),
        };
        let id1 = insert(&conn, &p1).unwrap();
        assert!(id1 > 0);

        let p2 = Profile {
            id: 0,
            name: "Public IPTV".to_string(),
            server_url: "https://iptv-org.github.io/api".to_string(),
            username: "public".to_string(),
            password: String::new(),
            epg_mode: "xmltv".to_string(),
            created_at: 1700000002,
            profile_type: "public_iptv".to_string(),
        };
        let id2 = insert(&conn, &p2).unwrap();
        assert!(id2 > id1);

        // Explicit id updates existing profile
        let updated = Profile {
            id: id1,
            name: "Provider A Updated".to_string(),
            ..p1
        };
        assert_eq!(insert(&conn, &updated).unwrap(), id1);

        let fetched = get(&conn, id1).unwrap().unwrap();
        assert_eq!(fetched.name, "Provider A Updated");
        assert_eq!(fetched.password, "secret");

        let all = get_all(&conn).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, id1);
        assert_eq!(all[1].id, id2);

        delete(&conn, id1).unwrap();
        assert!(get(&conn, id1).unwrap().is_none());
        assert_eq!(get_all(&conn).unwrap().len(), 1);
    }
}
