use aws_config::Region;
use aws_sdk_s3::{config::Credentials, primitives::ByteStream, Client, Config};
use serde_json::json;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct R2Config {
    pub account_id: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

impl R2Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            account_id: std::env::var("CF_ACCOUNT_ID")?,
            access_key: std::env::var("R2_ACCESS_KEY")?,
            secret_key: std::env::var("R2_SECRET_KEY")?,
            bucket: std::env::var("CF_BUCKET_NAME")?,
        })
    }

    pub fn endpoint_url(&self) -> String {
        format!("https://{}.r2.cloudflarestorage.com", self.account_id)
    }
}

pub async fn create_client(config: &R2Config) -> Result<Client, Box<dyn Error>> {
    let aws_config = Config::builder()
        .region(Region::new("auto"))
        .endpoint_url(config.endpoint_url())
        .credentials_provider(Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "r2",
        ))
        .behavior_version_latest()
        .build();

    Ok(Client::from_conf(aws_config))
}

pub async fn upload_file(
    client: &Client,
    bucket: &str,
    key: &str,
    data: Vec<u8>,
    content_type: &str,
    metadata: &[(&str, &str)],
) -> Result<(), Box<dyn Error>> {
    let mut request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(data))
        .content_type(content_type);

    for (k, v) in metadata {
        request = request.metadata(k.to_string(), v.to_string());
    }

    request.send().await?;
    Ok(())
}

pub type ApiResponse = (axum::http::StatusCode, axum::Json<serde_json::Value>);

pub fn success_response(data: serde_json::Value) -> ApiResponse {
    (axum::http::StatusCode::OK, axum::Json(json!({
        "status": "success",
        "data": data
    })))
}

pub fn error_response(message: &str, status: axum::http::StatusCode) -> ApiResponse {
    (status, axum::Json(json!({
        "status": "error",
        "message": message
    })))
}