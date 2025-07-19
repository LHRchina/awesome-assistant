use redis::{Client, RedisError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub user_id: i64,
    pub email: String,
    pub name: String,
    pub created_at: i64,
    pub expires_at: i64,
}

pub struct RedisTokenStore {
    client: Arc<Mutex<Client>>,
    ttl_seconds: u64,
}

impl RedisTokenStore {
    pub async fn new(redis_url: &str, ttl_seconds: u64) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        
        // Test connection
        let mut conn = client.get_async_connection().await?;
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await?;
        
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            ttl_seconds,
        })
    }

    pub async fn store_token(&self, token: &str, token_info: &TokenInfo) -> Result<(), RedisError> {
        let client = self.client.lock().await.clone();
        let mut conn = client.get_async_connection().await?;
        
        let key = format!("auth:token:{}", token);
        let value = serde_json::to_string(token_info).map_err(|e| {
            RedisError::from((redis::ErrorKind::TypeError, "Serialization error", e.to_string()))
        })?;
        
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.ttl_seconds)
            .arg(&value)
            .query_async::<_, ()>(&mut conn)
            .await?;
        
        Ok(())
    }

    pub async fn get_token_info(&self, token: &str) -> Result<Option<TokenInfo>, RedisError> {
        let client = self.client.lock().await.clone();
        let mut conn = client.get_async_connection().await?;
        
        let key = format!("auth:token:{}", token);
        let result: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async::<_, Option<String>>(&mut conn)
            .await?;
        
        match result {
            Some(value) => {
                let token_info: TokenInfo = serde_json::from_str(&value).map_err(|e| {
                    RedisError::from((redis::ErrorKind::TypeError, "Deserialization error", e.to_string()))
                })?;
                Ok(Some(token_info))
            }
            None => Ok(None),
        }
    }

    pub async fn invalidate_token(&self, token: &str) -> Result<(), RedisError> {
        let client = self.client.lock().await.clone();
        let mut conn = client.get_async_connection().await?;
        
        let key = format!("auth:token:{}", token);
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<_, ()>(&mut conn)
            .await?;
        
        Ok(())
    }

    pub async fn is_token_valid(&self, token: &str) -> Result<bool, RedisError> {
        let info = self.get_token_info(token).await?;
        Ok(info.is_some())
    }

    pub async fn get_all_user_tokens(&self, user_id: i64) -> Result<Vec<String>, RedisError> {
        let client = self.client.lock().await.clone();
        let mut conn = client.get_async_connection().await?;
        
        let pattern = format!("auth:token:*");
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async::<_, Vec<String>>(&mut conn)
            .await?;
        
        let mut user_tokens = Vec::new();
        
        for key in keys {
            if let Some(token_info) = self.get_token_info(&key[12..]).await? {
                if token_info.user_id == user_id {
                    user_tokens.push(key[12..].to_string());
                }
            }
        }
        
        Ok(user_tokens)
    }

    pub async fn invalidate_all_user_tokens(&self, user_id: i64) -> Result<(), RedisError> {
        let tokens = self.get_all_user_tokens(user_id).await?;
        
        for token in tokens {
            self.invalidate_token(&token).await?;
        }
        
        Ok(())
    }
}