use serde::{Deserialize, Serialize};
use super::client::deserialize_option_string;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryApi {
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub category_id: Option<String>,
    pub category_name: String,
    #[serde(default)]
    pub profile_id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_api_deserializes_numeric_and_string_ids() {
        let numeric: CategoryApi = serde_json::from_str(
            r#"{"category_id": 42, "category_name": "Sports"}"#,
        )
        .unwrap();
        assert_eq!(numeric.category_id.as_deref(), Some("42"));
        assert_eq!(numeric.category_name, "Sports");
        assert_eq!(numeric.profile_id, None);

        let empty: CategoryApi = serde_json::from_str(
            r#"{"category_id": "", "category_name": "Other"}"#,
        )
        .unwrap();
        assert_eq!(empty.category_id, None);
    }
}
