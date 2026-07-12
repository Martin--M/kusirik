use anyhow::Result;
use rusqlite::Connection;
use crate::api::live::{LiveStreamApi, LiveStreamDto};
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
            "INSERT INTO live_streams (
                profile_id, stream_id, name, stream_icon, epg_channel_id,
                category_id, tv_archive, tv_archive_duration, added
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(profile_id, stream_id) DO UPDATE SET
                name = excluded.name,
                stream_icon = excluded.stream_icon,
                epg_channel_id = excluded.epg_channel_id,
                category_id = excluded.category_id,
                tv_archive = excluded.tv_archive,
                tv_archive_duration = excluded.tv_archive_duration,
                added = excluded.added",
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

pub fn query_categories(conn: &Connection, profile_id: Option<i64>) -> Result<Vec<CategoryApi>> {
    crate::db::common::query_categories_generic(conn, "live_categories", profile_id)
}

pub fn query_streams(
    conn: &Connection,
    profile_id: Option<i64>,
    category_id: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<Vec<LiveStreamDto>> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let (db_category, is_uncategorized) = match category_id {
        None | Some("all") => (None, false),
        Some("0") | Some("") | Some("uncategorized") => (None, true),
        Some(cat) => (Some(cat), false),
    };

    let sql = "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, max(is_favorite) as is_favorite, profile_id, countries,
                      (SELECT title FROM epg_entries
                       WHERE epg_entries.profile_id = live_streams.profile_id
                         AND epg_entries.channel_id = live_streams.epg_channel_id
                         AND epg_entries.start <= ?6 AND epg_entries.stop > ?6 LIMIT 1) AS current_title
               FROM live_streams
               WHERE (?1 IS NULL OR profile_id = ?1)
                 AND (
                   (?2 IS NULL AND ?3 = 0) OR
                   (?3 = 1 AND (category_id = '0' OR category_id = '' OR category_id IS NULL)) OR
                   (category_id = ?2)
                 )
               GROUP BY profile_id, name
               ORDER BY name ASC
               LIMIT ?4 OFFSET ?5";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, db_category, is_uncategorized, limit, offset, &now], |row| {
        Ok(LiveStreamDto {
            stream: LiveStreamApi {
                stream_id: row.get(0)?,
                name: row.get(1)?,
                stream_icon: row.get(2)?,
                epg_channel_id: row.get(3)?,
                category_id: row.get(4)?,
                tv_archive: row.get(5)?,
                tv_archive_duration: row.get(6)?,
                added: row.get(7)?,
                is_favorite: row.get(8)?,
                profile_id: Some(row.get(9)?),
                countries: row.get(10)?,
                url: None,
            },
            current_title: row.get(11)?,
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
) -> Result<Vec<LiveStreamDto>> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let sql = "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, is_favorite, profile_id, countries,
                      (SELECT title FROM epg_entries
                       WHERE epg_entries.profile_id = live_streams.profile_id
                         AND epg_entries.channel_id = live_streams.epg_channel_id
                         AND epg_entries.start <= ?4 AND epg_entries.stop > ?4 LIMIT 1) AS current_title
               FROM live_streams
               WHERE (?1 IS NULL OR profile_id = ?1) AND (
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
               LIMIT ?3";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, query, limit, &now], |row| {
        Ok(LiveStreamDto {
            stream: LiveStreamApi {
                stream_id: row.get(0)?,
                name: row.get(1)?,
                stream_icon: row.get(2)?,
                epg_channel_id: row.get(3)?,
                category_id: row.get(4)?,
                tv_archive: row.get(5)?,
                tv_archive_duration: row.get(6)?,
                added: row.get(7)?,
                is_favorite: row.get(8)?,
                profile_id: Some(row.get(9)?),
                countries: row.get(10)?,
                url: None,
            },
            current_title: row.get(11)?,
        })
    })?;

    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

pub fn query_mirrors(
    conn: &Connection,
    profile_id: i64,
    name: &str,
) -> Result<Vec<LiveStreamApi>> {
    let mut stmt = conn.prepare_cached(
        "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, is_favorite, profile_id, url, countries
         FROM live_streams
         WHERE profile_id = ?1 AND name = ?2
         ORDER BY stream_id ASC"
    )?;

    let rows = stmt.query_map(rusqlite::params![profile_id, name], |row| {
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
            profile_id: Some(row.get(9)?),
            url: row.get(10)?,
            countries: row.get(11)?,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

