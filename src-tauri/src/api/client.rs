use std::time::Duration;
use reqwest::Client as HttpClient;
use anyhow::{Result, Context};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};

#[derive(Debug, Clone, Default)]
pub struct XtreamConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

pub struct XtreamClient {
    http: HttpClient,
    config: std::sync::RwLock<XtreamConfig>,
    last_request_time: tokio::sync::Mutex<Option<std::time::Instant>>,
    last_short_epg_time: tokio::sync::Mutex<Option<std::time::Instant>>,
}

impl XtreamClient {
    pub fn new(server_url: String, username: String, password: String) -> Self {
        let http = HttpClient::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(120))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| HttpClient::new());
        Self {
            http,
            config: std::sync::RwLock::new(XtreamConfig {
                server_url,
                username,
                password,
            }),
            last_request_time: tokio::sync::Mutex::new(None),
            last_short_epg_time: tokio::sync::Mutex::new(None),
        }
    }

    pub fn update_credentials(&self, server_url: String, username: String, password: String) {
        let mut guard = self.config.write().unwrap();
        guard.server_url = server_url;
        guard.username = username;
        guard.password = password;
        tracing::info!("XtreamClient credentials updated in-memory");
    }

    pub fn has_credentials(&self) -> bool {
        let config = self.config.read().unwrap();
        !config.server_url.trim().is_empty() 
            && !config.username.trim().is_empty() 
            && !config.password.trim().is_empty()
    }

    pub fn get_url(&self, action: Option<&str>) -> String {
        let config = self.config.read().unwrap();
        let mut base = format!(
            "{}/player_api.php?username={}&password={}",
            config.server_url.trim_end_matches('/'),
            config.username,
            config.password
        );
        if let Some(act) = action {
            base.push_str(&format!("&action={}", act));
        }
        base
    }

    pub async fn fetch<T: DeserializeOwned>(&self, action: Option<&str>) -> Result<T> {
        let is_short_epg = action.map(|a| a.starts_with("get_short_epg")).unwrap_or(false);

        if is_short_epg {
            let mut guard = self.last_short_epg_time.lock().await;
            let wait_time = guard.map(|last_time| {
                let elapsed = last_time.elapsed();
                let min_delay = std::time::Duration::from_millis(500);
                if elapsed < min_delay {
                    min_delay - elapsed
                } else {
                    std::time::Duration::ZERO
                }
            }).unwrap_or(std::time::Duration::ZERO);

            if !wait_time.is_zero() {
                tracing::debug!("Rate limiting: sleeping for {:?} before next request", wait_time);
                tokio::time::sleep(wait_time).await;
            }
            *guard = Some(std::time::Instant::now());
        } else {
            let mut guard = self.last_request_time.lock().await;
            let wait_time = guard.map(|last_time| {
                let elapsed = last_time.elapsed();
                let min_delay = std::time::Duration::from_secs(2);
                if elapsed < min_delay {
                    min_delay - elapsed
                } else {
                    std::time::Duration::ZERO
                }
            }).unwrap_or(std::time::Duration::ZERO);

            if !wait_time.is_zero() {
                tracing::debug!("Rate limiting: sleeping for {:?} before next request", wait_time);
                tokio::time::sleep(wait_time).await;
            }
            *guard = Some(std::time::Instant::now());
        };

        let url = self.get_url(action);
        tracing::debug!(url = %url, "Fetching Xtream API");
        let res = self.http.get(&url).send().await.context("HTTP request failed")?;
        let status = res.status();
        if !status.is_success() {
            anyhow::bail!("HTTP request returned status code {}", status);
        }
        
        let text = res.text().await.context("Failed to read response body as text")?;
        let body = match serde_json::from_str::<T>(&text) {
            Ok(b) => b,
            Err(e) => {
                let action_str = action.unwrap_or("none").replace(['&', '='], "_");
                let log_dir = if let Ok(home) = std::env::var("HOME") {
                    format!("{}/.local/share/com.martinm.kusirik", home)
                } else if let Ok(profile) = std::env::var("USERPROFILE") {
                    format!("{}/AppData/Local/com.martinm.kusirik", profile)
                } else {
                    "/tmp/com.martinm.kusirik".to_string()
                };
                let log_path = format!("{}/failed_{}.txt", log_dir, action_str);
                
                if let Err(dir_err) = std::fs::create_dir_all(&log_dir) {
                    tracing::error!("Failed to create debug log directory {}: {}", log_dir, dir_err);
                }
                
                if let Err(write_err) = std::fs::write(&log_path, &text) {
                    tracing::error!("Failed to write debug log to {}: {}", log_path, write_err);
                } else {
                    tracing::warn!("Wrote failed raw response to {}", log_path);
                }
                
                anyhow::bail!("Failed to deserialize response JSON: {} (wrote raw body to {})", e, log_path);
            }
        };
        Ok(body)
    }
}

