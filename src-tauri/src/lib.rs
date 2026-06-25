pub mod db;
pub mod commands;
pub mod api;
pub mod sync;

use anyhow::Context;
use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("iptv_helper=debug,info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .setup(|app| {
            // Determine DB path
            let app_data_dir = app.path().app_data_dir()
                .context("Failed to resolve app_data_dir")?;
            let db_path = app_data_dir.join("iptv.db");

            // Open Rust DB connection (runs migrations)
            let db_conn = db::open(&db_path)?;
            
            // Manage state for Rust commands
            app.manage(db_conn);

            // Spawn background startup tasks (clean old EPG and check/run EPG sync)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::sync::engine::run_startup_tasks(app_handle).await {
                    tracing::error!("Startup EPG tasks failed: {}", e);
                }
            });

            tracing::info!("App setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::profile::save_profile,
            commands::profile::get_profile,
            commands::profile::delete_profile,
            commands::profile::test_connection,
            commands::sync::trigger_sync,
            commands::sync::get_sync_status,
            commands::live::get_live_categories,
            commands::live::get_live_streams,
            commands::vod::get_vod_categories,
            commands::vod::get_vod_streams,
            commands::vod::get_vod_info,
            commands::series::get_series_categories,
            commands::series::get_series,
            commands::series::get_series_info,
            commands::epg::get_epg_for_channel,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::player::launch_player,
            commands::player::resolve_stream_url,
            commands::player::copy_to_clipboard,
            commands::image::get_image_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
