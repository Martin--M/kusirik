#![allow(unused_variables)]

#[tauri::command]
pub fn launch_player(url: String) -> Result<(), String> {
    // Implemented in P6
    Ok(())
}

#[tauri::command]
pub fn copy_to_clipboard(url: String) -> Result<(), String> {
    // Implemented in P6
    Ok(())
}
