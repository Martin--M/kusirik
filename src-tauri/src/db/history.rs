use anyhow::{Context, Result};
use rusqlite::Connection;
use crate::api::live::{LiveStreamApi, LiveStreamDto};
use crate::api::vod::VodStreamApi;
use crate::api::series::SeriesApi;
use crate::commands::search::SearchResults;

pub fn record_playback(
    conn: &Connection,
    profile_id: i64,
    media_type: &str,
    stream_id: i64,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    conn.execute(
        "INSERT OR REPLACE INTO playback_history (profile_id, media_type, stream_id, played_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![profile_id, media_type, stream_id, &now],
    ).context("Failed to record playback history")?;
    Ok(())
}

pub fn delete_playback(
    conn: &Connection,
    profile_id: i64,
    media_type: &str,
    stream_id: i64,
) -> Result<()> {
    conn.execute(
        "DELETE FROM playback_history 
         WHERE profile_id = ?1 AND media_type = ?2 AND stream_id = ?3",
        rusqlite::params![profile_id, media_type, stream_id],
    ).context("Failed to delete playback history entry")?;
    Ok(())
}

pub fn clear_history(conn: &Connection, profile_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM playback_history WHERE profile_id = ?1",
        rusqlite::params![profile_id],
    ).context("Failed to clear playback history")?;
    Ok(())
}

pub fn query_history(conn: &Connection, profile_id: i64) -> Result<SearchResults> {
    // 1. Live streams (join with playback_history)
    let mut live_stmt = conn.prepare(
        "SELECT s.stream_id, s.name, s.stream_icon, s.epg_channel_id, s.category_id, s.tv_archive, s.tv_archive_duration, s.added, s.is_favorite, s.profile_id
         FROM playback_history h
         JOIN live_streams s ON h.profile_id = s.profile_id AND h.stream_id = s.stream_id
         WHERE h.profile_id = ?1 AND h.media_type = 'live'
         ORDER BY h.played_at DESC",
    )?;
    let live_rows = live_stmt.query_map(rusqlite::params![profile_id], |row| {
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
            },
            current_title: None,
        })
    })?;
    let mut live = Vec::new();
    for r in live_rows {
        live.push(r?);
    }

    // 2. Vod streams (join with playback_history)
    let mut vod_stmt = conn.prepare(
        "SELECT s.stream_id, s.name, s.stream_icon, s.category_id, s.rating, s.container_extension, s.added, s.is_favorite, s.profile_id
         FROM playback_history h
         JOIN vod_streams s ON h.profile_id = s.profile_id AND h.stream_id = s.stream_id
         WHERE h.profile_id = ?1 AND h.media_type = 'vod'
         ORDER BY h.played_at DESC",
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
            profile_id: Some(row.get(8)?),
        })
    })?;
    let mut vod = Vec::new();
    for r in vod_rows {
        vod.push(r?);
    }

    // 3. Series (join with playback_history)
    let mut series_stmt = conn.prepare(
        "SELECT s.series_id, s.name, s.cover, s.category_id, s.rating, s.plot, s.cast_, s.director, s.genre, s.release_date, s.last_modified, s.is_favorite, s.profile_id
         FROM playback_history h
         JOIN series s ON h.profile_id = s.profile_id AND h.stream_id = s.series_id
         WHERE h.profile_id = ?1 AND h.media_type = 'series'
         ORDER BY h.played_at DESC",
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
            profile_id: Some(row.get(12)?),
        })
    })?;
    let mut series = Vec::new();
    for r in series_rows {
        series.push(r?);
    }

    Ok(SearchResults { live, vod, series })
}
