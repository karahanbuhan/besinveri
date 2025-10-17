use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use reqwest::{header::{CACHE_CONTROL, CONTENT_TYPE}, StatusCode};

use crate::SharedState;

pub async fn cache_middleware(
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
