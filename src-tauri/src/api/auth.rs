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
