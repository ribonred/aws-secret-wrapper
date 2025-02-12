#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use serde_json::json;
    use std::env;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_secret_parsing() {
        // Mock secret JSON string
        let secret_string = json!({
            "DB_HOST": "localhost",
            "DB_PORT": "5432",
            "API_KEY": "test-key"
        })
        .to_string();

        // Parse secret string
        let secrets: serde_json::Value = serde_json::from_str(&secret_string).unwrap();

        // Test JSON parsing
        assert!(secrets.is_object());
        let obj = secrets.as_object().unwrap();
        assert_eq!(obj.get("DB_HOST").unwrap().as_str().unwrap(), "localhost");
        assert_eq!(obj.get("DB_PORT").unwrap().as_str().unwrap(), "5432");
        assert_eq!(obj.get("API_KEY").unwrap().as_str().unwrap(), "test-key");
    }

    #[tokio::test]
    async fn test_env_variable_setting() {
        // Mock secret JSON
        let secret_string = json!({
            "TEST_VAR_1": "value1",
            "TEST_VAR_2": "value2"
        })
        .to_string();

        let secrets: serde_json::Value = serde_json::from_str(&secret_string).unwrap();

        // Set environment variables
        for (key, value) in secrets.as_object().unwrap() {
            if let Some(value_str) = value.as_str() {
                env::set_var(key, value_str);
            }
        }

        // Verify environment variables were set
        assert_eq!(env::var("TEST_VAR_1").unwrap(), "value1");
        assert_eq!(env::var("TEST_VAR_2").unwrap(), "value2");
    }

    #[test]
    fn test_config_loading() {
        let config_content = r#"
aws_access_key: "test-access-key"
aws_secret_key: "test-secret-key"
aws_region: "us-east-1"
"#;
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, config_content).unwrap();

        // Update the CONFIG constant temporarily for testing
        let config_str = fs::read_to_string(&temp_file).unwrap();
        let settings: Settings = serde_yaml::from_str(&config_str).unwrap();

        // Verify settings
        assert_eq!(settings.aws_access_key, "test-access-key");
        assert_eq!(settings.aws_secret_key, "test-secret-key");
        assert_eq!(settings.aws_region, "us-east-1");
    }

    #[tokio::test]
    async fn test_invalid_secret_format() {
        let invalid_secret = "not a json";
        let result = serde_json::from_str::<serde_json::Value>(invalid_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_config() {
        // Test missing aws_access_key
        let config_content = r#"
    aws_secret_key: "test-secret-key"
    aws_region: "us-east-1"
    "#;
        let result = serde_yaml::from_str::<Settings>(config_content);
        assert!(result.is_err());

        // Test missing aws_secret_key
        let config_content = r#"
    aws_access_key: "test-access-key"
    aws_region: "us-east-1"
    "#;
        let result = serde_yaml::from_str::<Settings>(config_content);
        assert!(result.is_err());

        // Test missing aws_region
        let config_content = r#"
    aws_access_key: "test-access-key"
    aws_secret_key: "test-secret-key"
    "#;
        let result = serde_yaml::from_str::<Settings>(config_content);
        assert!(result.is_err());
    }
}
