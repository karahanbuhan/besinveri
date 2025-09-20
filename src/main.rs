use std::sync::Arc;

use anyhow::Error;
use axum::{Router, routing::get};
use sqlx::{Pool, Sqlite};
use tokio::sync::Mutex;

use crate::{
    api::{
        endpoints::get_endpoints,
        foods::{get_food_handler, get_foods_handler, get_foods_search_handler},
        status::get_status_handler,
    },
    core::config::Config,
};

mod api;
mod core;

#[derive(Clone)]
struct SharedState {
    api_db: Arc<Mutex<Pool<Sqlite>>>,
    config: Arc<Mutex<Config>>,
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
        config: Arc::new(Mutex::new(core::config::load_config_with_defaults()?)),
    };

    let router = Router::new()
        .route(
            "/api", // Burada handler yerine sadece statik bir endpoints JSON'ı oluşturuyoruz
            get(get_endpoints(&shared_state.config.lock().await.api.base_url).await),
        )
        .route("/api/status", get(get_status_handler))
        .route("/api/foods", get(get_foods_handler))
        .route("/api/foods/{slug}", get(get_food_handler))
        .route("/api/search/foods", get(get_foods_search_handler))
        .with_state(shared_state.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8099").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
