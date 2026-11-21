use anyhow::Result;
use rusqlite::Connection;
use crate::api::series::{SeriesCategoryApi, SeriesApi};
use crate::api::common::CategoryApi;

pub fn upsert_categories(conn: &mut Connection, profile_id: i64, categories: &[SeriesCategoryApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO series_categories (profile_id, category_id, category_name)
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

pub fn upsert_series(conn: &mut Connection, profile_id: i64, series_list: &[SeriesApi]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO series (
                profile_id, series_id, name, cover, category_id, rating,
                plot, cast_, director, genre, release_date, last_modified
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(profile_id, series_id) DO UPDATE SET
                name = excluded.name,
                cover = excluded.cover,
                category_id = excluded.category_id,
                rating = excluded.rating,
                plot = excluded.plot,
                cast_ = excluded.cast_,
                director = excluded.director,
                genre = excluded.genre,
                release_date = excluded.release_date,
                last_modified = excluded.last_modified",
        )?;

        for series in series_list {
            let cat_id = series.category_id.as_deref().unwrap_or("0");

            // Normalize series release_date into Unix timestamp (seconds) if missing epoch
            let rel_timestamp: Option<i64> = match series.release_date {
                Some(ts) if ts > 0 => Some(ts),
                _ => {
                    series.name.as_deref().and_then(|name| {
                        let bytes = name.as_bytes();
                        if bytes.len() >= 6 {
                            for i in 0..=bytes.len() - 6 {
                                if bytes[i] == b'('
                                    && bytes[i + 5] == b')'
                                    && bytes[i + 1..i + 5].iter().all(|b| b.is_ascii_digit())
                                {
                                    if let Ok(year) = name[i + 1..i + 5].parse::<i32>() {
                                        if let Some(ndt) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) {
                                            if let Some(dt) = ndt.and_hms_opt(0, 0, 0) {
                                                return Some(dt.and_utc().timestamp());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None
                    })
                }
            };

            stmt.execute(rusqlite::params![
                profile_id,
                series.series_id,
                series.name,
                series.cover,
                cat_id,
                series.rating,
                series.plot,
                series.cast,
                series.director,
                series.genre,
                rel_timestamp,
                series.last_modified
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn upsert_series_info(
    conn: &Connection,
    profile_id: i64,
    series_id: i64,
    info_json: &str,
    fetched_at: i64,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO series_info (profile_id, series_id, info_json, fetched_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![profile_id, series_id, info_json, fetched_at],
    )?;
    Ok(())
}

pub fn get_series_info(conn: &Connection, profile_id: i64, series_id: i64) -> Result<Option<(String, i64)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT info_json, fetched_at FROM series_info WHERE profile_id = ?1 AND series_id = ?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![profile_id, series_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?)))
    } else {
        Ok(None)
    }
}

pub fn query_categories(conn: &Connection, profile_id: Option<i64>) -> Result<Vec<CategoryApi>> {
    crate::db::common::query_categories_generic(conn, "series_categories", profile_id)
}

pub fn query_series(
    conn: &Connection,
    profile_id: Option<i64>,
    category_id: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<Vec<SeriesApi>> {
    let db_category = match category_id {
        None | Some("all") => None,
        Some(cat) => Some(cat),
    };

    let sql = "SELECT series_id, name, cover, category_id, rating, plot, cast_, director, genre, release_date, last_modified, is_favorite, profile_id
               FROM series
               WHERE (?1 IS NULL OR profile_id = ?1)
                 AND (?2 IS NULL OR category_id = ?2)
               ORDER BY name ASC
               LIMIT ?3 OFFSET ?4";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, db_category, limit, offset], |row| {
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
            last_episode_id: None,
        })
    })?;

    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

pub fn search_series(
    conn: &Connection,
    profile_id: Option<i64>,
    query: &str,
    limit: u32,
) -> Result<Vec<SeriesApi>> {
    let sanitized = crate::commands::search::sanitize_fts5_query(query);
    if sanitized.is_empty() {
        return Ok(Vec::new());
    }

    let sql = "SELECT s.series_id, s.name, s.cover, s.category_id, s.rating, s.plot, s.cast_, s.director, s.genre, s.release_date, s.last_modified, s.is_favorite, s.profile_id
               FROM series s
               JOIN fts_series fts ON s.series_id = fts.rowid
               WHERE (?1 IS NULL OR s.profile_id = ?1) AND fts_series MATCH ?2
               ORDER BY bm25(fts_series, 10.0, 2.0, 2.0, 1.0) ASC
               LIMIT ?3";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, sanitized, limit], |row| {
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
            last_episode_id: None,
        })
    })?;
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_series(id: i64, name: &str, cat: &str, plot: &str, release_date: Option<i64>) -> SeriesApi {
        SeriesApi {
            series_id: id,
            name: Some(name.to_string()),
            cover: None,
            category_id: Some(cat.to_string()),
            rating: Some("8.5".to_string()),
            plot: Some(plot.to_string()),
            cast: Some("Jane Doe, John Smith".to_string()),
            director: Some("Alex Director".to_string()),
            genre: Some("Sci-Fi".to_string()),
            release_date,
            last_modified: Some(1700000000),
            is_favorite: None,
            profile_id: None,
            last_episode_id: None,
        }
    }

    #[test]
    fn test_series_upsert_query_info_and_fts5_ranking() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        upsert_categories(
            &mut conn,
            1,
            &[
                SeriesCategoryApi { category_id: Some("5".into()), category_name: "Sci-Fi".into(), profile_id: None },
                SeriesCategoryApi { category_id: Some("6".into()), category_name: "Drama".into(), profile_id: None },
            ],
        )
        .unwrap();
        assert_eq!(query_categories(&conn, Some(1)).unwrap().len(), 2);

        upsert_series(
            &mut conn,
            1,
            &[
                // Exact API release_date preserved
                sample_series(1, "Starlight", "5", "Space exploration drama", Some(1760000000)),
                // Missing release_date falls back to (2024) from title
                sample_series(2, "Dark Harbor (2024)", "6", "A mystery about starlight on the coast", None),
            ],
        )
        .unwrap();

        let all = query_series(&conn, Some(1), Some("all"), 0, 50).unwrap();
        assert_eq!(all.len(), 2);
        // Ordered by name ASC: Dark Harbor (2024), then Starlight
        assert_eq!(all[0].name.as_deref(), Some("Dark Harbor (2024)"));
        assert_eq!(all[0].release_date, Some(1704067200)); // 2024-01-01
        assert_eq!(all[1].release_date, Some(1760000000));

        let cat5 = query_series(&conn, Some(1), Some("5"), 0, 50).unwrap();
        assert_eq!(cat5.len(), 1);
        assert_eq!(cat5[0].series_id, 1);

        // Series info cache
        assert_eq!(get_series_info(&conn, 1, 1).unwrap(), None);
        upsert_series_info(&conn, 1, 1, r#"{"episodes":{}}"#, 1700000999).unwrap();
        let (json, ts) = get_series_info(&conn, 1, 1).unwrap().unwrap();
        assert_eq!(json, r#"{"episodes":{}}"#);
        assert_eq!(ts, 1700000999);

        // FTS5 BM25 search: title match ("Starlight") ranks above plot-only match ("starlight")
        let hits = search_series(&conn, Some(1), "starlight", 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].series_id, 1);
        assert_eq!(hits[1].series_id, 2);

        // Cast / director search works
        let cast_hits = search_series(&conn, Some(1), "Alex Director", 10).unwrap();
        assert_eq!(cast_hits.len(), 2);
    }
}

