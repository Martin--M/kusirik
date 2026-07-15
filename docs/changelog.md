# Kusirik

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