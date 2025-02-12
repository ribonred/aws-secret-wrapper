use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub aws_access_key: String,
    pub aws_secret_key: String,
    pub aws_region: String,
}

const CONFIG: &str = include_str!("../config.yaml");

impl Settings {
    pub fn new() -> Result<Self> {
        // Check for runtime config file path
        if let Ok(config_path) = std::env::var("CONFIG_FILE") {
            let contents = fs::read_to_string(config_path)?;
            Ok(serde_yaml::from_str(&contents)?)
        } else {
            // Fall back to compile-time config
            Ok(serde_yaml::from_str(CONFIG)?)
        }
    }
}
