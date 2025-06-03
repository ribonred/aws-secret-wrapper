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
use std::fs;
use std::path::Path;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;

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
    async fn set_secret_cache(&self, secret_id: &str, content: &str);
    async fn get_secret_cache(&self, secret_id: &str) -> Result<serde_json::Value>;
}
struct AwsSecretGetter {
    client: Client,
    encryption_key: [u8; 32],
}

const CACHE_DIR: &str = "/tmp/wrapper/cache";
const KEY_FILE: &str = "/tmp/wrapper/encryption.key";

impl AwsSecretGetter {
    fn ensure_directories() -> Result<()> {
        let wrapper_dir = "/tmp/wrapper";
        if !Path::new(wrapper_dir).exists() {
            fs::create_dir_all(wrapper_dir)?;
        }
        if !Path::new(CACHE_DIR).exists() {
            fs::create_dir_all(CACHE_DIR)?;
        }
        Ok(())
    }

    fn load_or_create_key() -> Result<[u8; 32]> {
        Self::ensure_directories()?;
        
        if Path::new(KEY_FILE).exists() {
            // Load existing key
            let key_data = fs::read(KEY_FILE)?;
            if key_data.len() != 32 {
                return Err(anyhow::anyhow!("Invalid key file length"));
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&key_data);
            Ok(key)
        } else {
            // Create new key
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            fs::write(KEY_FILE, &key)?;
            Ok(key)
        }
    }

    fn clear_cache() -> Result<()> {
        if Path::new(CACHE_DIR).exists() {
            for entry in fs::read_dir(CACHE_DIR)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        
        // Also remove the encryption key for better security
        if Path::new(KEY_FILE).exists() {
            fs::remove_file(KEY_FILE)?;
        }
        
        Ok(())
    }

    fn encrypt_content(&self, content: &str) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_data = cipher.encrypt(nonce, content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Combine nonce + encrypted data and encode as base64
        let mut combined = Vec::with_capacity(12 + encrypted_data.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&encrypted_data);
        
        Ok(general_purpose::STANDARD.encode(&combined))
    }

    fn decrypt_content(&self, encrypted_b64: &str) -> Result<String> {
        let combined = general_purpose::STANDARD.decode(encrypted_b64)?;
        
        if combined.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }
        
        let (nonce_bytes, encrypted_data) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        
        let decrypted_data = cipher.decrypt(nonce, encrypted_data)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        Ok(String::from_utf8(decrypted_data)?)
    }
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

        let encryption_key = Self::load_or_create_key()?;

        Ok(Self {
            client: Client::from_conf(config),
            encryption_key,
        })
    }
    
    async fn set_secret_cache(&self, secret_id: &str, content: &str) {
        let cache_file_path = format!("{}/{}", CACHE_DIR, secret_id);
        print!("Setting cache for {} at {}", secret_id, cache_file_path);
        match self.encrypt_content(content) {
            Ok(encrypted_content) => {
                if let Err(e) = fs::write(&cache_file_path, encrypted_content) {
                    eprintln!("Failed to write cache file {}: {}", cache_file_path, e);
                }
            }
            Err(e) => {
                eprintln!("Failed to encrypt content for {}: {}", secret_id, e);
            }
        }
    }
    
    async fn get_secret_cache(&self, secret_id: &str) -> Result<serde_json::Value> {
        let cache_file_path = format!("{}/{}", CACHE_DIR, secret_id);
        
        if !Path::new(&cache_file_path).exists() {
            return Err(anyhow::anyhow!("Cache file not found for secret_id: {}", secret_id));
        }
        
        let encrypted_content = fs::read_to_string(&cache_file_path)?;
        let decrypted_content = self.decrypt_content(&encrypted_content)?;
        let secrets: serde_json::Value = serde_json::from_str(&decrypted_content)?;
        
        Ok(secrets)
    }
      async fn get_secrets(&self, secret_id: &str) -> Result<serde_json::Value> {
        // First try to get from cache
        if let Ok(cached_secrets) = self.get_secret_cache(secret_id).await {
            println!("using cache for secret_id: {}", secret_id);
            return Ok(cached_secrets);
        }

        // If not in cache, fetch from AWS
        let secret = self
            .client
            .get_secret_value()
            .secret_id(secret_id)
            .send()
            .await?;

        let secret_string = secret
            .secret_string()
            .ok_or_else(|| anyhow::anyhow!("Secret string is empty"))?;

        // Cache the secret
        self.set_secret_cache(secret_id, secret_string).await;

        let secrets: serde_json::Value = serde_json::from_str(&secret_string)?;
        Ok(secrets)
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(long, required_unless_present_any = ["sf", "clear_cache"])]
    secret_id: Option<String>,
    #[arg(long, required_unless_present_any = ["secret_id", "clear_cache"])]
    sf: Option<String>,
    #[arg(last = true, required_unless_present = "clear_cache")]
    command: Vec<String>,
    #[arg(long)]
    /// if supplied set or change the AWS region
    region: Option<String>,
    #[arg(long, default_value_t = false)]
    /// print fancy banner and available secret keys
    fancy: bool,
    #[arg(long, default_value_t = false)]
    /// clear cache directory and encryption key, then exit
    clear_cache: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut settings = Settings::new()?;
    let cli = Cli::parse();
    
    // Handle clear cache option first
    if cli.clear_cache {
        if let Err(e) = AwsSecretGetter::clear_cache() {
            eprintln!("Failed to clear cache: {}", e);
            std::process::exit(1);
        }
        println!("Cache cleared successfully");
        return Ok(());
    }
    
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
            .ok_or_else(|| anyhow::anyhow!("Secret is not a JSON object"))?
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
