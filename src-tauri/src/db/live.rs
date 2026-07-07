use anyhow::Result;
use rusqlite::Connection;
use crate::api::live::LiveStreamApi;
use crate::api::common::CategoryApi;

pub fn upsert_categories(conn: &mut Connection, profile_id: i64, categories: &[CategoryApi]) -> Result<()> {
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

pub fn query_categories(conn: &Connection, profile_id: i64) -> Result<Vec<CategoryApi>> {
    crate::db::common::query_categories_generic(conn, "live_categories", profile_id)
}

pub fn query_streams(
    conn: &Connection,
    profile_id: i64,
    category_id: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<Vec<LiveStreamApi>> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let streams = match category_id {
        None | Some("all") => {
            let mut stmt = conn.prepare(
                "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, is_favorite,
                        (SELECT title FROM epg_entries
                         WHERE epg_entries.profile_id = live_streams.profile_id
                           AND epg_entries.channel_id = live_streams.epg_channel_id
                           AND epg_entries.start <= ?4 AND epg_entries.stop > ?4 LIMIT 1) AS current_title
                 FROM live_streams
                 WHERE profile_id = ?1
                 ORDER BY name ASC
                 LIMIT ?2 OFFSET ?3",
            )?;
            let rows = stmt.query_map(rusqlite::params![profile_id, limit, offset, &now], |row| {
                Ok(LiveStreamApi {
                    stream_id: row.get(0)?,
                    name: row.get(1)?,
                    stream_icon: row.get(2)?,
                    epg_channel_id: row.get(3)?,
                    category_id: row.get(4)?,
                    tv_archive: row.get(5)?,
                    tv_archive_duration: row.get(6)?,
                    added: row.get(7)?,
                    is_favorite: row.get(8)?,
                    current_title: row.get(9)?,
                })
            })?;
            let mut res = Vec::new();
            for r in rows {
                res.push(r?);
            }
            res
        }
        Some("0") | Some("") | Some("uncategorized") => {
            let mut stmt = conn.prepare(
                "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, is_favorite,
                        (SELECT title FROM epg_entries
                         WHERE epg_entries.profile_id = live_streams.profile_id
                           AND epg_entries.channel_id = live_streams.epg_channel_id
                           AND epg_entries.start <= ?4 AND epg_entries.stop > ?4 LIMIT 1) AS current_title
                 FROM live_streams
                 WHERE profile_id = ?1 AND (category_id = '0' OR category_id = '' OR category_id IS NULL)
                 ORDER BY name ASC
                 LIMIT ?2 OFFSET ?3",
            )?;
            let rows = stmt.query_map(rusqlite::params![profile_id, limit, offset, &now], |row| {
                Ok(LiveStreamApi {
                    stream_id: row.get(0)?,
                    name: row.get(1)?,
                    stream_icon: row.get(2)?,
                    epg_channel_id: row.get(3)?,
                    category_id: row.get(4)?,
                    tv_archive: row.get(5)?,
                    tv_archive_duration: row.get(6)?,
                    added: row.get(7)?,
                    is_favorite: row.get(8)?,
                    current_title: row.get(9)?,
                })
            })?;
            let mut res = Vec::new();
            for r in rows {
                res.push(r?);
            }
            res
        }
        Some(cat) => {
            let mut stmt = conn.prepare(
                "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, is_favorite,
                        (SELECT title FROM epg_entries
                         WHERE epg_entries.profile_id = live_streams.profile_id
                           AND epg_entries.channel_id = live_streams.epg_channel_id
                           AND epg_entries.start <= ?5 AND epg_entries.stop > ?5 LIMIT 1) AS current_title
                 FROM live_streams
                 WHERE profile_id = ?1 AND category_id = ?2
                 ORDER BY name ASC
                 LIMIT ?3 OFFSET ?4",
            )?;
            let rows = stmt.query_map(rusqlite::params![profile_id, cat, limit, offset, &now], |row| {
                Ok(LiveStreamApi {
                    stream_id: row.get(0)?,
                    name: row.get(1)?,
                    stream_icon: row.get(2)?,
                    epg_channel_id: row.get(3)?,
                    category_id: row.get(4)?,
                    tv_archive: row.get(5)?,
                    tv_archive_duration: row.get(6)?,
                    added: row.get(7)?,
                    is_favorite: row.get(8)?,
                    current_title: row.get(9)?,
                })
            })?;
            let mut res = Vec::new();
            for r in rows {
                res.push(r?);
            }
            res
        }
    };
    Ok(streams)
}

pub fn search_streams(
    conn: &Connection,
    profile_id: i64,
    query: &str,
    limit: u32,
) -> Result<Vec<LiveStreamApi>> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mut stmt = conn.prepare_cached(
        "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, is_favorite,
                (SELECT title FROM epg_entries
                 WHERE epg_entries.profile_id = live_streams.profile_id
                   AND epg_entries.channel_id = live_streams.epg_channel_id
                   AND epg_entries.start <= ?4 AND epg_entries.stop > ?4 LIMIT 1) AS current_title
         FROM live_streams
         WHERE profile_id = ?1 AND (
             name LIKE ?2 OR
             EXISTS (
                 SELECT 1 FROM epg_entries
                 WHERE epg_entries.profile_id = live_streams.profile_id
                   AND epg_entries.channel_id = live_streams.epg_channel_id
                   AND epg_entries.start <= ?4 AND epg_entries.stop > ?4
                   AND epg_entries.title LIKE ?2
             )
         )
         ORDER BY name ASC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(rusqlite::params![profile_id, query, limit, &now], |row| {
        Ok(LiveStreamApi {
            stream_id: row.get(0)?,
            name: row.get(1)?,
            stream_icon: row.get(2)?,
            epg_channel_id: row.get(3)?,
            category_id: row.get(4)?,
            tv_archive: row.get(5)?,
            tv_archive_duration: row.get(6)?,
            added: row.get(7)?,
            is_favorite: row.get(8)?,
            current_title: row.get(9)?,
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

