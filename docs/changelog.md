# Kusirik

## [Unreleased]

## v3.1.0

### Features
* **Live TV & Global Search by Current EPG Title** Searching in Live TV (debounced on-demand lookup) and Global Search now matches currently airing EPG program titles within a bounded ±6h window (`start >= now - 6h`, `stop <= now + 6h`) for fast indexed queries on desktop and mobile.

### Internal
* **Public IPTV Stream Validator Script** Added `scripts/validate_public_iptv.ts` (`npm run validate:iptv`) to concurrently validate `iptv-org` streams via HTTP + HLS manifest/first-segment probing (with optional `--ffprobe` deep checks) and export `valid`/`invalid` reports in both JSON and CSV.
* **Changelog Maintenance Rule** Added `.agent/rules/changelog.md` to keep `## [Unreleased]` updated alongside changes.

## v3.0.0

### Features
* **Release Date & Date Added Sorting** Added release date and date-added sorting for Movies and TV Series, defaulting both catalogs to newest releases first with `added`/`last_modified` tie-breaking when bulk movie data only provides a title year.
* **Series Resume & Watched Indicators** Track the last played episode ID in playback history, display watched badges on series cards and episode rows, and auto-expand the last played season accordion.
* **About Section & Mobile Navigation** Added an About section and modal in Settings and added the History shortcut to mobile bottom navigation.
* **Public IPTV Onboarding** Supported public IPTV profiles in the initial sync completion router check.

### Performance & Architecture
* **SQLite FTS5 Full-Text Search** Replaced `LIKE` table scans with SQLite FTS5 external-content virtual tables (`fts_vod`, `fts_series`, `fts_live`), automatic sync triggers, safe query sanitization, and BM25 relevance ranking.
* **Dual-Connection SQLite Architecture** Separated writer and read-only (`query_only = ON`) SQLite connections in WAL mode with `busy_timeout` so UI reads remain responsive during background syncs.
* **Integer Unix Epoch Schema** Flattened the database schema to a clean v1 baseline and converted all timestamp columns to 64-bit Unix epoch integers, updating frontend sorting and time calculations to direct numeric math (incompatible with v2.0.0 databases).
* **EPG & Listing Query Optimizations** Added composite indexes for EPG time-window and channel lookups, replaced correlated EPG subqueries in live stream listings, and switched TV Guide channel queries from `EXISTS` to indexed `INNER JOIN`s.

### Fixes
* **SQLite Pragma Startup Crash** Configured returning pragmas (`busy_timeout` and `journal_mode = WAL`) via dedicated `rusqlite` APIs instead of `execute_batch` to work cleanly with `bundled-full`/`extra_check`.
* **Live TV & EPG Timestamp Alignment** Aligned `LiveStreamApi.added` with the `INTEGER` epoch schema, normalized epoch-second timestamps across TV Guide, channel rows, and Settings sync status, and surfaced Live TV query errors in the UI.
* **Media Player Launch Error Reporting** Surface exact spawn and fallback dispatcher errors in toast notifications instead of failing silently, and skip recording playback history when launching the external player fails.
* **VOD Release Date Refinement** Parse ISO date strings when provided by `get_vod_streams` and opportunistically refine `vod_streams.release_date` from `get_vod_info` while preserving refined dates across resyncs.

### Internal
* **Unit Tests & Clippy Checks** Expanded backend coverage to 36 Rust unit tests, added a 22-test frontend unit test suite (`npm test` via `node:test`), extracted shared EPG/sorting helpers, and resolved all `cargo clippy --all-targets -- -D warnings` issues.
* **Project Agent Rules** Added `.agent/rules/` guidelines for commit conventions and engineering best practices.

## v2.0.0

### Features
* **Catch-up functionality** Added capability to watch past programs for TV channels that support catch-up
* **TV Guide** Added a TV guide for channels with EPG data
* **Localisation** Added support for French
* **Favorites** Movies/Series/Live TV can now be set as favorite
* **History** Add configurable playback history
* **Profiles** Add support for multiple profiles (Xtream only for now)
* **Public IPTV** Added support for public IPTV

### Fixes
* Fixed copying to clipboard on mobile. Now the frontend first attempts to write to clipboard via the browser/webview API first. Then it falls back to the Rust backend command utilities if needed.
* Fixed global search in mobile vertical layout. Before it was inaccessible.
* Rate limiting on short epg now works properly

### Internal
* Stop using hardcoded profile id 1
* Database changes flattened (incompatible with v1.0.0)

### Minor
* Live stream side bar logos are now square

## v1.0.0

### Core Architecture & Performance
* **Cross-Platform Compatibility:** Native-performance desktop and mobile client built using Tauri 2, Vue 3, and Vite for Windows, Linux, and Android.
* **SQLite Sync Engine:** Local playlist synchronization (Live, VOD, and Series) using SQLite Write-Ahead Logging (WAL) mode to allow instant offline browsing and fast app startup.
* **On-Demand VOD Caching:** Dynamic caching of intensive media details (synopsis, cast lists, media formats) upon user request to minimize network overhead.
* **Credentials Cache:** User credentials (server URL, username, password) are securely stored in the local SQLite settings table, with passwords isolated from the webview and injected only during server-side request signing.
* **Provider Rate Limiting:** Built-in backend request guard enforcing a mandatory 2-second interval between API calls to prevent provider-side IP bans.
* **Native Rust Commands:** System clipboard interactions and external process management are handled natively via Rust handlers rather than the frontend sandbox.

### Media & EPG Management
* **High-Performance EPG Engine:** Streaming parser built for large XMLTV EPG databases, featuring UTC indexing, automatic timezone correction, base64 text-decoding, and live frontend category filters.
* **Blob-Based Image Storage:** Channel logos and movie posters are stored directly in the local database as binary BLOBs. A custom base64 image pre-loader maps these dynamically to prevent local file storage path collisions on Android.
* **External Android Player Integration:** Custom Android Kotlin `IntentPlugin` leveraging `Intent.ACTION_VIEW` and `Intent.createChooser` to resolve stream URLs and hand off playback to external media players (VLC, MX Player, Just Player).
* **Network Stability Hook:** Custom TLS validation layer (`danger_accept_invalid_certs(true)`) that bypasses platform verifier JNI dependencies to eliminate HTTPS thread panics on Android during large EPG and asset downloads.

### User Interface & Layouts
* **Virtualized Media Grids:** Dataset rendering engine powered by TanStack Vue Virtual and a custom `ResizeObserver` implementation, dynamically calculating columns to display lists exceeding 100,000 items with low memory usage.
* **Lazy Image Loading:** Integrated `IntersectionObserver` that defers asset retrieval from the SQLite database until the specific channel logo or poster enters the active viewport.
* **Unified Theme System:** Dark mode interface styled strictly through unified, global CSS variable tokens.
* **Sync Monitoring:** Integrated control center panel providing real-time logging of synchronization progress, including items fetched, elapsed duration, and active data categories.