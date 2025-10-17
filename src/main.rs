use std::{net::SocketAddr, sync::Arc};

use anyhow::Error;
use axum::{
    Router, ServiceExt,
    extract::Request,
    middleware::{self},
    routing::get,
};
use axum_governor::GovernorLayer;
use lazy_limit::{Duration, RuleConfig, init_rate_limiter};
use moka::future::Cache;
use real::RealIpLayer;
use sqlx::{Pool, Sqlite};
use tokio::{net::TcpListener, sync::Mutex};
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;

use crate::core::config::Config;

mod api;
mod core;

// Veritabanı ve config'i, tüm handlerlar içinde kullanabilmek için bir shared_state oluşturuyoruz, cache de dahil
#[derive(Clone)]
struct SharedState {
    api_db: Arc<Mutex<Pool<Sqlite>>>,
    config: Arc<Mutex<Config>>,
    cache: Cache<String, String>, // URL -> JSON şeklinde caching yapacağız
}

impl SharedState {
    async fn new() -> Result<Self, Error> {
        let api_db = Arc::new(Mutex::new(api::database::connect_database().await?));
        let config = Arc::new(Mutex::new(core::config::load_config_with_defaults()?));

        let cache_capacity = config.lock().await.core.cache_capacity;
        let cache = Cache::builder()
            .max_capacity(cache_capacity)
            .time_to_live(std::time::Duration::from_secs(10 * 60))
            .build();

        Ok(Self {
            api_db,
            config,
            cache,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();

    // Lazy-limit ile rate-limit ayarlıyoruz, şimdilik basit bir sistem kullanıyoruz; 1 saniyede maksimum 5 istek.
    // Gelecekte kova mantığına geçilebilir ama şimdilik bu sistemin yeterli olması gerekli
    init_rate_limiter!(
        default: RuleConfig::new(Duration::Seconds(1), 5),
        max_memory: Some(64 * 1024 * 1024) // 64MB maksimum bellek
    )
    .await;

    let shared_state = SharedState::new().await?;

    // http(s)://alanadi.com/API/NEST/PATH -> Bu şekilde girildiğinde /API/NEST/PATH'i kullanacağız nest için
    // Scope içine açıyorum ownership sorununu düzeltmek için, ayrıca String kullanmamız gerekecek referans kullanamayız burada
    let api_path: String = {
        let config_guard = shared_state.config.lock().await;
        config_guard
            .api
            .base_url
            .replace("://", "") // Kesme işaretlerini istemiyoru başlangıçtaki
            .split_once('/')
            .map(|(_before, after)| format!("/{}", after))
            .unwrap_or("/".to_owned())
    };

    // Nest'in içine boş path yazarsak Axum sorun çıkartıyor o yüzden böyle yapıyoruz
    let router = if api_path == "/" {
        api_router(shared_state)
    } else {
        Router::new().nest(&api_path, api_router(shared_state))
    };

    // trim_trailing_slash ile /api/ -> /api şeklinde düzeltiyoruz aksi takdirde routelar çalışmıyor, ayrıca IP adreslerine de ihtiyacımız var rate limit için, connect info ayarlıyoruz
    let router = ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(
        NormalizePathLayer::trim_trailing_slash().layer(router),
    );

    axum::serve(TcpListener::bind("0.0.0.0:8099").await?, router).await?;

    Ok(())
}

fn api_router(shared_state: SharedState) -> Router {
    Router::new()
        .route("/", get(api::endpoints::endpoints))
        .route("/health", get(api::health::health))
        .route("/food/{slug}", get(api::foods::food))
        .route("/foods", get(api::foods::foods))
        .route("/foods/list", get(api::foods::foods_list))
        .route("/foods/search", get(api::foods::foods_search))
        .with_state(shared_state.clone())
        .layer(middleware::from_fn(api::error::handle_axum_rejections)) // Bu da axum'un kendi hataları için, özellikle deserializasyon gibi hatalar için JSON çevirici
        .fallback(api::error::APIError::not_found_handler)
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            |state, request, next| api::cache::cache_middleware(state, request, next),
        ))
        .layer(
            tower::ServiceBuilder::new()
                .layer(RealIpLayer::default()) // Governor'dan önce kurulmalı
                .layer(GovernorLayer::default()), // Bu katman rate limiter için
        )
}
