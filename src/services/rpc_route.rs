use axum::extract::Multipart;
use serde_json::json;
use uuid::Uuid;

use crate::services::r2_config::{create_client, upload_file, R2Config, ApiResponse, error_response, success_response};

struct UploadData {
    name: String,
    bytes: Vec<u8>,
    description: Option<String>,
}

async fn parse_multipart(mut multipart: Multipart) -> Result<UploadData, ApiResponse> {
    let mut file_name = format!("uploads/{}.bin", Uuid::new_v4());
    let mut file_bytes = Vec::new();
    let mut description = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let Some(name) = field.name() else { continue };

        match name {
            "file" => {
                if let Some(fname) = field.file_name() {
                    file_name = format!("uploads/{}", fname);
                }
                file_bytes = field.bytes().await.unwrap_or_default().to_vec();
            }
            "description" => {
                description = field.text().await.ok();
            }
            _ => {}
        }
    }

    if file_bytes.is_empty() {
        return Err(error_response("No file provided", axum::http::StatusCode::BAD_REQUEST));
    }

    Ok(UploadData { name: file_name, bytes: file_bytes, description })
}

pub async fn upload_handler(multipart: Multipart) -> ApiResponse {
    dotenvy::dotenv().ok();

    let config = R2Config::from_env().unwrap(); // Config harus ada karena .env sudah di-set
    let client = create_client(&config).await.unwrap(); // Client creation pasti berhasil dengan config valid

    let upload_data = match parse_multipart(multipart).await {
        Ok(data) => data,
        Err(response) => return response,
    };

    let metadata: Vec<(&str, &str)> = upload_data.description
        .iter()
        .map(|d| ("description", d.as_str()))
        .collect();

    upload_file(&client, &config.bucket, &upload_data.name, upload_data.bytes, "application/octet-stream", &metadata).await.unwrap();

    success_response(json!({
        "file": upload_data.name,
        "description": upload_data.description
    }))
}