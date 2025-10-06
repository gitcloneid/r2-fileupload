use std::{error::Error, ffi::FromBytesWithNulError, result};

use aws_config::{default_provider::account_id_endpoint_mode, imds::client, BehaviorVersion, Region};
use aws_sdk_s3::{config::{endpoint, Credentials}, primitives::ByteStream, types::{builders::BucketLifecycleConfigurationBuilder, ContinuationEvent}, Client, Config};


#[derive(Debug, Clone)]
pub struct R2Config {
    pub account_id: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: Option<String>,
    pub endpoint_override: Option<String>
}

impl R2Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self { 
            account_id: std::env::var("CF_ACCOUNT_ID")?,
            access_key: std::env::var("R2_ACCESS_KEY")?,
            secret_key: std::env::var("R2_SECRET_KEY")?,
            bucket: std::env::var("CF_BUCKET_NAME")?,
            region: Some("auto".to_string()),
            endpoint_override: None,
         })
    }
}

pub async fn build_r2_client(cfg: &R2Config) -> Result<Client, Box<dyn Error>> {
    let endpoint = if let Some(e) = &cfg.endpoint_override {
        e.clone()
    } else {
        format!("https://{}.r2.cloudflare.com", cfg.account_id)
    };

    let aws_config = Config::builder()
         .behavior_version_latest()
         .region(Region::new(cfg.region.clone().unwrap_or_else(|| "auto".into())))
         .endpoint_url(endpoint)
         .credentials_provider(Credentials::new(
            &cfg.access_key,
            &cfg.secret_key,
            None,
            None,
            "r2",
         ))
         .build();

    Ok(Client::from_conf(aws_config))
}

pub async fn upload_project(
    client: &Client,
    bucket: &str,
    key: &str,
    bytes: Vec<u8>,
    content_type: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut req = client.put_object()
         .bucket(bucket)
         .key(key)
         .body(ByteStream::from(bytes));

    if let Some(ct) = content_type {
        req = req.content_type(ct);
    }

    req.send().await?;
    Ok(())
}