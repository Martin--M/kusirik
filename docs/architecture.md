# System Architecture & Data Flow

This document details the high-level system components, communication flows, and data synchronization patterns of the IPTV Helper application.

## High-Level System Architecture

IPTV Helper is built on Tauri, splitting the codebase into a frontend User Interface (running in a system webview) and a backend Core process (written in Rust).

```mermaid
graph TD
    subgraph Frontend ["WebView - Vue 3 / Vite"]
        UI["Vue View Layer"]
        PQ["Pinia Store / vue-query"]
        TC["Tauri command wrappers"]
    end

    subgraph Backend ["Tauri Core - Rust"]
        TH["Tauri Command Handlers"]
        SE["Sync Engine"]
        AC["Xtream API Client"]
        RL["Rate Limiter Guard"]
        DB["rusqlite Database Manager"]
    end

    subgraph External ["External Services"]
        XS[("Xtream Codes Server")]
    end

    UI -->|Reacts to| PQ
    UI -->|Initiates action| TC
    TC -->|IPC Invoke| TH
    TH -->|Spawns sync| SE
    SE -->|Uses client| AC
    AC -->|Acquires lock| RL
    RL -->|HTTP GET| XS
    SE -->|Bulk Write| DB
    DB -.->|Shared SQLite WAL| PQ
    SE -->|Tauri Event Emitter| PQ
```

---

## Component Descriptions

### 1. Frontend Layer
* **Vue 3 Views & Components**: Manages rendering and visual states. Implements clean, reusable layout structures (e.g. [StreamDetailPanel.vue](../src/components/ui/StreamDetailPanel.vue) to consolidate the details panel and drawer, and [CategorySidebar.vue](../src/components/ui/CategorySidebar.vue) for responsive layouts).
* **Pinia Sync Store**: Stores state indicators (e.g. syncing, success timestamps, counts, errors) for live tracking.
* **Pinia Toast Store (`src/stores/toast.store.ts`)**: Manages global toast state and notifications, paired with the root-mounted [AppToast.vue](../src/components/ui/AppToast.vue) component.
* **Tauri Command Wrappers (`src/lib/tauri-commands.ts`)**: Center of type-safe IPC calls. The UI calls these wrappers instead of invoking commands directly.
* **useSync Composable (`src/composables/useSync.ts`)**: Listens to global Tauri events (`sync://started`, `sync://progress`, `sync://done`, `sync://error`) emitted by the backend to coordinate frontend transitions.
* **Lazy Image Loader ([CachedImage.vue](../src/components/ui/CachedImage.vue))**: Uses a browser `IntersectionObserver` to defer fetching logo and poster images from the backend/database until they enter the viewport. Properly handles cleanup and object URL revocation on unmount to prevent leaks.
* **Virtualized Media Grids ([MovieList.vue](../src/components/movies/MovieList.vue), [SeriesList.vue](../src/components/series/SeriesList.vue))**: Calculates columns and row metrics dynamically via `ResizeObserver` to virtually render large collections of movies and TV series. Utilizes a unified [ListRowItem.vue](../src/components/ui/ListRowItem.vue) shared row component to display metadata uniformally in list views.

