mod services;

use std::error::Error;

use aws_sdk_s3::{config::Credentials, primitives::ByteStream, Client, Config};
use aws_types::region::Region;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let account_id = std::env::var("CF_ACCOUNT_ID")?;
    let access_key = std::env::var("R2_ACCESS_KEY")?;
    let secret_key = std::env::var("R2_SECRET_KEY")?;
    let bucket = std::env::var("CF_BUCKET_NAME")?;

    let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", account_id);

    let config = Config::builder()
         .region(Region::new("auto"))
         .endpoint_url(endpoint_url)
         .credentials_provider(Credentials::new(access_key, secret_key, None, None, "r2"))
         .behavior_version_latest() 
         .build();

    let client = Client::from_conf(config);

    let data = serde_json::json!({
        "message" : "HAI DARI RUST",
        "timestamp" : Utc::now().to_rfc3339(),
    });

    let body = ByteStream::from(data.to_string().into_bytes());

    client.put_object()
         .bucket(bucket)
         .key("example/json")
         .body(body)
         .send()
         .await?;

    println!("JSON uploaded successfully");
    Ok(())
}