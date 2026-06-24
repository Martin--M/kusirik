use anyhow::Result;
use rusqlite::Connection;
use crate::api::live::{LiveCategoryApi, LiveStreamApi};

pub fn upsert_categories(conn: &mut Connection, profile_id: i64, categories: &[LiveCategoryApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO live_categories (profile_id, category_id, category_name)
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

pub fn upsert_streams(conn: &mut Connection, profile_id: i64, streams: &[LiveStreamApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO live_streams (
                profile_id, stream_id, name, stream_icon, epg_channel_id,
                category_id, tv_archive, tv_archive_duration, added
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;

        for stream in streams {
            let cat_id = stream.category_id.as_deref().unwrap_or("0");
            stmt.execute(rusqlite::params![
                profile_id,
                stream.stream_id,
                stream.name,
                stream.stream_icon,
                stream.epg_channel_id,
                cat_id,
                stream.tv_archive.unwrap_or(0),
                stream.tv_archive_duration.unwrap_or(0),
                stream.added
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
