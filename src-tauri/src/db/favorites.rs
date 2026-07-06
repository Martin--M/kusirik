use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, OptionalExtension};
use crate::api::live::LiveStreamApi;
use crate::api::vod::VodStreamApi;
use crate::api::series::SeriesApi;
use crate::commands::search::SearchResults;

pub fn toggle_favorite(
    conn: &Connection,
    profile_id: i64,
    media_type: &str,
    stream_id: i64,
) -> Result<bool> {
    let (table, id_col) = match media_type {
        "live" => ("live_streams", "stream_id"),
        "vod" => ("vod_streams", "stream_id"),
        "series" => ("series", "series_id"),
        _ => return Err(anyhow!("Invalid media type: {}", media_type)),
    };

    let select_sql = format!(
        "SELECT is_favorite FROM {} WHERE profile_id = ?1 AND {} = ?2",
        table, id_col
    );

    let current_favorite: i32 = conn.query_row(
        &select_sql,
        rusqlite::params![profile_id, stream_id],
        |row| row.get(0),
    ).optional()?
    .unwrap_or(0);

    let new_favorite = if current_favorite == 1 { 0 } else { 1 };

    let update_sql = format!(
        "UPDATE {} SET is_favorite = ?1 WHERE profile_id = ?2 AND {} = ?3",
        table, id_col
    );

    conn.execute(
        &update_sql,
        rusqlite::params![new_favorite, profile_id, stream_id],
    ).context("Failed to update favorite status")?;

    Ok(new_favorite == 1)
}

pub fn query_favorites(conn: &Connection, profile_id: i64) -> Result<SearchResults> {
    // 1. Live streams
    let mut live_stmt = conn.prepare(
        "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, is_favorite
         FROM live_streams
         WHERE profile_id = ?1 AND is_favorite = 1
         ORDER BY name ASC",
    )?;
    let live_rows = live_stmt.query_map(rusqlite::params![profile_id], |row| {
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
        })
    })?;
    let mut live = Vec::new();
    for r in live_rows {
        live.push(r?);
    }

    // 2. Vod streams
    let mut vod_stmt = conn.prepare(
        "SELECT stream_id, name, stream_icon, category_id, rating, container_extension, added, is_favorite
         FROM vod_streams
         WHERE profile_id = ?1 AND is_favorite = 1
         ORDER BY name ASC",
    )?;
    let vod_rows = vod_stmt.query_map(rusqlite::params![profile_id], |row| {
        Ok(VodStreamApi {
            stream_id: row.get(0)?,
            name: row.get(1)?,
            stream_icon: row.get(2)?,
            category_id: row.get(3)?,
            rating: row.get(4)?,
            container_extension: row.get(5)?,
            added: row.get(6)?,
            is_favorite: row.get(7)?,
        })
    })?;
    let mut vod = Vec::new();
    for r in vod_rows {
        vod.push(r?);
    }

    // 3. Series
    let mut series_stmt = conn.prepare(
        "SELECT series_id, name, cover, category_id, rating, plot, cast_, director, genre, release_date, last_modified, is_favorite
         FROM series
         WHERE profile_id = ?1 AND is_favorite = 1
         ORDER BY name ASC",
    )?;
    let series_rows = series_stmt.query_map(rusqlite::params![profile_id], |row| {
        Ok(SeriesApi {
            series_id: row.get(0)?,
            name: row.get(1)?,
            cover: row.get(2)?,
            category_id: row.get(3)?,
            rating: row.get(4)?,
            plot: row.get(5)?,
            cast: row.get(6)?,
            director: row.get(7)?,
            genre: row.get(8)?,
            release_date: row.get(9)?,
            last_modified: row.get(10)?,
            is_favorite: row.get(11)?,
        })
    })?;
    let mut series = Vec::new();
    for r in series_rows {
        series.push(r?);
    }

    Ok(SearchResults { live, vod, series })
}
