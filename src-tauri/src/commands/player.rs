#[tauri::command]
pub fn launch_player(_url: String) -> Result<(), String> {
    // Implemented in P6
    Ok(())
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
