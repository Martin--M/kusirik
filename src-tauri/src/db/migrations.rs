//! Database migrations — run once on first start, version-gated thereafter.
//!
//! Each migration is a (version, sql) pair. The current schema version is stored
//! in SQLite's built-in `PRAGMA user_version`. On startup:
//!   1. Read current user_version from the DB
//!   2. Apply every migration whose version > current user_version, in order
//!   3. Update user_version to the latest applied version

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Current schema version. Bump this when adding a new migration.
const CURRENT_VERSION: u32 = 3;

pub fn run(conn: &Connection) -> Result<()> {
    let current_version: u32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .context("Failed to read user_version pragma")?;

    tracing::info!(
        current_version,
        target_version = CURRENT_VERSION,
        "Running database migrations"
    );

    if current_version < 1 {
        migration_v1(conn).context("Migration v1 failed")?;
    }
    if current_version < 2 {
        migration_v2(conn).context("Migration v2 failed")?;
    }
    if current_version < 3 {
        migration_v3(conn).context("Migration v3 failed")?;
    }

    // Update schema version
    conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_VERSION}"))
        .context("Failed to update user_version")?;

    tracing::info!("Migrations complete (schema v{CURRENT_VERSION})");
    Ok(())
}

fn migration_v1(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v1 — initial flattened schema");
    conn.execute_batch(V1_SCHEMA).context("Failed to execute v1 schema SQL")?;
    Ok(())
}

fn migration_v2(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v2 — add epg_entries indexes for fast queries & pruning");
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_epg_stop ON epg_entries(profile_id, stop);
        CREATE INDEX IF NOT EXISTS idx_epg_channel ON epg_entries(profile_id, channel_id);
        CREATE INDEX IF NOT EXISTS idx_epg_time_window ON epg_entries(profile_id, start, stop);
        CREATE INDEX IF NOT EXISTS idx_epg_active_channels ON epg_entries(profile_id, channel_id, stop, start);
        ",
    )
    .context("Failed to execute v2 schema SQL")?;
    Ok(())
}

fn migration_v3(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v3 — add release_date integer column to vod_streams");
    conn.execute_batch("ALTER TABLE vod_streams ADD COLUMN release_date INTEGER;")
        .context("Failed to execute v3 schema SQL")?;
    Ok(())
}

