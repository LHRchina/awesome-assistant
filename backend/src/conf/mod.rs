use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct CloudflareConfig {
    pub account_id: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket_name: String,
}

#[derive(Deserialize)]
pub struct PostgresConfig {
    pub postgres_url: String,
}

#[derive(Deserialize)]
pub struct BackendConfig {
    pub jwt_secret: String,
}

#[derive(Deserialize)]
pub struct RedisConfig {
    pub redis_url: String,
    pub token_ttl_seconds: u64,
}

#[derive(Deserialize)]
pub struct Config {
    pub cloudflare: CloudflareConfig,
    pub postgres: PostgresConfig,
    pub backend: BackendConfig,
    pub redis: RedisConfig,
}

/// Load configuration from TOML file
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("conf/init.toml")?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

/// Validate Cloudflare configuration
pub fn validate_config(config: &CloudflareConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.account_id.is_empty() || config.access_key_id.is_empty() || config.access_key_secret.is_empty() {
        return Err("Missing required Cloudflare credentials".into());
    }
    Ok(())
}