//! Database connection bootstrap.
//!
//! Design contract:
//! - This module owns a single `Mutex<Connection>` used for **all Rust-side writes**:
//!   migrations, bulk EPG inserts, profile CRUD, sync log updates.
//! - `tauri-plugin-sql` manages its own internal connection pool, used exclusively
//!   for **JS-initiated SELECT queries** from the frontend. Both paths share the same
//!   SQLite file; WAL mode makes concurrent access safe at the OS level.
//!
//! Never call rusqlite from frontend-facing JS query paths; never call tauri-plugin-sql
//! for write operations. Keep this boundary explicit.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub mod migrations;
pub mod profile;
pub mod live;
pub mod vod;
pub mod series;
pub mod epg;
pub mod settings;
pub mod image;

/// Tauri managed state wrapper around the single rusqlite connection.
pub struct DbConn(pub Mutex<Connection>);

/// Open (or create) the SQLite database at `db_path`, configure it for
/// optimal performance, and run any pending migrations.
pub fn open(db_path: &Path) -> Result<DbConn> {
    tracing::info!(path = %db_path.display(), "Opening SQLite database");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create DB directory: {}", parent.display()))?;
    }

    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open SQLite at {}", db_path.display()))?;

    configure(&conn).context("Failed to configure SQLite pragmas")?;
    migrations::run(&conn).context("Database migration failed")?;

    Ok(DbConn(Mutex::new(conn)))
}

/// Apply WAL mode and performance pragmas.
fn configure(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode  = WAL;
        PRAGMA synchronous   = NORMAL;
        PRAGMA cache_size    = -64000;  -- 64 MB page cache
        PRAGMA foreign_keys  = ON;
        PRAGMA temp_store    = MEMORY;
        ",
    )
    .context("Failed to set SQLite pragmas")?;
    Ok(())
}
