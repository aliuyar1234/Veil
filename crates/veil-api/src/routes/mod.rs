//! API route handlers.

pub mod health;
pub mod protect;
pub mod scan;

use axum::{routing::get, routing::post, Router};

use crate::AppState;

/// Create the main API router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health endpoints
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        // API v1 endpoints
        .nest("/api/v1", api_v1_router())
        .with_state(state)
}

/// API v1 routes.
fn api_v1_router() -> Router<AppState> {
    Router::new()
        .route("/scan", post(scan::scan_file))
        .route("/protect", post(protect::protect_file))
}
