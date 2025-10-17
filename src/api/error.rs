use std::fmt::{Display, Formatter};

use axum::{body::Body, extract::Request, http::{self, header, HeaderMap, HeaderValue}, middleware::Next, response::IntoResponse};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize)]
pub(crate) struct APIError {
    // StatusCode kullanmak yerine u16 olarak saklama sebebimiz deserialize ve serialize fonksiyonlarını kullanabilmek
    pub(crate) code: u16,
    pub(crate) message: String,
}

impl APIError {
    pub(crate) fn new(code: StatusCode, message: &str) -> Self {
        APIError {
            code: code.as_u16(),
            message: message.to_owned(),
        }
    }
}

impl IntoResponse for APIError {
    fn into_response(self) -> axum::response::Response {
        let status = match StatusCode::from_u16(self.code) {
            Ok(code) => code,
            _ => StatusCode::INTERNAL_SERVER_ERROR, // Varsayılan INTERNAL_SERVER_ERROR kullanıyoruz eğer kod geçersizse
        };

        // Serde başarısız olursa manuel yapıyoruz? Tehlikeli olabilri mi bu kod
        let body = serde_json::to_string(&self).unwrap_or_else(|e| {
            error!(
                "Serde JSON serileştirme başarısız oldu: {}\nDetaylar: {}",
                self, e
            );

            format!(
                // {{ ve }} kullanıyoruz escape etmek için
                r#"{{"code": {}, "message": "Serde JSON serileştirme başarısız oldu: {}"}}"#,
                status.as_u16(),
                self.message.replace("\"", "\\\""), // Özel karakterleri escape ediyoruz
            )
        });

        // JSON için ve belki gelecekte başka değeler için header açalım
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

        (status, headers, body).into_response()
    }
}

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} (Kod: {})", self.message, self.code)
    }
}

// Middleware için hata işleyicisi
pub(crate) async fn handle_axum_rejections(
    request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, APIError> {
    let response = next.run(request).await;

    // Deserializasyon hatalarını yakalamak için response status kontrolü
    match response.status() {
        status if status.is_client_error() => Err(APIError::new(
            status,
            &(format!(
                "İstemci hatası: {}",
                status.canonical_reason().unwrap_or("Tanımsız davranış")
            )),
        )),
        status if status.is_server_error() => Err(APIError::new(
            status,
            &(format!(
                "Sunucu hatası: {}",
                status.canonical_reason().unwrap_or("Tanımsız davranış")
            )),
        )),
        _ => Ok(response),
    }
}

// Axum'un kendi hatalarını APIError formatına getirmek için middleware
pub(crate) async fn handle_rejections(err: axum::BoxError) -> impl IntoResponse {
    if let Some(rejection) = err.downcast_ref::<axum::extract::rejection::QueryRejection>() {
        let (status, message) = match rejection {
            axum::extract::rejection::QueryRejection::FailedToDeserializeQueryString(e) => (
                StatusCode::BAD_REQUEST,
                format!("Sorgu ters serileştirilirken bir problem oluştu {}", e),
            ),
            _ => (StatusCode::BAD_REQUEST, "Geçeriz sorgu".to_string()),
        };

        APIError::new(status, &message).into_response()
    } else {
        // Diğer hatalar için varsayılan
        APIError::new(StatusCode::INTERNAL_SERVER_ERROR, "Teknik bir hata oluştu").into_response()
    }
}
