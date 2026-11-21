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
    last_episode_id: Option<i64>,
) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO playback_history (profile_id, media_type, stream_id, last_episode_id, played_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![profile_id, media_type, stream_id, last_episode_id, now],
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

pub fn query_history(conn: &Connection, profile_id: Option<i64>) -> Result<SearchResults> {
    // 1. Live streams (join with playback_history)
    let mut live_stmt = conn.prepare(
        "SELECT s.stream_id, s.name, s.stream_icon, s.epg_channel_id, s.category_id, s.tv_archive, s.tv_archive_duration, s.added, s.is_favorite, s.profile_id, s.countries
         FROM playback_history h
         JOIN live_streams s ON h.profile_id = s.profile_id AND h.stream_id = s.stream_id
         WHERE (?1 IS NULL OR h.profile_id = ?1) AND h.media_type = 'live'
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
                countries: row.get(10)?,
                url: None,
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
        "SELECT s.stream_id, s.name, s.stream_icon, s.category_id, s.rating, s.container_extension, s.added, s.is_favorite, s.profile_id, s.release_date
         FROM playback_history h
         JOIN vod_streams s ON h.profile_id = s.profile_id AND h.stream_id = s.stream_id
         WHERE (?1 IS NULL OR h.profile_id = ?1) AND h.media_type = 'vod'
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
            release_date: row.get(9)?,
        })
    })?;
    let mut vod = Vec::new();
    for r in vod_rows {
        vod.push(r?);
    }

    // 3. Series (join with playback_history)
    let mut series_stmt = conn.prepare(
        "SELECT s.series_id, s.name, s.cover, s.category_id, s.rating, s.plot, s.cast_, s.director, s.genre, s.release_date, s.last_modified, s.is_favorite, s.profile_id, h.last_episode_id
         FROM playback_history h
         JOIN series s ON h.profile_id = s.profile_id AND h.stream_id = s.series_id
         WHERE (?1 IS NULL OR h.profile_id = ?1) AND h.media_type = 'series'
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
            last_episode_id: row.get(13)?,
        })
    })?;
    let mut series = Vec::new();
    for r in series_rows {
        series.push(r?);
    }

    Ok(SearchResults { live, vod, series })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_media(conn: &mut Connection) {
        crate::db::migrations::run(conn).unwrap();
        crate::db::live::upsert_streams(
            conn,
            1,
            &[LiveStreamApi {
                stream_id: 10,
                name: Some("CNN".into()),
                stream_icon: None,
                epg_channel_id: None,
                category_id: Some("1".into()),
                tv_archive: Some(0),
                tv_archive_duration: Some(0),
                added: Some(1700000000),
                is_favorite: None,
                profile_id: None,
                url: None,
                countries: None,
            }],
        )
        .unwrap();
        crate::db::vod::upsert_streams(
            conn,
            1,
            &[VodStreamApi {
                stream_id: 20,
                name: Some("Arrival (2016)".into()),
                stream_icon: None,
                category_id: Some("1".into()),
                rating: Some("7.9".into()),
                container_extension: Some("mkv".into()),
                added: Some(1700000000),
                release_date: None,
                is_favorite: None,
                profile_id: None,
            }],
        )
        .unwrap();
        crate::db::series::upsert_series(
            conn,
            1,
            &[SeriesApi {
                series_id: 30,
                name: Some("Andor".into()),
                cover: None,
                category_id: Some("1".into()),
                rating: Some("8.4".into()),
                plot: None,
                cast: None,
                director: None,
                genre: None,
                release_date: Some(1663718400),
                last_modified: Some(1700000000),
                is_favorite: None,
                profile_id: None,
                last_episode_id: None,
            }],
        )
        .unwrap();
    }

    #[test]
    fn test_playback_history_record_update_delete_and_clear() {
        let mut conn = Connection::open_in_memory().unwrap();
        seed_media(&mut conn);

        record_playback(&conn, 1, "live", 10, None).unwrap();
        record_playback(&conn, 1, "vod", 20, None).unwrap();
        record_playback(&conn, 1, "series", 30, Some(3001)).unwrap();

        let history = query_history(&conn, Some(1)).unwrap();
        assert_eq!(history.live.len(), 1);
        assert_eq!(history.vod.len(), 1);
        assert_eq!(history.series.len(), 1);
        assert_eq!(history.series[0].last_episode_id, Some(3001));

        // Re-recording series updates last_episode_id in place
        record_playback(&conn, 1, "series", 30, Some(3002)).unwrap();
        let updated = query_history(&conn, Some(1)).unwrap();
        assert_eq!(updated.series.len(), 1);
        assert_eq!(updated.series[0].last_episode_id, Some(3002));

        // Delete a single item
        delete_playback(&conn, 1, "vod", 20).unwrap();
        let after_delete = query_history(&conn, Some(1)).unwrap();
        assert!(after_delete.vod.is_empty());
        assert_eq!(after_delete.live.len(), 1);

        // Clearing another profile leaves profile 1 intact
        clear_history(&conn, 2).unwrap();
        assert_eq!(query_history(&conn, Some(1)).unwrap().live.len(), 1);

        // Clearing profile 1 empties all history
        clear_history(&conn, 1).unwrap();
        let empty = query_history(&conn, Some(1)).unwrap();
        assert!(empty.live.is_empty() && empty.vod.is_empty() && empty.series.is_empty());
    }
}
