# Future Improvements: Native Mobile Integration

This document tracks planned architectural improvements for native mobile integrations in Kusirik.

## Current Setup: Inline Android Plugin with Rust Bridge

For launching external media players on Android, we currently use a custom Kotlin class (`IntentPlugin.kt`) located inside the main Android host application module (`src-tauri/gen/android`). Because inline plugins bypass standard compilation pipelines, we use a custom Rust command (`launch_android_intent`) in `src-tauri/src/commands/player.rs` to invoke the mobile plugin via `run_mobile_plugin` rather than calling the plugin directly from Javascript.

### Current Limitations

1. **Host Re-generation Vulnerability**: The Android host directory (`src-tauri/gen/android`) is generated scaffolding. Running `tauri android init` or reset commands will overwrite or delete custom Kotlin files (`IntentPlugin.kt`) if they are not manually backed up.
2. **Access Control List (ACL) Bypass**: To circumvent Tauri v2's strict IPC capability verification (which blocks direct frontend invocation of unregistered inline plugins), we route calls through a custom Rust backend command. While secure, this bypasses the standard Tauri capabilities system.
3. **Decoupling**: The app’s core rust code is coupled with custom Kotlin files in the host app package instead of being modularized.

---

## Proposed Improvements

To align with Tauri v2 best practices and industry standards, we recommend adopting one of the following approaches as the codebase grows:

### Approach A: Standalone Workspace Crate (Recommended)

Extract native mobile integration into a local Tauri plugin crate managed via a Cargo workspace (e.g. `plugins/intent`).

#### How it Works
1. Run `npx tauri plugin new --android intent` to generate the plugin structure.
2. Add the plugin crate to a cargo workspace in the root `Cargo.toml`.
3. Move `IntentPlugin.kt` to the plugin’s native source tree (`plugins/intent/android`).
4. Initialize the plugin in `lib.rs` using `.plugin(tauri_plugin_intent::init())`.

#### Benefits
* **Safeguarded Code**: Native mobile code is stored in the workspace repository root, completely isolated from generated scaffolding.
* **Standard Permissions**: Allows normal capability configuration in `capabilities/default.json` (e.g. `"intent:default"` or `"intent:allow-launchPlayer"`) instead of using bridge commands.
* **Reusability**: Decouples native bridge logic from the main application crate, preparing the app for multi-platform extensions (iOS, TV, etc.).

---

### Approach B: Direct JNI Calls in Rust (Alternative)

For simple integrations (like firing a single Android intent), we can bypass Kotlin altogether by writing JNI (Java Native Interface) calls directly in Rust.

#### How it Works
1. Add the `jni` crate as an Android-specific dependency.
2. Inside `src-tauri/src/commands/player.rs`, resolve the Android context dynamically and invoke Java classes (`android.content.Intent`, `android.net.Uri`) from Rust.

#### Benefits
* Eliminates Kotlin code and custom Java/Kotlin classes completely.
* Keeps 100% of the logic within `src-tauri/src/`.
* No extra Cargo workspaces or Gradle configurations to manage.

#### Drawbacks
* Rust JNI syntax is verbose and harder to maintain/debug compared to idiomatic Kotlin.