### 2. Backend Layer
* **Tauri Command Handlers (`src-tauri/src/commands/`)**: Receives calls from the frontend, maps input variables, and routes commands (e.g., VOD and Series queries, EPG, or player actions).
* **Sync Engine (`src-tauri/src/sync/engine.rs`)**: Controls sequential caching of TV elements (`live_streams` $\rightarrow$ `vod_streams` $\rightarrow$ `series`). It handles SQLite connections and writes.
* **Xtream Client (`src-tauri/src/api/client.rs`)**: Orchestrates calls to the server. Includes robust deserializers (`deserialize_option_string`, `deserialize_option_i32`) to coerce conflicting server datatypes (e.g., `"tv_archive": "1"` vs `1`).
* **Unified Category Modules**: Utilizes [CategoryApi](../src-tauri/src/api/common.rs) and the generic [query_categories_generic](../src-tauri/src/db/common.rs) function to unify categories mapping and database queries for Live, VOD, and Series.
* **On-Demand VOD & TV Series Caching**: Backend command handlers check local SQLite cache tables (`vod_info` and `series_info`), fallback to Xtream API queries on miss, cache results to the DB, and return.
* **System Clipboard Integration**: Due to frontend sandbox limitations, copy-to-clipboard operations are delegated to Rust commands on the backend to copy URLs and EPG information reliably.
* **Rate Limiter**: Tracks a thread-safe `last_request_time: Mutex<Option<Instant>>`. Ensures that no requests fire within 2 seconds of each other, preventing client bans.

---

## Data Sync & Auto-Recovery Flow

The application implements a resilient sequential sync flow that recovers automatically if interrupted or failed.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant Router as Vue Router
    participant UI as SetupView (Vue)
    participant Pinia as Sync Store
    participant SE as Sync Engine (Rust)
    participant Client as API Client (Rust)
    participant DB as SQLite DB

    User->>Router: Opens App
    Router->>DB: Query profile & sync_log
    alt Profile exists but sync is incomplete
        Router->>UI: Redirect to SetupView (Auto-Trigger mode)
        UI->>Pinia: Clear/Reset states
        UI->>SE: trigger_sync('live_streams')
        activate SE
        Note over SE: Loop through live, VOD, and series streams
        
        SE->>Client: Fetch Live Categories
        Client->>Client: Enforce 2s delay
        Client->>SE: Returns Categories JSON
        
        SE->>Client: Fetch Live Streams
        Client->>Client: Enforce 2s delay
        Client->>SE: Returns Streams JSON
        
        SE->>DB: Bulk Write (Upsert categories & streams)
        SE->>Pinia: Emit 'sync://done' (live_streams)
        
        Note over SE: Process VOD Streams...
        Note over SE: Process TV Series...
        
        SE->>UI: Emit completion events
        deactivate SE
        UI->>Router: Navigate to /live
    else Profile exists and sync is complete
        Router->>Router: Navigate to /live
    end
