use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn resolve_stream_url(
    state: State<'_, DbConn>,
    url: String,
    profile_id: Option<i64>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let password = if let Some(pid) = profile_id {
        match crate::db::profile::get(&conn, pid) {
            Ok(Some(p)) => p.password,
            _ => crate::db::settings::get(&conn, "password")
                .map_err(|e| e.to_string())?
                .unwrap_or_default(),
        }
    } else {
        crate::db::settings::get(&conn, "password")
            .map_err(|e| e.to_string())?
            .unwrap_or_default()
    };
    Ok(url.replace("***", &password))
}

#[tauri::command]
pub fn launch_player(
    state: State<'_, DbConn>,
    url: String,
    profile_id: Option<i64>,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = state;
        let _ = url;
        let _ = profile_id;
        Err("On Android, launch_player should be handled via the native IntentPlugin.".to_string())
    }

    #[cfg(not(target_os = "android"))]
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let password = if let Some(pid) = profile_id {
            match crate::db::profile::get(&conn, pid) {
                Ok(Some(p)) => p.password,
                _ => crate::db::settings::get(&conn, "password")
                    .map_err(|e| e.to_string())?
                    .unwrap_or_default(),
            }
        } else {
            crate::db::settings::get(&conn, "password")
                .map_err(|e| e.to_string())?
                .unwrap_or_default()
        };
        let final_url = url.replace("***", &password);
        use std::process::Command;

        let player_path = crate::db::settings::get(&conn, "player_windows")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();

        #[allow(unused_mut)]
        let mut final_player_path = player_path;
        if final_player_path.is_empty() {
            #[cfg(target_os = "windows")]
            {
                let default_paths = [
                    "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
                    "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
                ];
                for path in default_paths {
                    if std::path::Path::new(path).exists() {
                        final_player_path = path.to_string();
                        break;
                    }
                }
            }
        }

        if !final_player_path.is_empty() && Command::new(&final_player_path).arg(&final_url).spawn().is_ok() {
            return Ok(());
        }

        // Fallbacks
        #[cfg(target_os = "windows")]
        {
            if Command::new("cmd").args(["/c", "start", &final_url]).spawn().is_ok() {
                return Ok(());
            }
        }

        #[cfg(target_os = "linux")]
        {
            if Command::new("vlc").arg(&final_url).spawn().is_ok() {
                return Ok(());
            }
            if Command::new("xdg-open").arg(&final_url).spawn().is_ok() {
                return Ok(());
            }
        }

        #[cfg(target_os = "macos")]
        {
            if Command::new("open").arg(&final_url).spawn().is_ok() {
                return Ok(());
            }
        }

        Err("Failed to launch media player or default system handler.".to_string())
    }
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Try clip.exe (WSL2 Windows clipboard)
    if let Ok(mut child) = Command::new("clip.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                drop(stdin);
                if child.wait().is_ok() {
                    return Ok(());
                }
            }
        }
    }

    // Try xclip (Linux X11)
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                drop(stdin);
                if child.wait().is_ok() {
                    return Ok(());
                }
            }
        }
    }

    // Try wl-copy (Linux Wayland)
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                drop(stdin);
                if child.wait().is_ok() {
                    return Ok(());
                }
            }
        }
    }

    Err("No clipboard utility (clip.exe, xclip, or wl-copy) succeeded.".to_string())
}

pub struct AndroidIntentState(pub Option<tauri::plugin::PluginHandle<tauri::Wry>>);

#[tauri::command]
pub fn launch_android_intent(
    state: State<'_, AndroidIntentState>,
    url: String,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        if let Some(ref plugin) = state.0 {
            plugin.run_mobile_plugin::<()>("launchPlayer", serde_json::json!({ "url": url }))
                .map_err(|e| e.to_string())
        } else {
            Err("Android intent plugin handle not initialized.".to_string())
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = state;
        let _ = url;
        Err("launch_android_intent is only supported on Android.".to_string())
    }
}

#[tauri::command]
pub async fn validate_stream_url(
    state: State<'_, DbConn>,
    url: String,
    profile_id: Option<i64>,
) -> Result<bool, String> {
    let password = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        if let Some(pid) = profile_id {
            match crate::db::profile::get(&conn, pid) {
                Ok(Some(p)) => p.password,
                _ => crate::db::settings::get(&conn, "password")
                    .map_err(|e| e.to_string())?
                    .unwrap_or_default(),
            }
        } else {
            crate::db::settings::get(&conn, "password")
                .map_err(|e| e.to_string())?
                .unwrap_or_default()
        }
    };
    let final_url = url.replace("***", &password);

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(6))
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&final_url)
        .header("User-Agent", "VLC/3.0.23 LibVLC/3.0.23")
        .send()
        .await;

    match response {
        Ok(res) => {
            let status = res.status();
            tracing::info!("Pre-check for stream URL: status {}", status);
            if status.is_success() || status.is_redirection() {
                Ok(true)
            } else {
                Err(format!("Precheck failed with HTTP status: {}", status))
            }
        }
        Err(err) => {
            tracing::warn!("Pre-check for stream URL failed: {}", err);
            Err(err.to_string())
        }
    }
}
