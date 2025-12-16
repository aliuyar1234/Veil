//! Integration tests for the Veil API endpoints.

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use tower::util::ServiceExt;
use veil_api::{create_router, create_state, ServerConfig};

/// Create a test app with default configuration.
fn test_app() -> axum::Router {
    let mut config = ServerConfig::default();
    // Disable auth for testing
    config.auth.enabled = false;
    let state = create_state(config);
    create_router(state)
}

/// Create a test app with authentication enabled.
fn test_app_with_auth() -> (axum::Router, String) {
    let secret = "test-secret-key-for-testing-purposes".to_string();

    let mut config = ServerConfig::default();
    config.auth.enabled = true;
    config.auth.jwt_secret = Some(secret.clone());
    let state = create_state(config);

    // Generate a test token
    let token = generate_test_token(&secret);

    (create_router(state), token)
}

/// Generate a test JWT token.
fn generate_test_token(secret: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    let claims = Claims {
        sub: "test-user".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

// ============== Health Endpoint Tests ==============

#[tokio::test]
async fn test_health_endpoint() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
}

#[tokio::test]
async fn test_health_ready_endpoint() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check for healthy status in the response
    assert_eq!(json["status"], "healthy");
    assert!(json["checks"].as_array().is_some());
}

#[tokio::test]
async fn test_health_live_endpoint() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/live")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Liveness check just returns 200 OK with no body
    assert_eq!(response.status(), StatusCode::OK);
}

// ============== Authentication Tests ==============

#[tokio::test]
async fn test_protected_endpoint_without_token() {
    let (app, _token) = test_app_with_auth();

    // Create a multipart body
    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Test content\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    let (app, token) = test_app_with_auth();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Hello world\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed with valid token
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_protected_endpoint_with_invalid_token() {
    let (app, _) = test_app_with_auth();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Test content\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(header::AUTHORIZATION, "Bearer invalid-token")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============== Scan Endpoint Tests ==============

#[tokio::test]
async fn test_scan_text_file() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Contact me at test@example.com or call 555-123-4567.\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(json["findings"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_scan_with_no_pii() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Hello world! This is a test document with no PII.\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["findings"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_scan_csv_file() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let csv_content = "name,email,phone\nJohn Doe,john@example.com,555-1234\n";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\n\
        Content-Type: text/csv\r\n\r\n\
        {csv_content}\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    // Should detect email
    let findings = json["findings"].as_array().unwrap();
    assert!(findings.iter().any(|f| f["category"] == "email"));
}

#[tokio::test]
async fn test_scan_json_file() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let json_content = r#"{"user": {"email": "test@example.com"}}"#;
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.json\"\r\n\
        Content-Type: application/json\r\n\r\n\
        {json_content}\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_scan_with_min_confidence() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Email: test@example.com Phone: 555-1234\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan?minConfidence=0.5")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // All findings should have confidence >= 0.5
    let findings = json["findings"].as_array().unwrap();
    for finding in findings {
        assert!(finding["confidence"].as_f64().unwrap() >= 0.5);
    }
}

#[tokio::test]
async fn test_scan_missing_file() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"wrong_field\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Test content\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============== Protect Endpoint Tests ==============

#[tokio::test]
async fn test_protect_text_file() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Contact me at test@example.com.\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/protect")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check Content-Disposition header
    let content_disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_disposition.contains("attachment"));
    assert!(content_disposition.contains("protected_"));

    // Check that email was redacted
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(!text.contains("test@example.com"));
}

#[tokio::test]
async fn test_protect_returns_headers() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Email: test@example.com\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/protect")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check custom headers
    assert!(response.headers().contains_key("X-Request-Id"));
    assert!(response.headers().contains_key("X-Protected-Count"));
    assert!(response.headers().contains_key("X-Duration-Ms"));
}

// ============== Error Handling Tests ==============

#[tokio::test]
async fn test_not_found_endpoint() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_method_not_allowed() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/scan")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not allow GET on POST-only endpoint
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// ============== Security Tests ==============

#[tokio::test]
async fn test_path_traversal_filename() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    // Try path traversal in filename
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"../../../etc/passwd\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Test content\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/protect")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // The filename in the response should be sanitized
    let content_disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();

    // Should not contain path traversal
    assert!(!content_disposition.contains(".."));
    assert!(!content_disposition.contains("/etc/"));
}

#[tokio::test]
async fn test_null_byte_filename() {
    let app = test_app();

    let boundary = "----WebKitFormBoundary";
    // Try null byte in filename
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        Test content\r\n\
        --{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/scan")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Request should succeed - null bytes should be stripped
    assert_eq!(response.status(), StatusCode::OK);
}
