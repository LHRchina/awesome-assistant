use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use futures_util::TryStreamExt as _;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

mod storage;
use storage::{CloudflareStorage, FileMetadata};

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
    mut payload: Multipart,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
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
                    // Store metadata in memory (in production, use a database)
                    let mut metadata_store = data.file_metadata.lock().await;
                    metadata_store.insert(file_metadata.id.clone(), file_metadata.clone());

                    return Ok(HttpResponse::Ok().json(UploadResponse {
                        success: true,
                        message: "File uploaded successfully".to_string(),
                        file: Some(file_metadata),
                    }));
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

async fn list_files(data: web::Data<AppState>) -> Result<HttpResponse> {
    let metadata_store = data.file_metadata.lock().await;
    let files: Vec<FileInfo> = metadata_store.values().cloned().collect();

    Ok(HttpResponse::Ok().json(FilesListResponse { files }))
}

async fn download_file(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let file_id = path.into_inner();
    
    // Find the file metadata
    let metadata_store = data.file_metadata.lock().await;
    if let Some(file_metadata) = metadata_store.get(&file_id) {
        let s3_key = file_metadata.s3_key.clone();
        let original_filename = file_metadata.filename.clone();
        drop(metadata_store); // Release the lock before async operation
        
        // Download from Cloudflare R2
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
    let storage = CloudflareStorage::new("file-upload-bucket".to_string())
        .await
        .map_err(|e| {
            eprintln!("Failed to initialize Cloudflare storage: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "Storage initialization failed")
        })?;
    
    // Create application state
    let app_state = web::Data::new(AppState {
        storage,
        file_metadata: Arc::new(Mutex::new(HashMap::new())),
    });
    
    println!("Starting file upload server on http://localhost:8080");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .route("/upload", web::post().to(upload_file))
            .route("/files", web::get().to(list_files))
            .route("/download/{id}", web::get().to(download_file))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}