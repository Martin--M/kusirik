use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::{DateTime, Utc, NaiveDateTime};
use super::client::XtreamClient;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpgProgramApi {
    pub id: Option<String>,
    pub title: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub description: Option<String>,
    pub channel_id: Option<String>,
    pub start_timestamp: Option<String>,
    pub stop_timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EpgListingResponse {
    pub epg_listings: Option<Vec<EpgProgramApi>>,
}

/// Fetches short EPG entries for a specific stream ID.
pub async fn fetch_short_epg(client: &XtreamClient, stream_id: i64) -> Result<Vec<EpgProgramApi>> {
    let action = format!("get_short_epg&stream_id={}", stream_id);
    let res: EpgListingResponse = client.fetch(Some(&action)).await?;
    Ok(res.epg_listings.unwrap_or_default())
}

/// Decodes base64 strings if the string is valid Base64 and decodes to valid printable UTF-8.
pub fn decode_base64(input: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;
    
    for c in input.chars() {
        if c == '=' {
            break;
        }
        let val = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' => 62,
            '/' => 63,
            _ => continue, // Skip whitespaces
        };
        buffer = (buffer << 6) | val;
        bits_collected += 6;
        if bits_collected >= 8 {
            bits_collected -= 8;
            bytes.push((buffer >> bits_collected) as u8);
        }
    }
    String::from_utf8(bytes).ok()
}

/// Checks if a string has no spaces, looks like base64, and decodes it.
/// Otherwise returns the original string.
pub fn maybe_decode_base64(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return input.to_string();
    }
    // Base64 strings from Xtream Codes do not contain spaces and only contain valid base64 characters.
    let is_base64_charset = trimmed.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
    });
    if is_base64_charset && trimmed.len() >= 4 {
        if let Some(decoded) = decode_base64(trimmed) {
            // Check if decoded contains readable ASCII/UTF-8 printables (not control codes)
            let is_printable = decoded.chars().all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t');
            if is_printable && !decoded.is_empty() {
                return decoded;
            }
        }
    }
    input.to_string()
}

/// Parses a naive local datetime string from the API (YYYY-MM-DD HH:MM:SS)
/// into a UTC ISO 8601 string. If parsing fails, returns the original.
pub fn parse_api_date_to_utc(s: &str) -> Option<String> {
    // Try standard MySQL format: YYYY-MM-DD HH:MM:SS
    if let Ok(naive) = NaiveDateTime::parse_from_str(s.trim(), "%Y-%m-%d %H:%M:%S") {
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        return Some(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
    // Try XMLTV compact format: YYYYMMDDHHMMSS
    if let Ok(naive) = NaiveDateTime::parse_from_str(s.trim(), "%Y%m%d%H%M%S") {
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        return Some(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
    None
}
