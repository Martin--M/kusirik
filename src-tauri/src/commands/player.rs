use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn resolve_stream_url(
    state: State<'_, DbConn>,
    url: String,
    profile_id: Option<i64>,
) -> Result<String, String> {
    let conn = state.read().map_err(|e| e.to_string())?;
    let pid = profile_id.ok_or_else(|| "Profile ID is required".to_string())?;
    let p = crate::db::profile::get(&conn, pid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Profile not found".to_string())?;

    if p.profile_type == "public_iptv" {
        if let Some(sid) = url.split('/').next_back().and_then(|s| s.split('.').next()).and_then(|s| s.parse::<i64>().ok()) {
            let mut stmt = conn.prepare("SELECT url FROM live_streams WHERE profile_id = ?1 AND stream_id = ?2").map_err(|e| e.to_string())?;
            let db_url: Option<String> = stmt.query_row(rusqlite::params![pid, sid], |row| row.get(0)).map_err(|e| e.to_string())?;
            return db_url.ok_or_else(|| "Public IPTV stream URL not found".to_string());
        }
        return Err("Invalid stream URL format".to_string());
    }

    Ok(url.replace("***", &p.password))
}

#[cfg(not(target_os = "android"))]
fn spawn_dispatcher(cmd: &mut std::process::Command, label: &str) -> Result<(), String> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let mut child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", label, e))?;

    // Short-lived OS dispatchers (e.g. xdg-open, open, cmd /c start) often exit
    // almost immediately with a non-zero status when no handler is available.
    let deadline = Instant::now() + Duration::from_millis(250);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }
                let mut stderr_buf = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut stderr_buf);
                }
                let detail = stderr_buf.trim();
                if detail.is_empty() {
                    return Err(format!("{} exited with status {}", label, status));
                }
                return Err(format!("{} failed ({}): {}", label, status, detail));
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(e) => {
                return Err(format!("Failed to check {} process status: {}", label, e));
            }
        }
    }
}

#[cfg(not(target_os = "android"))]
fn launch_desktop_player(configured_player_path: &str, final_url: &str) -> Result<(), String> {
    use std::process::Command;

    let trimmed_path = configured_player_path.trim();

    // If the user explicitly configured a player path, surface its exact launch error
    // instead of silently falling back to a system dispatcher that masks path issues.
    if !trimmed_path.is_empty() {
        return Command::new(trimmed_path)
            .arg(final_url)
            .spawn()
            .map(|_| ())
            .map_err(|e| {
                tracing::error!(
                    player_path = %trimmed_path,
                    error = %e,
                    "Failed to launch configured media player"
                );
                format!("Failed to launch media player '{}': {}", trimmed_path, e)
            });
    }

    // No custom player configured: try platform defaults and system handlers.
    #[cfg(target_os = "windows")]
    {
        let default_paths = [
            "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
            "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
        ];
        for path in default_paths {
            if std::path::Path::new(path).exists() {
                if Command::new(path).arg(final_url).spawn().is_ok() {
                    return Ok(());
                }
            }
        }

        let mut cmd = Command::new("cmd");
        cmd.args(["/c", "start", "", final_url]);
        if let Err(e) = spawn_dispatcher(&mut cmd, "cmd /c start") {
            return Err(format!(
                "No media player configured and Windows default handler failed: {}",
                e
            ));
        }
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        if Command::new("vlc").arg(final_url).spawn().is_ok() {
            return Ok(());
        }

        let mut cmd = Command::new("xdg-open");
        cmd.arg(final_url);
        return spawn_dispatcher(&mut cmd, "xdg-open").map_err(|e| {
            format!(
                "No media player configured, 'vlc' was not found in PATH, and system handler failed ({}). Set a valid player path in Settings.",
                e
            )
        });
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        cmd.arg(final_url);
        return spawn_dispatcher(&mut cmd, "open").map_err(|e| {
            format!(
                "No media player configured and macOS default handler failed ({}). Set a valid player path in Settings.",
                e
            )
        });
    }

    #[allow(unreachable_code)]
    Err("Failed to launch media player or default system handler.".to_string())
}

#[tauri::command]
pub async fn launch_player(
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
        let (final_url, player_path) = {
            let conn = state.read().map_err(|e| e.to_string())?;
            let pid = profile_id.ok_or_else(|| "Profile ID is required".to_string())?;
            let p = crate::db::profile::get(&conn, pid)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Profile not found".to_string())?;

            let final_url = if p.profile_type == "public_iptv" {
                if let Some(sid) = url.split('/').next_back().and_then(|s| s.split('.').next()).and_then(|s| s.parse::<i64>().ok()) {
                    let mut stmt = conn.prepare("SELECT url FROM live_streams WHERE profile_id = ?1 AND stream_id = ?2").map_err(|e| e.to_string())?;
                    let db_url: Option<String> = stmt.query_row(rusqlite::params![pid, sid], |row| row.get(0)).map_err(|e| e.to_string())?;
                    db_url.ok_or_else(|| "Public IPTV stream URL not found".to_string())?
                } else {
                    return Err("Invalid stream URL format".to_string());
                }
            } else {
                url.replace("***", &p.password)
            };

            let player_path = crate::db::settings::get(&conn, "player_windows")
                .map_err(|e| e.to_string())?
                .unwrap_or_default();

            (final_url, player_path)
        };

        tokio::task::spawn_blocking(move || launch_desktop_player(&player_path, &final_url))
            .await
            .map_err(|e| format!("Failed to run player launch task: {}", e))?
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
    let final_url = {
        let conn = state.read().map_err(|e| e.to_string())?;
        let pid = profile_id.ok_or_else(|| "Profile ID is required".to_string())?;
        let p = crate::db::profile::get(&conn, pid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Profile not found".to_string())?;

        if p.profile_type == "public_iptv" {
            if let Some(sid) = url.split('/').next_back().and_then(|s| s.split('.').next()).and_then(|s| s.parse::<i64>().ok()) {
                let mut stmt = conn.prepare("SELECT url FROM live_streams WHERE profile_id = ?1 AND stream_id = ?2").map_err(|e| e.to_string())?;
                let db_url: Option<String> = stmt.query_row(rusqlite::params![pid, sid], |row| row.get(0)).map_err(|e| e.to_string())?;
                db_url.ok_or_else(|| "Public IPTV stream URL not found".to_string())?
            } else {
                return Err("Invalid stream URL format".to_string());
            }
        } else {
            url.replace("***", &p.password)
        }
    };

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

#[cfg(all(test, not(target_os = "android")))]
mod tests {
    use super::*;

    #[test]
    fn test_configured_missing_player_returns_error() {
        let missing = "/nonexistent/path/to/custom-player-binary";
        let err = launch_desktop_player(missing, "http://example.com/stream.ts")
            .expect_err("Missing configured player path must return an error instead of falling back");
        assert!(
            err.contains(missing),
            "Expected error message to include configured path, got: {}",
            err
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_spawn_dispatcher_reports_immediate_nonzero_exit() {
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", "echo 'no handler' >&2; exit 3"]);
        let err = spawn_dispatcher(&mut cmd, "xdg-open")
            .expect_err("Non-zero dispatcher exit must return an error");
        assert!(
            err.contains("xdg-open") && err.contains("no handler"),
            "Expected dispatcher label and stderr in error, got: {}",
            err
        );
    }
}
