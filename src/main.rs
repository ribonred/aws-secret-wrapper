mod config;
#[cfg(test)]
mod tests;

use crate::config::Settings;
use anyhow::Result;
use async_trait::async_trait;
use aws_config::Region;
use aws_sdk_secretsmanager::config::Credentials;
use aws_sdk_secretsmanager::{Client, Config};
use clap::Parser;
use std::process::Command;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SecretGetter {
    async fn new(settings: Settings) -> Result<Self>
    where
        Self: Sized;
    async fn get_secrets(&self, secret_id: &str) -> Result<serde_json::Value>;
}
struct AwsSecretGetter {
    client: Client,
}

#[async_trait]
impl SecretGetter for AwsSecretGetter {
    async fn new(settings: Settings) -> Result<Self> {
        let creds = Credentials::new(
            &settings.aws_access_key,
            &settings.aws_secret_key,
            None,
            None,
            "static-credentials",
        );

        let config = Config::builder()
            .credentials_provider(creds)
            .region(Region::new(settings.aws_region))
            .behavior_version_latest()
            .build();

        Ok(Self {
            client: Client::from_conf(config),
        })
    }
    async fn get_secrets(&self, secret_id: &str) -> Result<serde_json::Value> {
        let secret = self
            .client
            .get_secret_value()
            .secret_id(secret_id)
            .send()
            .await?;

        let secret_string = secret
            .secret_string()
            .ok_or_else(|| anyhow::anyhow!("Secret string is empty"))?;

        let secrets: serde_json::Value = serde_json::from_str(&secret_string)?;
        Ok(secrets)
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    secret_id: String,
    #[arg(last = true)]
    command: Vec<String>,
    #[arg(long)]
     /// if supplied set or change the AWS region
    region: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut settings = Settings::new()?;
    let cli = Cli::parse();
    if let Some(region) = cli.region {
        settings.aws_region = region;
    }
    let getter = AwsSecretGetter::new(settings).await?;
    let secret_ids = if cli.secret_id.contains(',') {
        cli.secret_id.split(',').map(|s| s.to_string()).collect()
    } else {
        vec![cli.secret_id.clone()]
    };
    for secret_id in secret_ids {
        let secrets = getter.get_secrets(&secret_id).await?;

        // Set environment variables from secrets
        for (key, value) in secrets
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Secret is not a JSON object"))?
        {
            if let Some(value_str) = value.as_str() {
                std::env::set_var(key, value_str);
            }
        }
    }

    // Execute the wrapped command
    let status = Command::new(&cli.command[0])
        .args(&cli.command[1..])
        .env_clear()
        .envs(std::env::vars())
        .status()?;

    std::process::exit(status.code().unwrap_or_else(|| {
        eprintln!("Process terminated by signal");
        1
    }));
}
