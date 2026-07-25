use anyhow::Result;
use rusqlite::Connection;

/// Retrieves a cached image from the database if it exists.
/// Returns Ok(Some((data, content_type))) if found, or Ok(None) if not.
pub fn get_cached_image(conn: &Connection, url: &str) -> Result<Option<(Vec<u8>, Option<String>)>> {
    let mut stmt = conn.prepare(
        "SELECT data, content_type FROM image_cache WHERE url = ?1",
    )?;
    
    let mut rows = stmt.query_map(rusqlite::params![url], |row| {
        Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Option<String>>(1)?))
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

/// Caches an image's bytes and content type in the database.
pub fn insert_cached_image(
    conn: &Connection,
    url: &str,
    data: &[u8],
    content_type: Option<&str>,
) -> Result<()> {
    let fetched_at = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO image_cache (url, data, content_type, fetched_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![url, data, content_type, fetched_at],
    )?;
    Ok(())
}
