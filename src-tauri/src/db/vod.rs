use anyhow::Result;
use rusqlite::Connection;
use crate::api::vod::VodStreamApi;
use crate::api::common::CategoryApi;

pub fn upsert_categories(conn: &mut Connection, profile_id: i64, categories: &[CategoryApi]) -> Result<()> {
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

pub fn query_categories(conn: &Connection, profile_id: Option<i64>) -> Result<Vec<CategoryApi>> {
    crate::db::common::query_categories_generic(conn, "vod_categories", profile_id)
}

pub fn query_streams(
    conn: &Connection,
    profile_id: Option<i64>,
    category_id: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<Vec<VodStreamApi>> {
    let (db_category, is_uncategorized) = match category_id {
        None | Some("all") => (None, false),
        Some("0") | Some("") | Some("uncategorized") => (None, true),
        Some(cat) => (Some(cat), false),
    };

    let sql = "SELECT stream_id, name, stream_icon, category_id, rating, container_extension, added, is_favorite, profile_id
               FROM vod_streams
               WHERE (?1 IS NULL OR profile_id = ?1)
                 AND (
                   (?2 IS NULL AND ?3 = 0) OR
                   (?3 = 1 AND (category_id = '0' OR category_id = '' OR category_id IS NULL)) OR
                   (category_id = ?2)
                 )
               ORDER BY name ASC
               LIMIT ?4 OFFSET ?5";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, db_category, is_uncategorized, limit, offset], |row| {
        Ok(VodStreamApi {
            stream_id: row.get(0)?,
            name: row.get(1)?,
            stream_icon: row.get(2)?,
            category_id: row.get(3)?,
            rating: row.get(4)?,
            container_extension: row.get(5)?,
            added: row.get(6)?,
            is_favorite: row.get(7)?,
            profile_id: Some(row.get(8)?),
        })
    })?;

    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

pub fn search_streams(
    conn: &Connection,
    profile_id: Option<i64>,
    query: &str,
    limit: u32,
) -> Result<Vec<VodStreamApi>> {
    let sql = "SELECT stream_id, name, stream_icon, category_id, rating, container_extension, added, is_favorite, profile_id
               FROM vod_streams
               WHERE (?1 IS NULL OR profile_id = ?1) AND name LIKE ?2
               ORDER BY name ASC
               LIMIT ?3";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, query, limit], |row| {
        Ok(VodStreamApi {
            stream_id: row.get(0)?,
            name: row.get(1)?,
            stream_icon: row.get(2)?,
            category_id: row.get(3)?,
            rating: row.get(4)?,
            container_extension: row.get(5)?,
            added: row.get(6)?,
            is_favorite: row.get(7)?,
            profile_id: Some(row.get(8)?),
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}


