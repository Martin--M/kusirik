use serde::{Deserialize, Serialize};
use anyhow::{Result, bail};
use super::client::XtreamClient;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub username: Option<String>,
    pub status: Option<String>,
    #[serde(default, deserialize_with = "super::client::deserialize_option_string")]
    pub exp_date: Option<String>,
    pub auth: Option<i32>,
    pub allowed_output_formats: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub url: Option<String>,
    pub port: Option<String>,
    pub server_protocol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub user_info: Option<UserInfo>,
    pub server_info: Option<ServerInfo>,
}

pub async fn test_connection(client: &XtreamClient) -> Result<AuthResponse> {
    let auth_res: AuthResponse = client.fetch(None).await?;
    if let Some(ref user) = auth_res.user_info {
        if user.auth != Some(1) {
            bail!("Authentication failed: invalid username or password");
        }
        if let Some(ref status) = user.status {
            if status != "Active" {
                bail!("Account status is '{}' (not Active)", status);
            }
        }
    } else {
        bail!("Invalid API response format (missing user_info)");
    }
    Ok(auth_res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response_deserializes_mixed_exp_date_types() {
        let json_str = r#"{
            "user_info": {
                "username": "alice",
                "status": "Active",
                "exp_date": 1800000000,
                "auth": 1,
                "allowed_output_formats": ["m3u8", "ts"]
            },
            "server_info": {
                "url": "iptv.example.com",
                "port": "8080",
                "server_protocol": "http"
            }
        }"#;
        let parsed: AuthResponse = serde_json::from_str(json_str).unwrap();
        let user = parsed.user_info.unwrap();
        assert_eq!(user.username.as_deref(), Some("alice"));
        assert_eq!(user.status.as_deref(), Some("Active"));
        assert_eq!(user.exp_date.as_deref(), Some("1800000000"));
        assert_eq!(user.auth, Some(1));
        assert_eq!(
            user.allowed_output_formats,
            Some(vec!["m3u8".to_string(), "ts".to_string()])
        );
        let server = parsed.server_info.unwrap();
        assert_eq!(server.url.as_deref(), Some("iptv.example.com"));

        // Empty string exp_date normalizes to None
        let null_exp: AuthResponse = serde_json::from_str(r#"{"user_info": {"exp_date": ""}}"#).unwrap();
        assert_eq!(null_exp.user_info.unwrap().exp_date, None);
    }
}
