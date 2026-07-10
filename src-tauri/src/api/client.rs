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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ClientRegistry {
    pub clients: Mutex<HashMap<i64, Arc<XtreamClient>>>,
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
}