/// A custom deserializer helper that reads fields that might be strings, numbers, or null,
/// and coerces them into an Option<String>.
pub fn deserialize_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper {
        String(String),
        Float(f64),
        Int(i64),
        Null,
    }

    match Helper::deserialize(deserializer)? {
        Helper::String(s) => {
            if s.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(s))
            }
        }
        Helper::Float(f) => Ok(Some(f.to_string())),
        Helper::Int(i) => Ok(Some(i.to_string())),
        Helper::Null => Ok(None),
    }
}

/// A custom deserializer helper that reads fields that might be strings, numbers, or null,
/// and coerces them into an Option<i32>.
pub fn deserialize_option_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper {
        String(String),
        Float(f64),
        Int(i64),
        Null,
    }

    match Helper::deserialize(deserializer)? {
        Helper::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i32>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        Helper::Float(f) => Ok(Some(f as i32)),
        Helper::Int(i) => Ok(Some(i as i32)),
        Helper::Null => Ok(None),
    }
}

/// Parses a date or timestamp string into Unix Epoch seconds.
/// Supports ISO dates ("YYYY-MM-DD"), 4-digit years ("YYYY" -> Jan 1), and numeric epoch strings.
pub fn parse_date_or_timestamp(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // 1. Try ISO date "YYYY-MM-DD" (e.g. "2024-09-19")
    if let Ok(ndt) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        if let Some(dt) = ndt.and_hms_opt(0, 0, 0) {
            return Some(dt.and_utc().timestamp());
        }
    }
    // 2. Try 4-digit year "YYYY" (e.g. "2024")
    if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(year) = s.parse::<i32>() {
            if let Some(ndt) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) {
                if let Some(dt) = ndt.and_hms_opt(0, 0, 0) {
                    return Some(dt.and_utc().timestamp());
                }
            }
        }
    }
    // 3. Try direct Unix epoch timestamp string (e.g. "1608130741")
    if let Ok(ts) = s.parse::<i64>() {
        return Some(ts);
    }
    None
}

