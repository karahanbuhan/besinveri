use axum::{Router, routing};

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", routing::get(|| async { "Hello, World" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8099").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
