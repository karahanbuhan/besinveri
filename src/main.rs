use std::sync::Arc;

use anyhow::Error;
use axum::{Router, routing::get};
use sqlx::{Pool, Sqlite};
use tokio::sync::Mutex;

use crate::api::{foods::get_foods_handler, status::get_status_handler};

mod api;
mod core;

#[derive(Clone)]
struct SharedState {
    api_db: Arc<Mutex<Pool<Sqlite>>>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();

    // Veritabanı ve benzerlerini, tüm handlerlar ile kullanabilmek için bir shared_state oluşturuyoruz
    let shared_state = SharedState {
        api_db: Arc::new(Mutex::new(api::database::connect().await?)),
    };

    let router = Router::new()
        .route("/api/status", get(get_status_handler))
        .route("/api/foods", get(get_foods_handler))
        .with_state(shared_state.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8099").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
