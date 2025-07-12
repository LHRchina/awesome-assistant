pub mod cloudflare_s3;

use aws_sdk_s3 as s3;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;
use aws_smithy_types::byte_stream::ByteStream;
use chrono::{DateTime, Utc};

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

#[derive(Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub upload_time: DateTime<Utc>,
    pub s3_key: String,
}

pub struct CloudflareStorage {
    client: s3::Client,
    bucket_name: String,
}

impl CloudflareStorage {
    /// Create a new CloudflareStorage instance
    pub async fn new(bucket_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Self::create_s3_client().await?;
        Ok(Self {
            client,
            bucket_name,
        })
    }

    /// Load configuration from TOML file
    fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string("src/conf/init.toml")?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }

    /// Create S3 client with Cloudflare R2 configuration
    async fn create_s3_client() -> Result<s3::Client, Box<dyn std::error::Error>> {
        let app_config = Self::load_config()?;
        
        let account_id = &app_config.cloudflare.account_id;
        let access_key_id = &app_config.cloudflare.access_key_id;
        let access_key_secret = &app_config.cloudflare.access_key_secret;

        // Configure the client
        let aws_config = aws_config::from_env()
            .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
            .credentials_provider(s3::config::Credentials::new(
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

    /// Upload a file to Cloudflare R2
    pub async fn upload_file(
        &self,
        filename: &str,
        content: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<FileMetadata, Box<dyn std::error::Error>> {
        let file_id = Uuid::new_v4().to_string();
        let file_extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{}", ext))
            .unwrap_or_default();
        
        let s3_key = format!("{}{}", file_id, file_extension);
        let content_size = content.len() as u64;
        
        let mut put_object = self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&s3_key)
            .body(ByteStream::from(content));

        if let Some(ct) = &content_type {
            put_object = put_object.content_type(ct);
        }

        put_object.send().await?;

        let metadata = FileMetadata {
            id: file_id,
            filename: filename.to_string(),
            size: content_size,
            content_type,
            upload_time: chrono::Utc::now(),
            s3_key,
        };

        Ok(metadata)
    }

    /// Download a file from Cloudflare R2
    pub async fn download_file(
        &self,
        s3_key: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(s3_key)
            .send()
            .await?;

        let data = response.body.collect().await?;
        Ok(data.into_bytes().to_vec())
    }

    /// Delete a file from Cloudflare R2
    pub async fn delete_file(
        &self,
        s3_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(s3_key)
            .send()
            .await?;

        Ok(())
    }

    /// List all files in the bucket
    pub async fn list_files(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self.client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .send()
            .await?;

        let keys = response
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|k| k.to_string()))
            .collect();

        Ok(keys)
    }
}