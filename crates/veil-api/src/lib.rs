//! Veil API Server Library
//!
//! This crate provides a REST API server for the Veil privacy platform.
//! It exposes endpoints for scanning documents for PII and protecting
//! them through redaction, masking, or other methods.
//!
//! # Endpoints
//!
//! - `GET /health` - Health check
//! - `GET /health/ready` - Readiness probe
//! - `GET /health/live` - Liveness probe
//! - `POST /api/v1/scan` - Scan a file for PII
//! - `POST /api/v1/protect` - Protect a file by redacting PII
//!
//! # Example
//!
//! ```no_run
//! use veil_api::{ServerConfig, run_server};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ServerConfig::default();
//!     run_server(config).await.unwrap();
//! }
//! ```

pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod security;

pub use config::ServerConfig;
pub use error::{ApiError, ApiResult};
pub use middleware::{Claims, RateLimiter};

use std::sync::Arc;
use std::time::Instant;

use axum::{middleware as axum_middleware, Router};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::middleware::{jwt_auth, rate_limit_layer};

/// Application state shared across handlers.
pub struct AppStateInner {
    /// Server configuration.
    pub config: ServerConfig,
    /// Server start time for uptime calculation.
    pub start_time: Instant,
    /// Rate limiter.
    pub rate_limiter: RateLimiter,
}

/// Shared application state.
pub type AppState = Arc<AppStateInner>;

/// Create the application state.
pub fn create_state(config: ServerConfig) -> AppState {
    let rate_limiter = RateLimiter::new(config.rate_limit.clone());
    Arc::new(AppStateInner {
        config,
        start_time: Instant::now(),
        rate_limiter,
    })
}

/// Create the router with all routes and middleware.
pub fn create_router(state: AppState) -> Router {
    let cors = if state.config.cors.enabled {
        let origins = if state.config.cors.allowed_origins.is_empty() {
            // Default to localhost origins only for security
            AllowOrigin::list([
                "http://localhost:3000".parse().unwrap(),
                "http://localhost:8080".parse().unwrap(),
                "http://127.0.0.1:3000".parse().unwrap(),
                "http://127.0.0.1:8080".parse().unwrap(),
            ])
        } else {
            // Use configured origins
            AllowOrigin::list(
                state
                    .config
                    .cors
                    .allowed_origins
                    .iter()
                    .filter_map(|o| o.parse().ok()),
            )
        };

        let methods: Vec<http::Method> = state
            .config
            .cors
            .allowed_methods
            .iter()
            .filter_map(|m| m.parse().ok())
            .collect();

        let headers: Vec<http::header::HeaderName> = state
            .config
            .cors
            .allowed_headers
            .iter()
            .filter_map(|h| h.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(methods)
            .allow_headers(headers)
            .max_age(std::time::Duration::from_secs(state.config.cors.max_age_secs))
    } else {
        CorsLayer::new()
    };

    // Clone config for middleware closures
    let auth_config = state.config.auth.clone();
    let rate_limiter = state.rate_limiter.clone();

    routes::create_router(state)
        .layer(axum_middleware::from_fn(move |req, next| {
            let config = auth_config.clone();
            jwt_auth(config, req, next)
        }))
        .layer(axum_middleware::from_fn(move |req, next| {
            let limiter = rate_limiter.clone();
            rate_limit_layer(limiter, req, next)
        }))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Create and run the server.
pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = config.socket_addr();
    let state = create_state(config);

    // Spawn rate limiter cleanup task if rate limiting is enabled
    if state.config.rate_limit.enabled {
        middleware::rate_limit::spawn_cleanup_task(
            state.rate_limiter.clone(),
            state.config.rate_limit.period_secs * 2,
        );
    }

    let app = create_router(state);

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Use into_make_service_with_connect_info to provide connection IP for rate limiting
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_state() {
        let config = ServerConfig::default();
        let state = create_state(config);
        assert_eq!(state.config.port, 3000);
    }

    #[test]
    fn test_create_router() {
        let config = ServerConfig::default();
        let state = create_state(config);
        let _router = create_router(state);
    }
}
