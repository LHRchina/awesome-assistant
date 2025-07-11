use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use futures_util::TryStreamExt as _;
use serde_json;
use std::io::Write;
use tokio::fs;
use uuid::Uuid;

fn get_current_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}", duration.as_secs())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FileInfo {
    id: String,
    filename: String,
    size: u64,
    content_type: Option<String>,
    upload_time: String,
}

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

async fn upload_file(mut payload: Multipart) -> Result<HttpResponse> {
    // Create uploads directory if it doesn't exist
    fs::create_dir_all("./uploads").await.map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to create uploads directory")
    })?;

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        
        if let Some(filename) = content_disposition.get_filename() {
            let filename = filename.to_string();
            let content_type = field.content_type().map(|ct| ct.to_string());

            let file_id = Uuid::new_v4().to_string();
            let file_extension = std::path::Path::new(&filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| format!(".{}", ext))
                .unwrap_or_default();
            
            let stored_filename = format!("{}{}", file_id, file_extension);
            let filepath = format!("./uploads/{}", stored_filename);
            
            // Create file
            let mut f = web::block(move || std::fs::File::create(filepath))
                .await
                .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create file"))??;

            let mut size = 0u64;
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.try_next().await? {
                size += chunk.len() as u64;
                // filesystem operations are blocking, we have to use threadpool
                f = web::block(move || f.write_all(&chunk).map(|_| f))
                    .await
                    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to write file"))??;
            }

            let file_info = FileInfo {
                id: file_id,
                filename,
                size,
                content_type,
                upload_time: chrono::Utc::now().to_rfc3339(),
            };

            // Save file metadata (in a real app, you'd use a database)
            let metadata_path = format!("./uploads/{}.json", stored_filename);
            let metadata_json = serde_json::to_string(&file_info).unwrap();
            fs::write(metadata_path, metadata_json).await.map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to save metadata")
            })?;

            return Ok(HttpResponse::Ok().json(UploadResponse {
                success: true,
                message: "File uploaded successfully".to_string(),
                file: Some(file_info),
            }));
        }
    }

    Ok(HttpResponse::BadRequest().json(UploadResponse {
        success: false,
        message: "No file found in request".to_string(),
        file: None,
    }))
}

async fn list_files() -> Result<HttpResponse> {
    let mut files = Vec::new();
    
    if let Ok(mut entries) = fs::read_dir("./uploads").await {
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    if let Ok(metadata_content) = fs::read_to_string(entry.path()).await {
                        if let Ok(file_info) = serde_json::from_str::<FileInfo>(&metadata_content) {
                            files.push(file_info);
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(FilesListResponse { files }))
}

async fn download_file(path: web::Path<String>) -> Result<HttpResponse> {
    let file_id = path.into_inner();
    
    // Find the file with this ID
    if let Ok(mut entries) = fs::read_dir("./uploads").await {
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.starts_with(&file_id) && !filename.ends_with(".json") {
                    let file_path = entry.path();
                    let file_content = fs::read(&file_path).await.map_err(|_| {
                        actix_web::error::ErrorNotFound("File not found")
                    })?;
                    
                    // Get original filename from metadata
                    let metadata_path = format!("{}.json", file_path.to_string_lossy());
                    let original_filename = if let Ok(metadata_content) = fs::read_to_string(&metadata_path).await {
                        if let Ok(file_info) = serde_json::from_str::<FileInfo>(&metadata_content) {
                            file_info.filename
                        } else {
                            filename.to_string()
                        }
                    } else {
                        filename.to_string()
                    };
                    
                    return Ok(HttpResponse::Ok()
                        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", original_filename)))
                        .body(file_content));
                }
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
    
    println!("Starting file upload server on http://localhost:8080");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
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