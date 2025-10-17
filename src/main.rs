use std::{net::SocketAddr, sync::Arc, usize};

use anyhow::Error;
use axum::{
    Router, ServiceExt,
    body::Body,
    error_handling::HandleErrorLayer,
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use axum_governor::GovernorLayer;
use lazy_limit::{Duration, RuleConfig, init_rate_limiter};
use moka::future::Cache;
use real::RealIpLayer;
use reqwest::{
    StatusCode,
    header::{CACHE_CONTROL, CONTENT_TYPE},
};
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

async fn cache_middleware(
    State(state): State<SharedState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let url = request.uri().to_string();
    let ttl = match request.uri().path() {
        "/api" | "/api/foods" => std::time::Duration::MAX, // Bu 2 endpoint zaten statik o yüzden bir defa cache atmamız yeterli,
        "/api/health" => std::time::Duration::from_secs(600), // Timestamp attığı ve anlık önemli olduğu için 10 dakikada 1 cache
        path if path.starts_with("/api/food") => std::time::Duration::from_secs(28800), // 8 saatte bir diğer yemek endpointleri için şimdilik güzel
        _ => std::time::Duration::from_secs(3600), // Varsayılan 1 saat, başka bir endpoint yok ama yine de ekleyelim
    };

    // Önce veri önbelleğe zaten kaydedilmiş mi bakıyoruz
    // Eğer cache edilen sayfanın ömrü bittiyse zaten moka halletmiş olacak, bizim bir ttl kontrolü yapmamıza gerek yok
    if let Some(cached) = state.cache.get(&url).await {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .header(CACHE_CONTROL, format!("public, max-age={}", ttl.as_secs()))
            .body(cached.into())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(response);
    }

    // Önbellekte yoksa yani ilk defa giriliyorsa veya ömrü bittiyse cache'in handlerı çalıştıracağız
    let response = next.run(request).await;
    // Eğer hata döndürüyorsa cache atmıyoruz çünkü geçici bir durum olabilir, direkt döndürüyoruz
    if response.status() != StatusCode::OK {
        return Ok(response);
    }

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = std::str::from_utf8(&body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_owned();
    // Daha sonra cache'e ekleyeceğiz, yanıt başarılı veya başarısız olabilir
    state.cache.insert(url, body.to_owned()).await;

    // Cache-Control başlığını da unutmuyoruz header olarak, client tarafında da cache için
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(CACHE_CONTROL, format!("public, max-age={}", ttl.as_secs()))
        .body(body.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
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

    let router = Router::new().nest("/api", api_router(shared_state));
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
            |state, request, next| cache_middleware(state, request, next),
        ))
        .layer(
            tower::ServiceBuilder::new()
                .layer(RealIpLayer::default()) // Governor'dan önce kurulmalı
                .layer(GovernorLayer::default()), // Bu katman rate limiter için
        )
}
