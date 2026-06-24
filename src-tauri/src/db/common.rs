use anyhow::Result;
use rusqlite::Connection;
use crate::api::common::CategoryApi;

pub fn query_categories_generic(
    conn: &Connection,
    table_name: &str,
    profile_id: i64,
) -> Result<Vec<CategoryApi>> {
    let sql = format!(
        "SELECT category_id, category_name
         FROM {}
         WHERE profile_id = ?1
         ORDER BY category_name ASC",
        table_name
    );
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(rusqlite::params![profile_id], |row| {
        Ok(CategoryApi {
            category_id: row.get(0)?,
            category_name: row.get(1)?,
        })
    })?;

    let mut categories = Vec::new();
    for row in rows {
        categories.push(row?);
    }
    Ok(categories)
}
