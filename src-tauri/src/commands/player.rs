use tauri::State;
use crate::db::DbConn;

#[tauri::command]
pub fn resolve_stream_url(state: State<'_, DbConn>, url: String) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let password = crate::db::settings::get(&conn, "password")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    Ok(url.replace("***", &password))
}

#[tauri::command]
pub fn launch_player(state: State<'_, DbConn>, url: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let password = crate::db::settings::get(&conn, "password")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let final_url = url.replace("***", &password);

    #[cfg(target_os = "android")]
    {
        Err("On Android, launch_player should be handled via the native IntentPlugin.".to_string())
    }

    #[cfg(not(target_os = "android"))]
    {
        use std::process::Command;

        let player_path = crate::db::settings::get(&conn, "player_windows")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();

        if !player_path.is_empty() && Command::new(&player_path).arg(&final_url).spawn().is_ok() {
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