const V1_SCHEMA: &str = r#"
-- ─────────────────────────────────────────────
-- Profile & Settings
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS profiles (
  id              INTEGER PRIMARY KEY,
  name            TEXT    NOT NULL,
  server_url      TEXT    NOT NULL,
  username        TEXT    NOT NULL,
  password        TEXT    NOT NULL DEFAULT '',
  epg_mode        TEXT    NOT NULL DEFAULT 'xmltv',
  created_at      TEXT    NOT NULL,
  profile_type    TEXT    NOT NULL DEFAULT 'xtream'
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- ─────────────────────────────────────────────
-- Sync tracking
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS sync_log (
  profile_id  INTEGER NOT NULL,
  data_type   TEXT    NOT NULL,
  fetched_at  TEXT    NOT NULL,
  item_count  INTEGER,
  last_error  TEXT,
  PRIMARY KEY (profile_id, data_type)
);

-- ─────────────────────────────────────────────
-- Live TV
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS live_categories (
  profile_id    INTEGER NOT NULL,
  category_id   TEXT    NOT NULL,
  category_name TEXT    NOT NULL,
  PRIMARY KEY (profile_id, category_id)
);

CREATE TABLE IF NOT EXISTS live_streams (
  profile_id            INTEGER NOT NULL,
  stream_id             INTEGER NOT NULL,
  name                  TEXT,
  stream_icon           TEXT,
  epg_channel_id        TEXT,
  category_id           TEXT,
  tv_archive            INTEGER DEFAULT 0,
  tv_archive_duration   INTEGER DEFAULT 0,
  added                 TEXT,
  is_favorite           INTEGER DEFAULT 0,
  url                   TEXT,
  countries             TEXT,
  PRIMARY KEY (profile_id, stream_id)
);

CREATE INDEX IF NOT EXISTS idx_live_category ON live_streams(profile_id, category_id);
CREATE INDEX IF NOT EXISTS idx_live_epg       ON live_streams(profile_id, epg_channel_id);

-- ─────────────────────────────────────────────
-- VOD (Movies)
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS vod_categories (
  profile_id    INTEGER NOT NULL,
  category_id   TEXT    NOT NULL,
  category_name TEXT    NOT NULL,
  PRIMARY KEY (profile_id, category_id)
);

CREATE TABLE IF NOT EXISTS vod_streams (
  profile_id            INTEGER NOT NULL,
  stream_id             INTEGER NOT NULL,
  name                  TEXT,
  stream_icon           TEXT,
  category_id           TEXT,
  rating                TEXT,
  container_extension   TEXT,
  added                 TEXT,
  release_date          INTEGER,
  is_favorite           INTEGER DEFAULT 0,
  PRIMARY KEY (profile_id, stream_id)
);

CREATE INDEX IF NOT EXISTS idx_vod_category ON vod_streams(profile_id, category_id);

CREATE TABLE IF NOT EXISTS vod_info (
  profile_id  INTEGER NOT NULL,
  stream_id   INTEGER NOT NULL,
  info_json   TEXT    NOT NULL,
  fetched_at  TEXT    NOT NULL,
  PRIMARY KEY (profile_id, stream_id)
);

-- ─────────────────────────────────────────────
-- Series
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS series_categories (
  profile_id    INTEGER NOT NULL,
  category_id   TEXT    NOT NULL,
  category_name TEXT    NOT NULL,
  PRIMARY KEY (profile_id, category_id)
);

CREATE TABLE IF NOT EXISTS series (
  profile_id    INTEGER NOT NULL,
  series_id     INTEGER NOT NULL,
  name          TEXT,
  cover         TEXT,
  category_id   TEXT,
  rating        TEXT,
  plot          TEXT,
  cast_         TEXT,
  director      TEXT,
  genre         TEXT,
  release_date  TEXT,
  last_modified TEXT,
  is_favorite   INTEGER DEFAULT 0,
  PRIMARY KEY (profile_id, series_id)
);

CREATE INDEX IF NOT EXISTS idx_series_category ON series(profile_id, category_id);

CREATE TABLE IF NOT EXISTS series_info (
  profile_id  INTEGER NOT NULL,
  series_id   INTEGER NOT NULL,
  info_json   TEXT    NOT NULL,
  fetched_at  TEXT    NOT NULL,
  PRIMARY KEY (profile_id, series_id)
);

-- ─────────────────────────────────────────────
-- EPG
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS epg_entries (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  profile_id  INTEGER NOT NULL,
  channel_id  TEXT    NOT NULL,
  start       TEXT    NOT NULL,
  stop        TEXT    NOT NULL,
  title       TEXT,
  description TEXT,
  tz_offset   TEXT DEFAULT '+00:00',
  UNIQUE (profile_id, channel_id, start)
);

CREATE INDEX IF NOT EXISTS idx_epg_lookup ON epg_entries(profile_id, channel_id, start);
DROP INDEX IF EXISTS idx_epg_query;
CREATE INDEX IF NOT EXISTS idx_epg_query ON epg_entries(profile_id, channel_id, start, stop);

-- ─────────────────────────────────────────────
-- Image Cache
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS image_cache (
    url          TEXT PRIMARY KEY,
    data         BLOB NOT NULL,
    content_type TEXT,
    fetched_at   TEXT NOT NULL
);

-- ─────────────────────────────────────────────
-- Playback History
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS playback_history (
    profile_id  INTEGER NOT NULL,
    media_type  TEXT NOT NULL,
    stream_id   INTEGER NOT NULL,
    played_at   TEXT NOT NULL,
    PRIMARY KEY (profile_id, media_type, stream_id)
);

CREATE INDEX IF NOT EXISTS idx_playback_history_played ON playback_history(profile_id, played_at);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_fresh_db() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Initially, user_version is 0
        let initial_version: u32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(initial_version, 0);

        // Run migrations
        run(&conn).unwrap();

        // After running, user_version should be 2
        let updated_version: u32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(updated_version, 2);

        // Verify that expected tables exist
        let tables = vec![
            "profiles",
            "settings",
            "sync_log",
            "live_categories",
            "live_streams",
            "vod_categories",
            "vod_streams",
            "vod_info",
            "series_categories",
            "series",
            "series_info",
            "epg_entries",
            "image_cache",
            "playback_history",
        ];

        for table in tables {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table '{}' should exist", table);
        }
    }
}

