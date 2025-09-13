use axum::{Router, routing::get};

use crate::api::status::get_status_handler;

mod api;
mod core;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();

    let api_db = api::database::connect().await;

    let router = Router::new().route("/api/status", get(get_status_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8099").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
