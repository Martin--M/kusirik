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
const CURRENT_VERSION: u32 = 8;

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

    if current_version < 4 {
        migration_v4(conn).context("Migration v4 failed")?;
    }

    if current_version < 5 {
        migration_v5(conn).context("Migration v5 failed")?;
    }

    if current_version < 6 {
        migration_v6(conn).context("Migration v6 failed")?;
    }

    if current_version < 7 {
        migration_v7(conn).context("Migration v7 failed")?;
    }

    if current_version < 8 {
        migration_v8(conn).context("Migration v8 failed")?;
    }

    // Update schema version
    conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_VERSION}"))
        .context("Failed to update user_version")?;

    tracing::info!("Migrations complete (schema v{CURRENT_VERSION})");
    Ok(())
}

fn migration_v8(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v8 — languages and countries in live_streams");
    conn.execute_batch(
        "ALTER TABLE live_streams ADD COLUMN languages TEXT;
         ALTER TABLE live_streams ADD COLUMN countries TEXT;"
    ).context("Failed to add columns for migration v8")?;
    Ok(())
}

fn migration_v7(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v7 — profile_type in profiles and url in live_streams");
    conn.execute_batch(
        "ALTER TABLE profiles ADD COLUMN profile_type TEXT NOT NULL DEFAULT 'xtream';
         ALTER TABLE live_streams ADD COLUMN url TEXT;"
    ).context("Failed to add columns for migration v7")?;
    Ok(())
}

fn migration_v6(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v6 — password column in profiles");
    // Add password column with default empty string.
    conn.execute_batch(
        "ALTER TABLE profiles ADD COLUMN password TEXT NOT NULL DEFAULT '';"
    ).context("Failed to add password column to profiles")?;
    Ok(())
}

fn migration_v5(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v5 — playback_history table");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS playback_history (
            profile_id  INTEGER NOT NULL,
            media_type  TEXT NOT NULL,
            stream_id   INTEGER NOT NULL,
            played_at   TEXT NOT NULL,
            PRIMARY KEY (profile_id, media_type, stream_id)
         );
         CREATE INDEX IF NOT EXISTS idx_playback_history_played ON playback_history(profile_id, played_at);"
    ).context("Failed to create playback_history table")?;
    Ok(())
}

fn migration_v4(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v4 — is_favorite columns");
    
    let has_live_fav: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('live_streams') WHERE name = 'is_favorite'",
        [],
        |row| row.get::<_, i64>(0).map(|c| c > 0)
    )?;
    if !has_live_fav {
        conn.execute_batch(
            "ALTER TABLE live_streams ADD COLUMN is_favorite INTEGER DEFAULT 0;"
        ).context("Failed to add is_favorite column to live_streams")?;
    }

    let has_vod_fav: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('vod_streams') WHERE name = 'is_favorite'",
        [],
        |row| row.get::<_, i64>(0).map(|c| c > 0)
    )?;
    if !has_vod_fav {
        conn.execute_batch(
            "ALTER TABLE vod_streams ADD COLUMN is_favorite INTEGER DEFAULT 0;"
        ).context("Failed to add is_favorite column to vod_streams")?;
    }

    let has_series_fav: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('series') WHERE name = 'is_favorite'",
        [],
        |row| row.get::<_, i64>(0).map(|c| c > 0)
    )?;
    if !has_series_fav {
        conn.execute_batch(
            "ALTER TABLE series ADD COLUMN is_favorite INTEGER DEFAULT 0;"
        ).context("Failed to add is_favorite column to series")?;
    }

    Ok(())
}

fn migration_v3(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v3 — epg_entries tz_offset column");
    let has_column: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('epg_entries') WHERE name = 'tz_offset'",
        [],
        |row| row.get::<_, i64>(0).map(|c| c > 0)
    )?;
    if !has_column {
        conn.execute_batch(
            "ALTER TABLE epg_entries ADD COLUMN tz_offset TEXT DEFAULT '+00:00';"
        ).context("Failed to add tz_offset column to epg_entries")?;
    }
    Ok(())
}

fn migration_v2(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v2 — image_cache table");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS image_cache (
            url          TEXT PRIMARY KEY,
            data         BLOB NOT NULL,
            content_type TEXT,
            fetched_at   TEXT NOT NULL
        );"
    ).context("Failed to execute v2 schema SQL")?;
    Ok(())
}

fn migration_v1(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v1 — initial schema");
    conn.execute_batch(V1_SCHEMA).context("Failed to execute v1 schema SQL")?;
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
  -- password lives in the OS keyring under:
  --   service: "kusirik"  account: "profile-<id>"
  epg_mode        TEXT    NOT NULL DEFAULT 'xmltv',
  created_at      TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
  -- ('player_windows',       'C:\Program Files\VideoLAN\VLC\vlc.exe')
  -- ('player_android',       'org.videolan.vlc')
  -- ('theme',                'dark')
  -- ('sync_hour',            '3')
  -- ('allowed_formats',      '["ts","m3u8"]')
  -- ('live_format_override', 'ts')
);

-- ─────────────────────────────────────────────
-- Sync tracking
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS sync_log (
  profile_id  INTEGER NOT NULL,
  data_type   TEXT    NOT NULL,
  -- 'live_streams' | 'vod_streams' | 'series' | 'epg'
  fetched_at  TEXT    NOT NULL,
  item_count  INTEGER,
  last_error  TEXT,
  -- NULL on success; last failure message otherwise
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
  -- UTC ISO 8601, e.g. 2026-06-17T14:00:00Z
  stop        TEXT    NOT NULL,
  title       TEXT,
  description TEXT,
  tz_offset   TEXT DEFAULT '+00:00',
  UNIQUE (profile_id, channel_id, start)
  -- prevents duplicate rows on daily re-sync; pairs with INSERT OR REPLACE
);

CREATE INDEX IF NOT EXISTS idx_epg_lookup ON epg_entries(profile_id, channel_id, start);
"#;
