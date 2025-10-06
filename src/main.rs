mod services;
use axum::{
    routing::post,
    Router,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use services::rpc_route::upload_handler;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/upload", post(upload_handler))
        .route("/", axum::routing::get(|| async {
            (StatusCode::OK, Json(json!({
                "message": "R2 Upload Server is running",
                "endpoints": {
                    "upload": "POST /upload - Upload file to R2"
                }
            })))
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}