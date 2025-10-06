use axum::{routing::post, Router};
use std::net::SocketAddr;
use services::rpc_route::upload_handler;

mod services;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/upload", post(upload_handler))
        .route("/", axum::routing::get(|| async {
            axum::Json(serde_json::json!({
                "message": "R2 Upload Server",
                "endpoint": "POST /upload"
            }))
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 27701));
    println!("Server ran on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}