/// A custom deserializer helper that reads fields that might be strings, numbers, or null,
/// and coerces them into an Option<i64> (Unix Epoch seconds).
pub fn deserialize_option_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper {
        String(String),
        Float(f64),
        Int(i64),
        Null,
    }

    match Helper::deserialize(deserializer)? {
        Helper::String(s) => Ok(parse_date_or_timestamp(&s)),
        Helper::Float(f) => Ok(Some(f as i64)),
        Helper::Int(i) => Ok(Some(i)),
        Helper::Null => Ok(None),
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ClientRegistry {
    pub clients: Mutex<HashMap<i64, Arc<XtreamClient>>>,
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_create(&self, profile_id: i64, conn: &rusqlite::Connection) -> Result<Arc<XtreamClient>, String> {
        let mut map = self.clients.lock().map_err(|e| e.to_string())?;
        if let Some(client) = map.get(&profile_id) {
            return Ok(Arc::clone(client));
        }

        // Load profile from DB
        let p = crate::db::profile::get(conn, profile_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Profile not found".to_string())?;

        let client = Arc::new(XtreamClient::new(p.server_url, p.username, p.password));
        map.insert(profile_id, Arc::clone(&client));
        Ok(client)
    }

    pub fn update(&self, profile_id: i64, server_url: String, username: String, password: String) {
        if let Ok(mut map) = self.clients.lock() {
            if let Some(client) = map.get(&profile_id) {
                client.update_credentials(server_url, username, password);
            } else {
                let client = Arc::new(XtreamClient::new(server_url, username, password));
                map.insert(profile_id, client);
            }
        }
    }

    pub fn remove(&self, profile_id: i64) {
        if let Ok(mut map) = self.clients.lock() {
            map.remove(&profile_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Dummy {
        #[serde(deserialize_with = "deserialize_option_string")]
        val: Option<String>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct DummyI64 {
        #[serde(default, deserialize_with = "deserialize_option_i64")]
        val: Option<i64>,
    }

    #[test]
    fn test_deserialize_option_string() {
        let cases = vec![
            (r#"{"val": 4.8}"#, Some("4.8".to_string())),
            (r#"{"val": "6.2"}"#, Some("6.2".to_string())),
            (r#"{"val": 7}"#, Some("7".to_string())),
            (r#"{"val": ""}"#, None),
            (r#"{"val": "   "}"#, None),
            (r#"{"val": null}"#, None),
        ];

        for (json_str, expected) in cases {
            let parsed: Dummy = serde_json::from_str(json_str).unwrap();
            assert_eq!(parsed.val, expected, "Failed for JSON: {}", json_str);
        }
    }

    #[test]
    fn test_deserialize_option_i64() {
        let cases = vec![
            (r#"{"val": 1608130741}"#, Some(1608130741)),
            (r#"{"val": "1608130741"}"#, Some(1608130741)),
            (r#"{"val": "2024-09-19"}"#, Some(1726704000)),
            (r#"{"val": "2020"}"#, Some(1577836800)),
            (r#"{"val": ""}"#, None),
            (r#"{"val": null}"#, None),
        ];

        for (json_str, expected) in cases {
            let parsed: DummyI64 = serde_json::from_str(json_str).unwrap();
            assert_eq!(parsed.val, expected, "Failed for JSON: {}", json_str);
        }
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct DummyI32 {
        #[serde(default, deserialize_with = "deserialize_option_i32")]
        val: Option<i32>,
    }

    #[test]
    fn test_deserialize_option_i32() {
        let cases = vec![
            (r#"{"val": 1}"#, Some(1)),
            (r#"{"val": 7.9}"#, Some(7)),
            (r#"{"val": "42"}"#, Some(42)),
            (r#"{"val": "  15  "}"#, Some(15)),
            (r#"{"val": ""}"#, None),
            (r#"{"val": "   "}"#, None),
            (r#"{"val": null}"#, None),
            (r#"{}"#, None),
        ];

        for (json_str, expected) in cases {
            let parsed: DummyI32 = serde_json::from_str(json_str).unwrap();
            assert_eq!(parsed.val, expected, "Failed for JSON: {}", json_str);
        }

        let err = serde_json::from_str::<DummyI32>(r#"{"val": "not-a-number"}"#);
        assert!(err.is_err());
    }

    #[test]
    fn test_parse_date_or_timestamp_edge_cases() {
        assert_eq!(parse_date_or_timestamp("2026-11-20"), Some(1795132800));
        assert_eq!(parse_date_or_timestamp("  2026  "), Some(1767225600));
        assert_eq!(parse_date_or_timestamp("1700000000"), Some(1700000000));
        assert_eq!(parse_date_or_timestamp(""), None);
        assert_eq!(parse_date_or_timestamp("   "), None);
        assert_eq!(parse_date_or_timestamp("invalid-date"), None);
        assert_eq!(parse_date_or_timestamp("2026-13-45"), None);
    }

    #[test]
    fn test_xtream_client_url_and_credentials() {
        let client = XtreamClient::new(
            "http://example.com:8080///".to_string(),
            "alice".to_string(),
            "secret".to_string(),
        );
        assert!(client.has_credentials());
        assert_eq!(
            client.get_url(None),
            "http://example.com:8080/player_api.php?username=alice&password=secret"
        );
        assert_eq!(
            client.get_url(Some("get_vod_streams")),
            "http://example.com:8080/player_api.php?username=alice&password=secret&action=get_vod_streams"
        );

        client.update_credentials("https://new.host".to_string(), "bob".to_string(), "  ".to_string());
        assert!(!client.has_credentials());
        assert_eq!(
            client.get_url(Some("get_series")),
            "https://new.host/player_api.php?username=bob&password=  &action=get_series"
        );
    }

    #[test]
    fn test_client_registry_lifecycle() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();

        let profile = crate::db::profile::Profile {
            id: 0,
            name: "Main".to_string(),
            server_url: "http://iptv.local".to_string(),
            username: "u1".to_string(),
            password: "p1".to_string(),
            epg_mode: "xmltv".to_string(),
            created_at: 1700000000,
            profile_type: "xtream".to_string(),
        };
        let pid = crate::db::profile::insert(&conn, &profile).unwrap();

        let registry = ClientRegistry::new();
        assert!(registry.get_or_create(999, &conn).is_err());

        let c1 = registry.get_or_create(pid, &conn).unwrap();
        assert_eq!(c1.get_url(None), "http://iptv.local/player_api.php?username=u1&password=p1");

        // Updating in-place reflects on existing Arc<XtreamClient>
        registry.update(pid, "http://iptv2.local".to_string(), "u2".to_string(), "p2".to_string());
        assert_eq!(c1.get_url(None), "http://iptv2.local/player_api.php?username=u2&password=p2");

        // Removing clears cached entry so get_or_create reloads from DB
        registry.remove(pid);
        let c2 = registry.get_or_create(pid, &conn).unwrap();
        assert_eq!(c2.get_url(None), "http://iptv.local/player_api.php?username=u1&password=p1");
    }
}
