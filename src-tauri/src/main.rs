#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    #[cfg(target_os = "linux")]
    {
        // Fixes WebKitGTK/WSL2 rendering lockups & blank screen issues
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    kusirik::run()
}
