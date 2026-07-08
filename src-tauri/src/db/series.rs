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
            "INSERT OR REPLACE INTO series (
                profile_id, series_id, name, cover, category_id, rating,
                plot, cast_, director, genre, release_date, last_modified
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )?;

        for series in series_list {
            let cat_id = series.category_id.as_deref().unwrap_or("0");
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
                series.release_date,
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
    fetched_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO series_info (profile_id, series_id, info_json, fetched_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![profile_id, series_id, info_json, fetched_at],
    )?;
    Ok(())
}

pub fn get_series_info(conn: &Connection, profile_id: i64, series_id: i64) -> Result<Option<(String, String)>> {
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
    let sql = "SELECT series_id, name, cover, category_id, rating, plot, cast_, director, genre, release_date, last_modified, is_favorite, profile_id
               FROM series
               WHERE (?1 IS NULL OR profile_id = ?1) AND name LIKE ?2
               ORDER BY name ASC
               LIMIT ?3";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map(rusqlite::params![profile_id, query, limit], |row| {
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
    let mut res = Vec::new();
    for r in rows {
        res.push(r?);
    }
    Ok(res)
}

