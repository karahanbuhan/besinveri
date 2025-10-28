use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
    http::header::{HeaderName, HeaderValue},
};

/// Security headers middleware that adds various security headers to responses
pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Prevent clickjacking attacks
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // Prevent MIME type sniffing
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // Enable XSS protection in older browsers
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Content Security Policy - restrict resource loading
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';"),
    );

    // Referrer policy - control referrer information
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions policy - disable unnecessary browser features
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=(), payment=(), usb=()"),
    );

    // Strict Transport Security - force HTTPS (only if using HTTPS)
    // Note: Only enable this if your server is behind HTTPS
    // headers.insert(
    //     HeaderName::from_static("strict-transport-security"),
    //     HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    // );

    response
}
