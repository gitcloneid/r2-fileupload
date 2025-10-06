mod services;
use axum::{
    routing::post,
    Router,
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use services::rpc_route::upload_handler;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/upload", post(upload_handler));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server run http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}