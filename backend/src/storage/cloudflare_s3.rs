use aws_sdk_s3 as s3;
use aws_smithy_types::date_time::Format::DateTime;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct CloudflareConfig {
    account_id: String,
    access_key_id: String,
    access_key_secret: String,
}

#[derive(Deserialize)]
struct Config {
    cloudflare: CloudflareConfig,
}

#[derive(Debug)]
pub enum StorageError {
    BucketNotFound(String),
    CredentialsInvalid,
    NetworkError(String),
    ConfigurationError(String),
}

fn validate_config(config: &CloudflareConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.account_id.is_empty() || config.access_key_id.is_empty() || config.access_key_secret.is_empty() {
        return Err("Missing required Cloudflare credentials".into());
    }
    Ok(())
}

/// Load configuration from TOML file
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("src/conf/init.toml")?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

/// Create S3 client with Cloudflare R2 configuration
async fn create_s3_client() -> Result<s3::Client, Box<dyn std::error::Error>> {
    let app_config = load_config()?;

    let account_id = &app_config.cloudflare.account_id;
    let access_key_id = &app_config.cloudflare.access_key_id;
    let access_key_secret = &app_config.cloudflare.access_key_secret;
    // print the account id and access key id
    println!("account_id: {}, access_key_id: {}", account_id, access_key_id);

    // Configure the client
    let aws_config = aws_config::from_env()
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            access_key_id.clone(),
            access_key_secret.clone(),
            None, // session token is not used with R2
            None,
            "R2",
        ))
        .region("auto")
        .load()
        .await;

    Ok(s3::Client::new(&aws_config))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bucket_name = "sdk-example";

    let client = create_s3_client().await?;

    // List buckets
    let list_buckets_output = client.list_buckets().send().await?;

    println!("Buckets:");
    for bucket in list_buckets_output.buckets() {
        println!("  - {}: {}",
            bucket.name().unwrap_or_default(),
            bucket.creation_date().map_or_else(
                || "Unknown creation date".to_string(),
                |date| date.fmt(DateTime).unwrap()
            )
        );
    }

    // List objects in a specific bucket
    let list_objects_output = client
        .list_objects_v2()
        .bucket(bucket_name)
        .send()
        .await?;

    println!("\nObjects in {}:", bucket_name);
    for object in list_objects_output.contents() {
        println!("  - {}: {} bytes, last modified: {}",
            object.key().unwrap_or_default(),
            object.size().unwrap_or_default(),
            object.last_modified().map_or_else(
                || "Unknown".to_string(),
                |date| date.fmt(DateTime).unwrap()
            )
        );
    }

    Ok(())
}