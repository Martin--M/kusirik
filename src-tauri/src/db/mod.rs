use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

pub mod migrations;
pub mod profile;
pub mod live;
pub mod vod;
pub mod series;
pub mod epg;
pub mod settings;
pub mod image;
pub mod common;
pub mod favorites;
pub mod history;

/// Tauri managed state wrapper separating heavy write operations (writer Mutex)
/// from concurrent UI read operations (reader Mutex).
/// In SQLite WAL mode, readers execute concurrently without blocking or waiting
/// for write transactions on the writer connection.
#[derive(Clone)]
pub struct DbConn {
    pub writer: Arc<Mutex<Connection>>,
    pub reader: Arc<Mutex<Connection>>,
}

impl DbConn {
    /// Backwards compatibility helper for write operations
    pub fn writer_conn(&self) -> &Arc<Mutex<Connection>> {
        &self.writer
    }

    /// Obtain a read-only lock from the dedicated reader connection for UI SELECT queries.
    /// Because background syncs write exclusively to `writer`, `reader` is never locked by background sync.
    pub fn read(&self) -> Result<MutexGuard<'_, Connection>> {
        self.reader
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire SQLite read lock: {}", e))
    }
}

/// Open (or create) the SQLite database at `db_path`, configure WAL mode, and run migrations.
pub fn open(db_path: &Path) -> Result<DbConn> {
    tracing::info!(path = %db_path.display(), "Opening SQLite database with Dual Connection (Writer + Reader) architecture");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create DB directory: {}", parent.display()))?;
    }

    // 1. Writer connection (for migrations and bulk writes)
    let writer_conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open SQLite writer at {}", db_path.display()))?;
    configure(&writer_conn).context("Failed to configure SQLite writer pragmas")?;
    migrations::run(&writer_conn).context("Database migration failed")?;

    let writer = Arc::new(Mutex::new(writer_conn));

    // 2. Reader connection (dedicated for UI queries, query_only mode)
    let reader_conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open SQLite reader at {}", db_path.display()))?;
    configure(&reader_conn).context("Failed to configure SQLite reader pragmas")?;
    reader_conn.execute("PRAGMA query_only = ON;", [])?;

    let reader = Arc::new(Mutex::new(reader_conn));

    Ok(DbConn { writer, reader })
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
        PRAGMA busy_timeout  = 5000;
        ",
    )
    .context("Failed to set SQLite pragmas")?;
    Ok(())
}
