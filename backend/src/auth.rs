use actix_web::{web, HttpRequest, HttpResponse, Result, FromRequest, dev::Payload};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::{Ready, ready};
use tokio_postgres::{NoTls, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub name: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct GoogleTokenInfo {
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub sub: String, // Google user ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub third_party_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub google_token: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub user: Option<User>,
}

pub struct AuthService {
    db_client: tokio_postgres::Client,
    database_url: String,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub async fn new(database_url: &str, jwt_secret: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
        
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });
        
        Ok(Self {
            db_client: client,
            database_url: database_url.to_string(),
            encoding_key,
            decoding_key,
        })
    }
    
    // Public method to execute database queries
    pub async fn execute_query(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64, tokio_postgres::Error> {
        self.db_client.execute(query, params).await
    }
    
    // Public method to query database
    pub async fn query_database(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error> {
        self.db_client.query(query, params).await
    }

    pub async fn verify_google_token(&self, token: &str) -> Result<GoogleTokenInfo, Box<dyn std::error::Error>> {
        let client = Client::new();
        let url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", token);
        
        let response = client.get(&url).send().await?;
        
        if response.status().is_success() {
            let token_info: GoogleTokenInfo = response.json().await?;
            Ok(token_info)
        } else {
            Err("Invalid Google token".into())
        }
    }

    pub async fn find_or_create_user(&self, google_info: &GoogleTokenInfo) -> Result<User, Box<dyn std::error::Error>> {
        // First, try to find existing user
        let rows = self.db_client
            .query("SELECT id, name, email, third_party_id FROM users WHERE third_party_id = $1", &[&google_info.sub])
            .await?;

        if let Some(row) = rows.first() {
            // User exists, return it
            Ok(User {
                id: row.get(0),
                name: row.get(1),
                email: row.get(2),
                third_party_id: row.get(3),
            })
        } else {
            // User doesn't exist, create new one
            let rows = self.db_client
                .query(
                    "INSERT INTO users (name, email, third_party_id) VALUES ($1, $2, $3) RETURNING id, name, email, third_party_id",
                    &[&google_info.name, &google_info.email, &google_info.sub]
                )
                .await?;

            let row = rows.first().ok_or("Failed to create user")?;
            Ok(User {
                id: row.get(0),
                name: row.get(1),
                email: row.get(2),
                third_party_id: row.get(3),
            })
        }
    }

    pub fn generate_jwt(&self, user: &User) -> Result<String, Box<dyn std::error::Error>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
            exp: expiration as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &self.encoding_key,
        )?;

        Ok(token)
    }

    pub fn verify_jwt(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let rows = self.db_client
            .query("SELECT id, name, email, third_party_id FROM users WHERE id = $1", &[&user_id])
            .await?;

        if let Some(row) = rows.first() {
            Ok(Some(User {
                id: row.get(0),
                name: row.get(1),
                email: row.get(2),
                third_party_id: row.get(3),
            }))
        } else {
            Ok(None)
        }
    }
}

// Custom extractor for authentication
impl FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        
        let token = match auth_header {
            Some(header_value) => {
                match header_value.to_str() {
                    Ok(header_str) => {
                        if header_str.starts_with("Bearer ") {
                            &header_str[7..]
                        } else {
                            return ready(Err(actix_web::error::ErrorUnauthorized("Invalid authorization header format")));
                        }
                    }
                    Err(_) => {
                        return ready(Err(actix_web::error::ErrorUnauthorized("Invalid authorization header")));
                    }
                }
            }
            None => {
                return ready(Err(actix_web::error::ErrorUnauthorized("Missing authorization header")));
            }
        };

        // Get auth service from app data
        let auth_service = match req.app_data::<web::Data<AuthService>>() {
            Some(service) => service,
            None => {
                return ready(Err(actix_web::error::ErrorInternalServerError("Auth service not found")));
            }
        };

        match auth_service.verify_jwt(token) {
            Ok(claims) => {
                // Note: We can't do async operations in FromRequest, so we'll skip the user existence check here
                // The user existence should be validated at the application level if needed
                ready(Ok(claims))
            }
            Err(_) => {
                ready(Err(actix_web::error::ErrorUnauthorized("Invalid token")))
            }
        }
    }
}

// Login endpoint
pub async fn login(
    login_req: web::Json<LoginRequest>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse> {
    match auth_service.verify_google_token(&login_req.google_token).await {
        Ok(google_info) => {
            match auth_service.find_or_create_user(&google_info).await {
                Ok(user) => {
                    match auth_service.generate_jwt(&user) {
                        Ok(token) => {
                            Ok(HttpResponse::Ok().json(LoginResponse {
                                success: true,
                                message: "Login successful".to_string(),
                                token: Some(token),
                                user: Some(user),
                            }))
                        }
                        Err(e) => {
                            eprintln!("JWT generation error: {}", e);
                            Ok(HttpResponse::InternalServerError().json(LoginResponse {
                                success: false,
                                message: "Failed to generate token".to_string(),
                                token: None,
                                user: None,
                            }))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("User creation error: {}", e);
                    Ok(HttpResponse::InternalServerError().json(LoginResponse {
                        success: false,
                        message: "Failed to create or find user".to_string(),
                        token: None,
                        user: None,
                    }))
                }
            }
        }
        Err(e) => {
            eprintln!("Google token verification error: {}", e);
            Ok(HttpResponse::Unauthorized().json(LoginResponse {
                success: false,
                message: "Invalid Google token".to_string(),
                token: None,
                user: None,
            }))
        }
    }
}

// Get current user info
pub async fn me(
    claims: Claims,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse> {
    let user_id: i64 = claims.sub.parse().map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid user ID")
    })?;
    
    match auth_service.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            Ok(HttpResponse::Ok().json(user))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(LoginResponse {
                success: false,
                message: "User not found".to_string(),
                token: None,
                user: None,
            }))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(LoginResponse {
                success: false,
                message: "Database error".to_string(),
                token: None,
                user: None,
            }))
        }
    }
}