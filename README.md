# IPTV Helper

A modern, cross-platform IPTV client for Windows and Android 16 (API 36), built with **Tauri 2**, **Vue 3**, and **Vite**.

IPTV Helper connects to any Xtream Codes-compatible server to fetch, cache, and display live channels, VOD movies, and TV series with full EPG support. Playback is handed off to your preferred external player (e.g., VLC or MX Player).

---

## 🚀 Key Features

*   **Fast local caching**: Primary metadata (Live, VOD, and Series) is synced from the server and cached in SQLite for instant startup and navigation.
*   **On-Demand VOD Caching**: Heavy movie synopsis, cast list, and media details are cached locally on-demand when requested to optimize bandwidth.
*   **Offline Image Cache**: Logos and posters are stored locally in the database as BLOBs, backed by `IntersectionObserver` lazy loading to keep runtime memory footprint low.
*   **Secure credentials**: Passwords are saved securely using the OS keyring (Windows Credential Manager / Android Keystore) rather than plaintext files.
*   **EPG Streaming Engine**: High-performance parsing and UTC indexing of large XMLTV files, with on-demand API fallback caching and automatic base64 text decoding.
*   **Virtualised lists**: Custom grid list virtualization featuring dynamic size and column calculations using `ResizeObserver`.
*   **External player handoff**: Seamlessly launches VLC or MX Player, with safe fallback copy-to-clipboard actions executed on the backend to avoid webview permission limitations.
*   **Settings Dashboard & Preferences**: Fully featured control screen supporting visual theme toggles, player path configs, stream format overrides, connection management, and real-time synchronization tracking logs.

---

## 🛠️ Technology Stack

*   **Frontend**: Vue 3 (Composition API with `<script setup lang="ts">`), Pinia 3 for state management, TanStack Vue Query v5 for client-side queries, and TanStack Vue Virtual for list virtualization.
*   **Backend**: Tauri 2 (Rust), Tokio for async runtime, reqwest for API communication, rusqlite for SQL database writes, and keyring-core for credential management.
*   **Database**: SQLite (managed with WAL mode for safe concurrent reads/writes).

---

## 📂 Project Structure

```text
iptv-helper/
├── scripts/                  # Helper utilities (inspect_db.py)
├── src/                      # Vue 3 Frontend
│   ├── main.ts               # App entrypoint & plugins configuration
│   ├── App.vue               # Layout shell & global listeners
│   ├── index.css             # Theme variables & design system tokens
│   ├── components/           # UI & Domain components (movies/MovieList.vue, ui/StreamDetailPanel.vue)
│   ├── composables/          # Reusable logic (usePlayer, useSync, useLiveStreams)
│   ├── views/                # Routed views (SetupView, LiveView, SettingsView)
│   └── stores/               # State management (profile, settings, sync, toast)
└── src-tauri/                # Rust Backend (Tauri)
    ├── Cargo.toml            # Backend dependencies configuration
    ├── tauri.conf.json       # Tauri system configurations
    └── src/
        ├── lib.rs            # Application bootstrap & plugin registration
        ├── main.rs           # Desktop application entrypoint
        ├── db/               # SQLite direct access & migrations management
        └── commands/         # Frontend-exposed commands (player, sync, vod, etc.)
```

---

## 💻 Local Development Setup

### Prerequisites

Ensure you have the following installed on your system:
*   **Node.js** (v22 LTS)
*   **Rust** (via `rustup`)
*   **Android Studio & SDK** (for Android builds, targeting API 36 / Android 16)
*   **Java 21** (JDK required by Gradle)

### Installation

Clone the repository and install npm dependencies:

```bash
# In the project root directory
npm install
```

### Running Locally

To run the application in development mode with hot-reloading:

#### Desktop (Windows / Linux)
```bash
npm run dev
# Or run with tauri CLI wrapper:
npm run tauri:dev
```

#### Android (Emulator or Connected Device)
Ensure your emulator is running or a device is connected via ADB, then run:
```bash
npm run tauri:android:dev
```

---

## 📦 Build & Release

To compile and package the application for production:

### Desktop (Windows / Linux)
Produces an installer (e.g., `.msi` or `.deb` depending on your host OS):
```bash
npm run tauri:build
```

### Android
Produces a release APK / App Bundle:
```bash
npm run tauri:android:build
```

---

## 🧪 Verification & Checks

### Linting & Type-Checking
Runs Vue template and TypeScript static analysis checks:
```bash
npm run type-check
```

### Rust Cargo Checks
Checks that Rust code compiles and satisfies style rules:
```bash
cd src-tauri
cargo check --all-targets
cargo clippy -- -D warnings
cargo test
```

### Database Inspection & Cache Audit
Analyze database file size, table row counts, sync logs, and detect duplicate image cached sizes:
```bash
python3 scripts/inspect_db.py
```

---

## 📖 Architecture & Design Documentation
For details on system internals, database layouts, and data flow architectures, refer to the following documents:
* [System Architecture & Data Flows](file:///home/martin/dev/iptv-helper/docs/architecture.md) — Process model, sequential synchronization sequence, and rate limiters.
* [Database Architecture & Schema](file:///home/martin/dev/iptv-helper/docs/database.md) — SQLite schema specs, WAL concurrency model, and table ERDs.
* [Xtream Codes API Protocol](file:///home/martin/dev/iptv-helper/docs/api_protocol.md) — Endpoint mappings, action parameters, and type-coercion details.
* [UI Design System & Component Guidelines](file:///home/martin/dev/iptv-helper/docs/design_system.md) — CSS token variables, glassmorphic layout models, and reusable component APIs.

