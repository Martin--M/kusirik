// Stub — implemented in P1
use anyhow::Result;
use rusqlite::Connection;

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(rusqlite::params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_get_set_and_overwrite() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        assert_eq!(get(&conn, "theme").unwrap(), None);

        set(&conn, "theme", "dark").unwrap();
        assert_eq!(get(&conn, "theme").unwrap().as_deref(), Some("dark"));

        set(&conn, "theme", "light").unwrap();
        assert_eq!(get(&conn, "theme").unwrap().as_deref(), Some("light"));

        set(&conn, "player_windows", "/usr/bin/vlc").unwrap();
        assert_eq!(
            get(&conn, "player_windows").unwrap().as_deref(),
            Some("/usr/bin/vlc")
        );
    }
}
