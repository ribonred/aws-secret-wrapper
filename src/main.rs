mod config;
#[cfg(test)]
mod tests;

use crate::config::Settings;
use anyhow::{Error, Result};
use async_trait::async_trait;
use aws_config::Region;
use aws_sdk_secretsmanager::config::Credentials;
use aws_sdk_secretsmanager::{Client, Config};
use clap::Parser;
use colored::Colorize;
use std::process::Command;

fn print_banner() {
    println!(
        "{}",
        r#"
 ___  ___  ___ _ __ ___| |_    __      ___ __ __ _ _ __  _ __   ___ _ __ 
/ __|/ _ \/ __| '__/ _ \ __|   \ \ /\ / / '__/ _` | '_ \| '_ \ / _ \ '__|
\__ \  __/ (__| | |  __/ |_     \ V  V /| | | (_| | |_) | |_) |  __/ |   
|___/\___|\___|_|  \___|\__|     \_/\_/ |_|  \__,_| .__/| .__/ \___|_|   
                                                   |_|   |_|              
"#
        .cyan()
        .bold()
    );
}

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
            .ok_or_else(|| anyhow::anyhow!("Secret string is empty".red()))?;

        let secrets: serde_json::Value = serde_json::from_str(&secret_string)?;
        Ok(secrets)
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(long, required_unless_present = "sf")]
    secret_id: Option<String>,
    #[arg(long, required_unless_present = "secret_id")]
    sf: Option<String>,
    #[arg(last = true)]
    command: Vec<String>,
    #[arg(long)]
    /// if supplied set or change the AWS region
    region: Option<String>,
    #[arg(long, default_value_t = false)]
    // print fancy banner and available secret keys
    fancy: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut settings = Settings::new()?;
    let cli = Cli::parse();
    if cli.fancy {
        print_banner();
    }
    if let Some(region) = cli.region {
        settings.aws_region = region;
    }
    let getter = AwsSecretGetter::new(settings).await?;

    let secret_ids = {
        if let Some(secret_ids) = &cli.secret_id {
            let ids: Vec<String> = secret_ids.split(',').map(|s| s.to_string()).collect();
            ids
        } else if let Some(secret_file) = &cli.sf {
            let ids = std::fs::read_to_string(secret_file)
                .expect(format!("Could not read file {}", secret_file).as_str())
                .lines()
                .map(|s| s.to_string())
                .collect();
            ids
        } else {
            return Err(Error::msg("No secret id provided"));
        }
    };
    for secret_id in secret_ids {
        let secrets = getter.get_secrets(&secret_id).await?;
        if cli.fancy {
            println!(
                "{}{}{}",
                "[".bold().bright_white(),
                secret_id.to_uppercase().bright_green(),
                "]".bold().bright_white()
            );
        }
        // Set environment variables from secrets
        for (key, value) in secrets
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Secret is not a JSON object".red()))?
        {
            if let Some(value_str) = value.as_str() {
                if cli.fancy {
                    println!("{}={}", key.bright_magenta(), "****".red());
                }
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
