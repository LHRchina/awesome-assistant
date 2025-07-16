use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use futures_util::TryStreamExt as _;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use actix_web::http::header::ContentType;

mod storage;
mod auth;
use storage::{CloudflareStorage, FileMetadata};
use auth::{AuthService, Claims, login, me};

fn get_current_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}", duration.as_secs())
}

// Application state to hold storage and metadata
struct AppState {
    storage: CloudflareStorage,
    // In-memory storage for file metadata (in production, use a database)
    file_metadata: Arc<Mutex<HashMap<String, FileMetadata>>>,
}

// Alias for FileInfo to maintain API compatibility
type FileInfo = FileMetadata;

#[derive(serde::Serialize)]
struct UploadResponse {
    success: bool,
    message: String,
    file: Option<FileInfo>,
}

#[derive(serde::Serialize)]
struct FilesListResponse {
    files: Vec<FileInfo>,
}

async fn upload_file(
    claims: Claims,
    mut payload: Multipart,
    data: web::Data<AppState>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse> {
    let user_id: i64 = claims.sub.parse().map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid user ID")
    })?;

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            let filename = filename.to_string();
            let content_type = field.content_type().map(|ct| ct.to_string());

            let mut file_content = Vec::new();
            // Collect all chunks into a single buffer
            while let Some(chunk) = field.try_next().await? {
                file_content.extend_from_slice(&chunk);
            }

            // Upload to Cloudflare R2
            match data.storage.upload_file(&filename, file_content, content_type).await {
                Ok(file_metadata) => {
                    // Store metadata in database
                    match auth_service.execute_query(
                        "INSERT INTO user_files (user_id, file_key) VALUES ($1, $2)",
                        &[&user_id, &file_metadata.s3_key.as_str()]
                    ).await {
                        Ok(_) => {
                            // Also store in memory for backward compatibility
                            let mut metadata_store = data.file_metadata.lock().await;
                            metadata_store.insert(file_metadata.id.clone(), file_metadata.clone());

                            return Ok(HttpResponse::Ok().json(UploadResponse {
                                success: true,
                                message: "File uploaded successfully".to_string(),
                                file: Some(file_metadata),
                            }));
                        }
                        Err(e) => {
                            eprintln!("Database error: {}", e);
                            // Try to delete the uploaded file since database insert failed
                            let _ = data.storage.delete_file(&file_metadata.s3_key).await;
                            return Ok(HttpResponse::InternalServerError().json(UploadResponse {
                                success: false,
                                message: "Failed to save file metadata".to_string(),
                                file: None,
                            }));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Upload error: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(UploadResponse {
                        success: false,
                        message: "Failed to upload file to storage".to_string(),
                        file: None,
                    }));
                }
            }
        }
    }

    Ok(HttpResponse::BadRequest().json(UploadResponse {
        success: false,
        message: "No file found in request".to_string(),
        file: None,
    }))
}

async fn list_files(
    claims: Claims,
    data: web::Data<AppState>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse> {
    let user_id: i64 = claims.sub.parse().map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid user ID")
    })?;

    // Get user's files from database
    match auth_service.query_database(
        "SELECT file_key FROM user_files WHERE user_id = $1",
        &[&user_id]
    ).await {
        Ok(rows) => {
            let metadata_store = data.file_metadata.lock().await;
            let mut user_files = Vec::new();

            for row in rows {
                let file_key: String = row.get(0);
                // Find matching file in memory store by s3_key
                for file_metadata in metadata_store.values() {
                    if file_metadata.s3_key == file_key {
                        user_files.push(file_metadata.clone());
                        break;
                    }
                }
            }

            Ok(HttpResponse::Ok().json(FilesListResponse { files: user_files }))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(UploadResponse {
                success: false,
                message: "Failed to retrieve files".to_string(),
                file: None,
            }))
        }
    }
}

async fn download_file(
    claims: Claims,
    path: web::Path<String>,
    data: web::Data<AppState>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse> {
    let file_id = path.into_inner();
    let user_id: i64 = claims.sub.parse().map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid user ID")
    })?;

    // Find the file metadata and verify ownership
    let metadata_store = data.file_metadata.lock().await;
    if let Some(file_metadata) = metadata_store.get(&file_id) {
        let s3_key = file_metadata.s3_key.clone();
        let original_filename = file_metadata.filename.clone();
        drop(metadata_store); // Release the lock before async operation

        // Check if user owns this file
        match auth_service.query_database(
            "SELECT 1 FROM user_files WHERE user_id = $1 AND file_key = $2",
            &[&user_id, &s3_key.as_str()]
        ).await {
            Ok(rows) => {
                if rows.is_empty() {
                    return Ok(HttpResponse::Forbidden().json(UploadResponse {
                        success: false,
                        message: "Access denied: You don't own this file".to_string(),
                        file: None,
                    }));
                }

                // User owns the file, proceed with download
                match data.storage.download_file(&s3_key).await {
                    Ok(file_content) => {
                        return Ok(HttpResponse::Ok()
                            .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", original_filename)))
                            .body(file_content));
                    }
                    Err(e) => {
                        eprintln!("Download error: {}", e);
                        return Ok(HttpResponse::InternalServerError().json(UploadResponse {
                            success: false,
                            message: "Failed to download file from storage".to_string(),
                            file: None,
                        }));
                    }
                }
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(UploadResponse {
                    success: false,
                    message: "Database error while checking file ownership".to_string(),
                    file: None,
                }));
            }
        }
    }

    Ok(HttpResponse::NotFound().json(UploadResponse {
        success: false,
        message: "File not found".to_string(),
        file: None,
    }))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    println!("Initializing Cloudflare R2 storage...");

    // Initialize CloudflareStorage
    let storage = CloudflareStorage::new("awesome-assistant".to_string())
        .await
        .map_err(|e| {
            eprintln!("Failed to initialize Cloudflare storage: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "Storage initialization failed")
        })?;

    // Initialize AuthService
    let database_url = "postgresql://awesome:mysecretpassword-awesome-assistant@127.0.0.1:5432/awesome";
    let jwt_secret = "your-super-secret-jwt-key-here-make-it-long-and-random";
    let auth_service = AuthService::new(database_url, jwt_secret).await.expect("Failed to connect to database");

    // Create application state
    let app_state = web::Data::new(AppState {
        storage,
        file_metadata: Arc::new(Mutex::new(HashMap::new())),
    });

    let auth_service_data = web::Data::new(auth_service);

    println!("Starting file upload server on http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .app_data(auth_service_data.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .route("/login", web::post().to(login))
                    .route("/upload", web::post().to(upload_file))
                    .route("/files", web::get().to(list_files))
                    .route("/download/{id}", web::get().to(download_file))
                    .route("/me", web::get().to(me))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}