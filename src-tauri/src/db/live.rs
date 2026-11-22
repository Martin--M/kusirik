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
    let (db_category, is_uncategorized) = match category_id {
        None | Some("all") => (None, false),
        Some("0") | Some("") | Some("uncategorized") => (None, true),
        Some(cat) => (Some(cat), false),
    };

    let sql = "SELECT stream_id, name, stream_icon, epg_channel_id, category_id, tv_archive, tv_archive_duration, added, max(is_favorite) as is_favorite, profile_id, countries, NULL AS current_title
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
    let rows = stmt.query_map(rusqlite::params![profile_id, db_category, is_uncategorized, limit, offset], |row| {
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
    search_streams_at(conn, profile_id, query, limit, chrono::Utc::now().timestamp())
}

pub fn search_streams_at(
    conn: &Connection,
    profile_id: Option<i64>,
    query: &str,
    limit: u32,
    now: i64,
) -> Result<Vec<LiveStreamDto>> {
    let trimmed = query.trim();
    if trimmed.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }

    let min_start = now - crate::db::epg::ACTIVE_EPG_SEARCH_WINDOW_SECS;
    let max_stop = now + crate::db::epg::ACTIVE_EPG_SEARCH_WINDOW_SECS;
    let mut res = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. Primary matches: channel name via FTS5 BM25
    let sanitized = crate::commands::search::sanitize_fts5_query(trimmed);
    if !sanitized.is_empty() {
        let sql = "SELECT s.stream_id, s.name, s.stream_icon, s.epg_channel_id, s.category_id, s.tv_archive, s.tv_archive_duration, s.added, s.is_favorite, s.profile_id, s.countries,
                          (SELECT title FROM epg_entries
                           WHERE epg_entries.profile_id = s.profile_id
                             AND epg_entries.channel_id = s.epg_channel_id
                             AND epg_entries.start >= ?5
                             AND epg_entries.start <= ?4
                             AND epg_entries.stop > ?4
                             AND epg_entries.stop <= ?6
                           LIMIT 1) AS current_title
                   FROM live_streams s
                   JOIN fts_live fts ON s.stream_id = fts.rowid
                   WHERE (?1 IS NULL OR s.profile_id = ?1) AND fts_live MATCH ?2
                   ORDER BY bm25(fts_live) ASC
                   LIMIT ?3";
        let mut stmt = conn.prepare_cached(sql)?;
        let rows = stmt.query_map(
            rusqlite::params![profile_id, sanitized, limit, now, min_start, max_stop],
            |row| {
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
            },
        )?;

        for r in rows {
            let dto = r?;
            if let Some(pid) = dto.stream.profile_id {
                seen.insert((pid, dto.stream.stream_id));
            }
            res.push(dto);
        }
    }

    // 2. Secondary matches: channels whose currently airing EPG title matches within [now-6h, now+6h]
    let remaining = (limit as usize).saturating_sub(res.len());
    if remaining > 0 {
        if let Some(like_pattern) = crate::db::epg::build_like_contains_pattern(trimmed) {
            let epg_sql = "SELECT s.stream_id, s.name, s.stream_icon, s.epg_channel_id, s.category_id, s.tv_archive, s.tv_archive_duration, s.added, s.is_favorite, s.profile_id, s.countries, ee.title AS current_title
                           FROM epg_entries ee
                           JOIN live_streams s
                             ON s.profile_id = ee.profile_id
                            AND s.epg_channel_id = ee.channel_id
                           WHERE (?1 IS NULL OR ee.profile_id = ?1)
                             AND ee.start >= ?2
                             AND ee.start <= ?3
                             AND ee.stop > ?3
                             AND ee.stop <= ?4
                             AND ee.title IS NOT NULL
                             AND ee.title LIKE ?5 ESCAPE '\\'
                           GROUP BY s.profile_id, s.name
                           ORDER BY s.name ASC
                           LIMIT ?6";
            let mut stmt = conn.prepare_cached(epg_sql)?;
            let rows = stmt.query_map(
                rusqlite::params![profile_id, min_start, now, max_stop, like_pattern, limit],
                |row| {
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
                },
            )?;

            for r in rows {
                if res.len() >= limit as usize {
                    break;
                }
                let dto = r?;
                let key = (dto.stream.profile_id.unwrap_or_default(), dto.stream.stream_id);
                if seen.insert(key) {
                    res.push(dto);
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_query_live_streams_with_integer_and_null_added() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        let streams = vec![
            LiveStreamApi {
                stream_id: 101,
                name: Some("Channel Alpha".to_string()),
                stream_icon: None,
                epg_channel_id: Some("alpha.tv".to_string()),
                category_id: Some("10".to_string()),
                tv_archive: Some(1),
                tv_archive_duration: Some(3),
                added: Some(1_700_000_000),
                is_favorite: Some(0),
                profile_id: Some(1),
                url: None,
                countries: None,
            },
            LiveStreamApi {
                stream_id: 102,
                name: Some("Channel Beta".to_string()),
                stream_icon: None,
                epg_channel_id: None,
                category_id: Some("10".to_string()),
                tv_archive: Some(0),
                tv_archive_duration: Some(0),
                added: None,
                is_favorite: Some(0),
                profile_id: Some(1),
                url: None,
                countries: None,
            },
        ];

        upsert_streams(&mut conn, 1, &streams).unwrap();

        let queried = query_streams(&conn, Some(1), None, 0, 50).unwrap();
        assert_eq!(queried.len(), 2);
        assert_eq!(queried[0].stream.added, Some(1_700_000_000));
        assert_eq!(queried[1].stream.added, None);

        let mirrors = query_mirrors(&conn, 1, "Channel Alpha").unwrap();
        assert_eq!(mirrors.len(), 1);
        assert_eq!(mirrors[0].added, Some(1_700_000_000));
    }

    #[test]
    fn test_live_categories_filtering_grouping_and_fts5_search() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        upsert_categories(
            &mut conn,
            1,
            &[
                CategoryApi { category_id: Some("10".into()), category_name: "Sports".into(), profile_id: None },
                CategoryApi { category_id: Some("20".into()), category_name: "News".into(), profile_id: None },
            ],
        )
        .unwrap();
        upsert_categories(
            &mut conn,
            2,
            &[CategoryApi { category_id: Some("99".into()), category_name: "Cinema".into(), profile_id: None }],
        )
        .unwrap();

        assert_eq!(query_categories(&conn, Some(1)).unwrap().len(), 2);
        assert_eq!(query_categories(&conn, None).unwrap().len(), 3);

        // Two mirrors with the same name on profile 1, plus uncategorized and another profile
        upsert_streams(
            &mut conn,
            1,
            &[
                LiveStreamApi {
                    stream_id: 1,
                    name: Some("ESPN HD".into()),
                    stream_icon: None,
                    epg_channel_id: Some("espn.us".into()),
                    category_id: Some("10".into()),
                    tv_archive: Some(1),
                    tv_archive_duration: Some(3),
                    added: Some(1700000000),
                    is_favorite: None,
                    profile_id: None,
                    url: None,
                    countries: None,
                },
                LiveStreamApi {
                    stream_id: 2,
                    name: Some("ESPN HD".into()),
                    stream_icon: None,
                    epg_channel_id: Some("espn.us".into()),
                    category_id: Some("10".into()),
                    tv_archive: Some(0),
                    tv_archive_duration: Some(0),
                    added: Some(1700000010),
                    is_favorite: None,
                    profile_id: None,
                    url: None,
                    countries: None,
                },
                LiveStreamApi {
                    stream_id: 3,
                    name: Some("Café News".into()),
                    stream_icon: None,
                    epg_channel_id: None,
                    category_id: None, // defaults to "0" (uncategorized)
                    tv_archive: Some(0),
                    tv_archive_duration: Some(0),
                    added: None,
                    is_favorite: None,
                    profile_id: None,
                    url: None,
                    countries: None,
                },
            ],
        )
        .unwrap();

        // Mark second mirror as favorite; GROUP BY should surface max(is_favorite) = 1
        crate::db::favorites::toggle_favorite(&conn, 1, "live", 2).unwrap();

        let sports = query_streams(&conn, Some(1), Some("10"), 0, 50).unwrap();
        assert_eq!(sports.len(), 1);
        assert_eq!(sports[0].stream.name.as_deref(), Some("ESPN HD"));
        assert_eq!(sports[0].stream.is_favorite, Some(1));
        assert_eq!(query_mirrors(&conn, 1, "ESPN HD").unwrap().len(), 2);

        let uncat = query_streams(&conn, Some(1), Some("uncategorized"), 0, 50).unwrap();
        assert_eq!(uncat.len(), 1);
        assert_eq!(uncat[0].stream.name.as_deref(), Some("Café News"));

        // FTS5 search with diacritics removal ("cafe" matches "Café News")
        let hits = search_streams(&conn, Some(1), "cafe", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].stream.stream_id, 3);

        // Punctuation-only query returns empty without SQL error
        assert!(search_streams(&conn, Some(1), "*** :: ---", 10).unwrap().is_empty());
    }

    #[test]
    fn test_search_streams_at_combines_channel_and_active_epg_titles_without_duplicates() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        upsert_streams(
            &mut conn,
            1,
            &[
                LiveStreamApi {
                    stream_id: 10,
                    name: Some("Sports Network".into()),
                    stream_icon: None,
                    epg_channel_id: Some("sports.net".into()),
                    category_id: Some("1".into()),
                    tv_archive: Some(0),
                    tv_archive_duration: Some(0),
                    added: Some(1700000000),
                    is_favorite: None,
                    profile_id: None,
                    url: None,
                    countries: None,
                },
                LiveStreamApi {
                    stream_id: 20,
                    name: Some("Channel Seven".into()),
                    stream_icon: None,
                    epg_channel_id: Some("ch7".into()),
                    category_id: Some("1".into()),
                    tv_archive: Some(0),
                    tv_archive_duration: Some(0),
                    added: Some(1700000000),
                    is_favorite: None,
                    profile_id: None,
                    url: None,
                    countries: None,
                },
                LiveStreamApi {
                    stream_id: 30,
                    name: Some("Loop Channel".into()),
                    stream_icon: None,
                    epg_channel_id: Some("loop.ch".into()),
                    category_id: Some("1".into()),
                    tv_archive: Some(0),
                    tv_archive_duration: Some(0),
                    added: Some(1700000000),
                    is_favorite: None,
                    profile_id: None,
                    url: None,
                    countries: None,
                },
            ],
        )
        .unwrap();

        let now = 1_795_269_600i64;
        let hour = 3600i64;
        crate::db::epg::bulk_insert(
            &mut conn,
            &[
                // Matches both channel name and EPG title -> must appear only once
                crate::db::epg::EpgEntry {
                    profile_id: 1,
                    channel_id: "sports.net".into(),
                    start: now - hour,
                    stop: now + hour,
                    title: Some("Sports Center".into()),
                    description: None,
                    tz_offset: Some(0),
                },
                // Matches only EPG title within +-6h -> included after channel name hit
                crate::db::epg::EpgEntry {
                    profile_id: 1,
                    channel_id: "ch7".into(),
                    start: now - hour,
                    stop: now + hour,
                    title: Some("Evening Sports Tonight".into()),
                    description: None,
                    tz_offset: Some(0),
                },
                // Outside +-6h start bound -> excluded
                crate::db::epg::EpgEntry {
                    profile_id: 1,
                    channel_id: "loop.ch".into(),
                    start: now - 7 * hour,
                    stop: now + hour,
                    title: Some("24h Sports Loop".into()),
                    description: None,
                    tz_offset: Some(0),
                },
            ],
        )
        .unwrap();

        let hits = search_streams_at(&conn, Some(1), "sports", 10, now).unwrap();
        assert_eq!(hits.len(), 2);
        // Channel name hit comes first, EPG-only hit second
        assert_eq!(hits[0].stream.stream_id, 10);
        assert_eq!(hits[0].current_title.as_deref(), Some("Sports Center"));
        assert_eq!(hits[1].stream.stream_id, 20);
        assert_eq!(hits[1].current_title.as_deref(), Some("Evening Sports Tonight"));
    }
}

