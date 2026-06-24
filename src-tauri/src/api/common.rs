use serde::{Deserialize, Serialize};
use super::client::deserialize_option_string;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryApi {
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub category_id: Option<String>,
    pub category_name: String,
}
