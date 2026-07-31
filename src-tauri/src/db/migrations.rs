//! Database migrations — run once on first start.
//!
//! The schema is defined in `V1_SCHEMA`.

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Current schema version.
const CURRENT_VERSION: u32 = 1;

pub fn run(conn: &Connection) -> Result<()> {
    let current_version: u32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .context("Failed to read user_version pragma")?;

    tracing::info!(
        current_version,
        target_version = CURRENT_VERSION,
        "Running database initialization"
    );

    if current_version < 1 {
        migration_v1(conn).context("Migration v1 failed")?;
    }

    // Update schema version
    conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_VERSION}"))
        .context("Failed to update user_version")?;

    tracing::info!("Database initialization complete (schema v{CURRENT_VERSION})");
    Ok(())
}

fn migration_v1(conn: &Connection) -> Result<()> {
    tracing::info!("Applying migration v1 — initial flattened schema");
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
  password        TEXT    NOT NULL DEFAULT '',
  epg_mode        TEXT    NOT NULL DEFAULT 'xmltv',
  created_at      INTEGER NOT NULL,
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
  fetched_at  INTEGER NOT NULL,
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
  added                 INTEGER,
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
  added                 INTEGER,
  release_date          INTEGER,
  is_favorite           INTEGER DEFAULT 0,
  PRIMARY KEY (profile_id, stream_id)
);

CREATE INDEX IF NOT EXISTS idx_vod_category ON vod_streams(profile_id, category_id);

CREATE TABLE IF NOT EXISTS vod_info (
  profile_id  INTEGER NOT NULL,
  stream_id   INTEGER NOT NULL,
  info_json   TEXT    NOT NULL,
  fetched_at  INTEGER NOT NULL,
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
  release_date  INTEGER,
  last_modified INTEGER,
  is_favorite   INTEGER DEFAULT 0,
  PRIMARY KEY (profile_id, series_id)
);

CREATE INDEX IF NOT EXISTS idx_series_category ON series(profile_id, category_id);

CREATE TABLE IF NOT EXISTS series_info (
  profile_id  INTEGER NOT NULL,
  series_id   INTEGER NOT NULL,
  info_json   TEXT    NOT NULL,
  fetched_at  INTEGER NOT NULL,
  PRIMARY KEY (profile_id, series_id)
);

-- ─────────────────────────────────────────────
-- EPG
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS epg_entries (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  profile_id  INTEGER NOT NULL,
  channel_id  TEXT    NOT NULL,
  start       INTEGER NOT NULL,
  stop        INTEGER NOT NULL,
  title       TEXT,
  description TEXT,
  tz_offset   INTEGER DEFAULT 0,
  UNIQUE (profile_id, channel_id, start)
);

CREATE INDEX IF NOT EXISTS idx_epg_stop ON epg_entries(profile_id, stop);
CREATE INDEX IF NOT EXISTS idx_epg_channel ON epg_entries(profile_id, channel_id);
CREATE INDEX IF NOT EXISTS idx_epg_time_window ON epg_entries(profile_id, start, stop);
CREATE INDEX IF NOT EXISTS idx_epg_active_channels ON epg_entries(profile_id, channel_id, stop, start);

-- ─────────────────────────────────────────────
-- Image Cache
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS image_cache (
    url          TEXT PRIMARY KEY,
    data         BLOB NOT NULL,
    content_type TEXT,
    fetched_at   INTEGER NOT NULL
);

-- ─────────────────────────────────────────────
-- Playback History
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS playback_history (
    profile_id       INTEGER NOT NULL,
    media_type       TEXT NOT NULL,
    stream_id        INTEGER NOT NULL,
    last_episode_id  INTEGER,
    played_at        INTEGER NOT NULL,
    PRIMARY KEY (profile_id, media_type, stream_id)
);

CREATE INDEX IF NOT EXISTS idx_playback_history_played ON playback_history(profile_id, played_at);

-- ─────────────────────────────────────────────
-- FTS5 Full-Text Search Tables & Triggers
-- ─────────────────────────────────────────────

CREATE VIRTUAL TABLE IF NOT EXISTS fts_vod USING fts5(
  name,
  category_id UNINDEXED,
  content='vod_streams',
  content_rowid='stream_id',
  tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER IF NOT EXISTS vod_ai AFTER INSERT ON vod_streams BEGIN
  INSERT INTO fts_vod(rowid, name, category_id) VALUES (new.stream_id, new.name, new.category_id);
END;
CREATE TRIGGER IF NOT EXISTS vod_ad AFTER DELETE ON vod_streams BEGIN
  INSERT INTO fts_vod(fts_vod, rowid, name, category_id) VALUES('delete', old.stream_id, old.name, old.category_id);
END;
CREATE TRIGGER IF NOT EXISTS vod_au AFTER UPDATE ON vod_streams BEGIN
  INSERT INTO fts_vod(fts_vod, rowid, name, category_id) VALUES('delete', old.stream_id, old.name, old.category_id);
  INSERT INTO fts_vod(rowid, name, category_id) VALUES (new.stream_id, new.name, new.category_id);
END;

CREATE VIRTUAL TABLE IF NOT EXISTS fts_series USING fts5(
  name,
  cast_,
  director,
  plot,
  category_id UNINDEXED,
  content='series',
  content_rowid='series_id',
  tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER IF NOT EXISTS series_ai AFTER INSERT ON series BEGIN
  INSERT INTO fts_series(rowid, name, cast_, director, plot, category_id)
  VALUES (new.series_id, new.name, new.cast_, new.director, new.plot, new.category_id);
END;
CREATE TRIGGER IF NOT EXISTS series_ad AFTER DELETE ON series BEGIN
  INSERT INTO fts_series(fts_series, rowid, name, cast_, director, plot, category_id)
  VALUES('delete', old.series_id, old.name, old.cast_, old.director, old.plot, old.category_id);
END;
CREATE TRIGGER IF NOT EXISTS series_au AFTER UPDATE ON series BEGIN
  INSERT INTO fts_series(fts_series, rowid, name, cast_, director, plot, category_id)
  VALUES('delete', old.series_id, old.name, old.cast_, old.director, old.plot, old.category_id);
  INSERT INTO fts_series(rowid, name, cast_, director, plot, category_id)
  VALUES (new.series_id, new.name, new.cast_, new.director, new.plot, new.category_id);
END;

CREATE VIRTUAL TABLE IF NOT EXISTS fts_live USING fts5(
  name,
  category_id UNINDEXED,
  content='live_streams',
  content_rowid='stream_id',
  tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER IF NOT EXISTS live_ai AFTER INSERT ON live_streams BEGIN
  INSERT INTO fts_live(rowid, name, category_id) VALUES (new.stream_id, new.name, new.category_id);
END;
CREATE TRIGGER IF NOT EXISTS live_ad AFTER DELETE ON live_streams BEGIN
  INSERT INTO fts_live(fts_live, rowid, name, category_id) VALUES('delete', old.stream_id, old.name, old.category_id);
END;
CREATE TRIGGER IF NOT EXISTS live_au AFTER UPDATE ON live_streams BEGIN
  INSERT INTO fts_live(fts_live, rowid, name, category_id) VALUES('delete', old.stream_id, old.name, old.category_id);
  INSERT INTO fts_live(rowid, name, category_id) VALUES (new.stream_id, new.name, new.category_id);
END;
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

        // After running, user_version should be 1
        let updated_version: u32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(updated_version, 1);

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

