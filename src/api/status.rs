use axum::{Json};
use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ServerStatus {
    name: &'static str,
    version: &'static str,
    status: &'static str,
    documentation: &'static str,
    last_updated: String,
}

// Cargo bize environment üzerinden sürümü sağlıyor, manuel girmeye gerek yok
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) async fn get_status_handler() -> Json<ServerStatus> {
    let timestamp = {
        let utc_time = Utc::now();
        let turkish_offset = FixedOffset::east_opt(3 * 3600).unwrap(); // +3 saat
        utc_time.with_timezone(&turkish_offset).to_rfc3339() // ör: 2025-09-13T21:42:35.785219+03:00 (ISO 8601)
    };

    let status = ServerStatus {
        name: "BesinVeri API",
        version: VERSION,
        status: "iyi",
        documentation: "https://github.com/karahanbuhan/besinveri",
        last_updated: timestamp,
    };

    Json(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::NaiveDateTime;
    use serde_json::Value;
    use tower::ServiceExt; // oneshot için

    #[tokio::test]
    async fn test_get_status_handler_success() {
        // Mevcut zamanı al
        let expected_last_updated = {
            let utc_time = Utc::now();
            let turkish_offset = FixedOffset::east_opt(3 * 3600).unwrap();
            utc_time.with_timezone(&turkish_offset).to_rfc3339()
        };

        // Handler’ı çağır
        let response = get_status_handler().await;

        // JSON yanıtını al
        let status: ServerStatus = response.0;

        // Beklenen değerleri kontrol et
        assert_eq!(status.name, "BesinVeri API");
        assert_eq!(status.version, VERSION);
        assert_eq!(status.status, "iyi");
        assert_eq!(
            status.documentation,
            "https://github.com/karahanbuhan/besinveri"
        );
        let actual_last_updated = status.last_updated;
        // ISO 8601 string’lerini karşılaştırmak için chrono ile parse et
        let expected_time =
            NaiveDateTime::parse_from_str(&expected_last_updated[..19], "%Y-%m-%dT%H:%M:%S")
                .expect("Zaman formatı hatalı");
        let actual_time =
            NaiveDateTime::parse_from_str(&actual_last_updated[..19], "%Y-%m-%dT%H:%M:%S")
                .expect("Zaman formatı hatalı");
        let tolerance = chrono::Duration::seconds(1);
        assert!(
            (actual_time - expected_time) <= tolerance
                && (expected_time - actual_time) <= tolerance,
            "Last updated beklenen aralıkta değil: {} != ~{}",
            actual_last_updated,
            expected_last_updated
        );
    }

    #[tokio::test]
    async fn test_get_status_handler_http() {
        // Router oluştur
        let app = axum::Router::new().route("/api/status", axum::routing::get(get_status_handler));

        // HTTP isteği oluştur
        let request = Request::builder()
            .method("GET")
            .uri("/api/status")
            .body(Body::empty())
            .unwrap();

        // İsteği gönder
        let response = app.oneshot(request).await.unwrap();

        // Yanıtın durum kodunu kontrol et
        assert_eq!(response.status(), StatusCode::OK);

        // Yanıtın JSON olduğunu kontrol et
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(); // 1 MB sınır
        let json: Value = serde_json::from_slice(body.as_ref()).expect("JSON parse edilmeli");

        // Alanları kontrol et
        assert_eq!(json["name"].as_str(), Some("BesinVeri API"));
        assert_eq!(json["status"].as_str(), Some("iyi"));
        let actual_last_updated = json["last_updated"].as_str().unwrap_or("");

        let now = {
            let utc_time = Utc::now();
            let turkish_offset = FixedOffset::east_opt(3 * 3600).unwrap();
            utc_time.with_timezone(&turkish_offset).to_rfc3339()
        };
        let expected_time = NaiveDateTime::parse_from_str(&now[..19], "%Y-%m-%dT%H:%M:%S")
            .expect("Zaman formatı hatalı");
        let actual_time =
            NaiveDateTime::parse_from_str(&actual_last_updated[..19], "%Y-%m-%dT%H:%M:%S")
                .expect("Zaman formatı hatalı");
        let tolerance = chrono::Duration::seconds(1);
        assert!(
            (actual_time - expected_time) <= tolerance
                && (expected_time - actual_time) <= tolerance,
            "Last updated beklenen aralıkta değil: {} != ~{}",
            actual_last_updated,
            now
        );
    }
}
