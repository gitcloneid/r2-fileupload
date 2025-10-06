use axum::{
    extract::Multipart,
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use serde_json::json;
use uuid::Uuid;

use crate::services::R2Config::{build_r2_client, upload_file_with_metadata, R2Config};
use std::error::Error;
pub async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    // Load config dari env
    if let Err(err) = dotenvy::dotenv() {
        eprintln!("Warning: .env load error: {:?}", err);
    }
    let cfg = match R2Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to read config: {}", e),
                })),
            );
        }
    };

    let client = match build_r2_client(&cfg).await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to build R2 client: {}", e),
                })),
            );
        }
    };

    // Parsing fields
    let mut file_name = format!("uploads/{}.bin", Uuid::new_v4());
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut metadata_description: Option<String> = None;

    while let Ok(Some(field_result)) = multipart.next_field().await {
        {
                if let Some(name) = field_result.name() {
                    let name = name.to_string();
                    if name == "file" {
                        // tentukan nama file jika tersedia
                        if let Some(fname) = field_result.file_name() {
                            file_name = format!("uploads/{}", fname);
                        }
                        match field_result.bytes().await {
                            Ok(b) => file_bytes = b.to_vec(),
                            Err(e) => {
                                return (
                                    StatusCode::BAD_REQUEST,
                                    Json(json!({
                                        "status": "error",
                                        "message": format!("Failed to read file bytes: {}", e),
                                    })),
                                );
                            }
                        }
                    } else if name == "description" {
                        match field_result.text().await {
                            Ok(txt) => metadata_description = Some(txt),
                            Err(e) => {
                                return (
                                    StatusCode::BAD_REQUEST,
                                    Json(json!({
                                        "status": "error",
                                        "message": format!("Failed to read description: {}", e),
                                    })),
                                );
                            }
                        }
                    }
                }
        }
    }

    // Siapkan metadata sebagai pasangan k/v
    let mut metadata_vec: Vec<(String, String)> = Vec::new();
    if let Some(desc) = metadata_description {
        metadata_vec.push(("description".to_string(), desc));
    }

    // Upload ke R2
    let upload_res = upload_file_with_metadata(
        &client,
        &cfg.bucket,
        &file_name,
        file_bytes,
        Some("application/octet-stream"),
        Some(&metadata_vec.iter().map(|(k,v)| (k.as_str(), v.as_str())).collect::<Vec<_>>()),
    ).await;

    match upload_res {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "file": file_name,
                    "metadata": metadata_vec,
                })),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": e.to_string(),
                })),
            )
        }
    }
}
