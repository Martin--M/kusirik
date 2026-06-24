use anyhow::Result;
use rusqlite::Connection;
use crate::api::vod::{VodCategoryApi, VodStreamApi};

pub fn upsert_categories(conn: &mut Connection, profile_id: i64, categories: &[VodCategoryApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO vod_categories (profile_id, category_id, category_name)
             VALUES (?1, ?2, ?3)",
        )?;

        for cat in categories {
            let cat_id = cat.category_id.as_deref().unwrap_or("0");
            stmt.execute(rusqlite::params![profile_id, cat_id, cat.category_name])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn upsert_streams(conn: &mut Connection, profile_id: i64, streams: &[VodStreamApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO vod_streams (
                profile_id, stream_id, name, stream_icon, category_id,
                rating, container_extension, added
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        for stream in streams {
            let cat_id = stream.category_id.as_deref().unwrap_or("0");
            stmt.execute(rusqlite::params![
                profile_id,
                stream.stream_id,
                stream.name,
                stream.stream_icon,
                cat_id,
                stream.rating,
                stream.container_extension,
                stream.added
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn upsert_vod_info(
    conn: &Connection,
    profile_id: i64,
    stream_id: i64,
    info_json: &str,
    fetched_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO vod_info (profile_id, stream_id, info_json, fetched_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![profile_id, stream_id, info_json, fetched_at],
    )?;
    Ok(())
}

pub fn get_vod_info(conn: &Connection, profile_id: i64, stream_id: i64) -> Result<Option<(String, String)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT info_json, fetched_at FROM vod_info WHERE profile_id = ?1 AND stream_id = ?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![profile_id, stream_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?)))
    } else {
        Ok(None)
    }
}
