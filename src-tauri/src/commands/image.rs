use tauri::State;
use crate::db::DbConn;
use std::time::Duration;
use serde::Serialize;

#[derive(Serialize)]
pub struct ImageData {
    pub bytes: Vec<u8>,
    pub mime: String,
}

#[tauri::command]
pub async fn get_image_data(
    state: State<'_, DbConn>,
    url: String,
) -> Result<ImageData, String> {
    if url.trim().is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    // 1. Check if the image is in database cache
    let cached = {
        let conn = state.read().map_err(|e| e.to_string())?;
        crate::db::image::get_cached_image(&conn, &url).map_err(|e| e.to_string())?
    };

    if let Some((data, content_type)) = cached {
        let mime = content_type.unwrap_or_else(|| "image/jpeg".to_string());
        return Ok(ImageData { bytes: data, mime });
    }

    // 2. Fetch the image from URL using reqwest
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
    
    if !res.status().is_success() {
        return Err(format!("Failed to fetch image, HTTP status: {}", res.status()));
    }

    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let bytes = res.bytes().await.map_err(|e| e.to_string())?.to_vec();

    // 3. Cache the image in database
    {
        let conn = state.writer.lock().map_err(|e| e.to_string())?;
        crate::db::image::insert_cached_image(&conn, &url, &bytes, content_type.as_deref())
            .map_err(|e| e.to_string())?;
    }

    // 4. Return raw bytes and MIME type
    let mime = content_type.unwrap_or_else(|| "image/jpeg".to_string());
    Ok(ImageData { bytes, mime })
}
