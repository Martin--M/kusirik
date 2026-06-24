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
* **Vue 3 Views & Components**: Manages rendering and visual states. 
* **Pinia Sync Store**: Stores state indicators (e.g. syncing, success timestamps, counts, errors) for live tracking.
* **Tauri Command Wrappers (`src/lib/tauri-commands.ts`)**: Center of type-safe IPC calls. The UI calls these wrappers instead of invoking commands directly.
* **useSync Composable (`src/composables/useSync.ts`)**: Listens to global Tauri events (`sync://started`, `sync://progress`, `sync://done`, `sync://error`) emitted by the backend to coordinate frontend transitions.

### 2. Backend Layer
* **Tauri Command Handlers (`src-tauri/src/commands/`)**: Receives calls from the frontend, maps input variables, and calls core libraries.
* **Sync Engine (`src-tauri/src/sync/engine.rs`)**: Controls sequential caching of TV elements (`live_streams` $\rightarrow$ `vod_streams` $\rightarrow$ `series`). It handles SQLite connections and writes.
* **Xtream Client (`src-tauri/src/api/client.rs`)**: Orchestrates calls to the server. Includes robust deserializers (`deserialize_option_string`, `deserialize_option_i32`) to coerce conflicting server datatypes (e.g., `"tv_archive": "1"` vs `1`).
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
