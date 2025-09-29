use std::{net::SocketAddr, sync::Arc};

use anyhow::Error;
use axum::{routing::get, Router};
use axum_governor::GovernorLayer;
use lazy_limit::{Duration, RuleConfig, init_rate_limiter};
use real::RealIpLayer;
use sqlx::{Pool, Sqlite};
use tokio::{net::TcpListener, sync::Mutex};

use crate::core::config::Config;

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
        api_db: Arc::new(Mutex::new(api::database::connect_database().await?)),
        config: Arc::new(Mutex::new(core::config::load_config_with_defaults()?)),
    };

    // Lazy-limit ile rate-limit ayarlıyoruz, şimdilik basit bir sistem kullanıyoruz; 1 saniyede maksimum 5 istek.
    // Gelecekte kova mantığına geçilebilir ama şimdilik bu sistemin yeterli olması gerekli
    init_rate_limiter!(
        default: RuleConfig::new(Duration::Seconds(1), 5),
        max_memory: Some(64 * 1024 * 1024) // 64MB maksimum bellek
    )
    .await;

    let router = Router::new()
        // Burada handler yerine sadece statik bir endpoints JSON'ı oluşturuyoruz
        .route(
            "/api",
            get(
                api::endpoints::get_endpoints(&shared_state.config.lock().await.api.base_url).await,
            ),
        )
        .route("/api/health", get(api::health::get_health_handler))
        .route("/api/food/{slug}", get(api::foods::get_food_handler))
        .route("/api/foods/list", get(api::foods::get_foods_handler))
        .route(
            "/api/foods/search",
            get(api::foods::get_foods_search_handler),
        )
        .with_state(shared_state.clone())
        .layer(
            tower::ServiceBuilder::new()
                .layer(RealIpLayer::default()) // Bu katman rate limiter için
                .layer(GovernorLayer::default()),
        )
        .into_make_service_with_connect_info::<SocketAddr>(); // Rate limiter için socket adreslere ihtiyacımız var

    axum::serve(TcpListener::bind("0.0.0.0:8099").await?, router).await?;

    Ok(())
}