```

### Resiliency & Auto-Recovery Behaviors:
1. **Routing Guard**: The router checks the database's `sync_log`. If the app was closed mid-sync, or a step failed on the last run, the app forces routing back to `/setup`.
2. **Auto-Resume**: When `/setup` mounts under these conditions, it automatically sets `isSyncing = true`, loads successful states, and resumes the sync.
3. **Log Diagnostics**: If the server returns bad formats, the backend reads the body as text, saves the exact output to `~/.local/share/com.iptv.helper/failed_<action>.txt`, and throws a descriptive error so you can see exactly what went wrong.

---

## Technical Details of Recent Implementations

### 1. Lazy Image Loading & BLOB Cache
To avoid overwhelming the application memory and disk channels during rapid scrolling of thousands of channels or movies:
- Logo and poster images render via `CachedImage.vue`.
- An `IntersectionObserver` with a `200px` root margin detects when the card is close to the viewport.
- Only then is the target URL requested. The backend fetches the image, stores it in the `image_cache` BLOB table, and returns a binary stream.
- The frontend loads this into a local Object URL (`blob:http...`). When the component unmounts, the observer disconnects and the Object URL is explicitly revoked to free up memory.

### 2. On-Demand VOD & TV Series Caching Flow
Rather than pre-syncing heavy, detailed metadata (such as cast, synopsis, plot, background covers, and the nested seasons/episodes array) for all items in a playlist (which could be tens of thousands of records):
1. The frontend invokes `get_vod_info` or `get_series_info` when opening a media detail drawer.
2. The Tauri Core intercepts the call and checks the `vod_info` or `series_info` SQLite table.
3. **Cache Hit**: Instantly returns cached JSON data.
4. **Cache Miss**: Authenticates via keyring-stored credentials, queries the Xtream API, caches the results back to the database (`vod_info` or `series_info`) for future hits, and returns the data.

### 3. Desktop/Mobile UI Simplification
Large view components have been broken down:
- Standardized filter heads ([FilterHeader.vue](../src/components/ui/FilterHeader.vue)) consolidate search criteria and layout settings.
- Sorted selects ([CustomSelect.vue](../src/components/ui/CustomSelect.vue)) provide custom styled, clickable drop-down fields.
- Drawer layouts ([StreamDetailPanel.vue](../src/components/ui/StreamDetailPanel.vue)) combine desktop side panels and mobile bottom sheets into a unified slot-based API.
- Dashboard Card Layouts: The settings view ([SettingsView.vue](../src/views/SettingsView.vue)) groups statistical details (categories counts and cached items counts) inside a dedicated "Database Statistics" card, leaving the "Database Synchronization" panel simplified to category last-synced times and background sync status indicators.

### 4. Collapsible & Resizable Sidebar Panels
To prevent mouse scrollbar interference, the left category sidebar ([CategorySidebar.vue](../src/components/ui/CategorySidebar.vue)) and right detailed stream panel ([StreamDetailPanel.vue](../src/components/ui/StreamDetailPanel.vue)) use non-scrollable flex wrapper layouts. Overlaid drag handles are positioned outside the scrollable containers, avoiding mouse event conflicts. Collapsible state triggers toggle sidebar visibility and persist state in `localStorage`.

### 5. Media Player Handoff & Password Isolation
* **Credentials Isolation**: Stream URLs exposed to the frontend use a secure `"***"` placeholder. The Tauri backend injects the real password from the database right before player execution or copying to the clipboard.
* **Platform Playback Delegation**: 
  - **Desktop (Windows/Linux)**: Spawns the configured player path (e.g. VLC) via child processes, falling back to shell handler commands (`cmd.exe /c start` or `vlc`/`xdg-open`). Probes common default install paths on Windows if the path configuration is left empty.
  - **Android**: Invokes a custom Kotlin `IntentPlugin` that launches Android's intent chooser using `ACTION_VIEW` and `video/*` mime-type, allowing stream playback in external players (such as VLC or MX Player). Only the platform-relevant player configuration input field (Windows path on desktop, android package on mobile) is displayed in the UI.
* **Safe Operations**: Synchronization updates are gated behind confirmation alerts on the client side to avoid starting background requests accidentally.

### 6. EPG (Electronic Program Guide) Subsystem
To provide a rich program guide without overloading memory or disk resources:
* **EPG Streaming Engine**: Built using `quick-xml`'s stream reader. It streams and parses heavy XMLTV EPG data, writing to SQLite in chunks of 2,000 entries. Progress updates are pushed to the UI via Tauri IPC events.
* **On-Demand Fallback**: When opening a channel with no EPG entries in the local database (or when existing database entries don't cover the current active time-window), the backend fetches short EPG data on-demand from the Xtream Codes API. To avoid hammering the player API during rapid scrolling or toggling, on-demand fetches are rate-limited in-memory to once every 2 hours per channel. It also decodes base64-encoded titles/descriptions (often returned by some providers) and caches the results to the local SQLite database.
* **UTC Timezone Normalization**: All EPG start and stop timestamps (from both XMLTV and short EPG APIs) are normalized into UTC ISO 8601 strings ending in `Z`. Since SQLite lacks native datetime columns, dates are queried using lexicographical string comparisons (`start < to AND stop > from`). Standardization to `Z` timezone suffixes prevents offset-based sorting bugs.
* **Startup Cleanup & Sync Logic**: On startup, a background task purges EPG entries older than 48 hours to manage database size. If the last successful sync was more than 24 hours ago, it triggers an automatic background sync.

