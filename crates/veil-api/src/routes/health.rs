//! Health check endpoints.

use axum::{extract::State, http::StatusCode, Json};

use crate::models::{ComponentCheck, HealthResponse, HealthStatus, ReadinessResponse};
use crate::AppState;

/// GET /health - Basic health check.
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    Json(HealthResponse::healthy_with_uptime(uptime))
}

/// GET /health/ready - Readiness probe for load balancers.
pub async fn readiness_check(
    State(_state): State<AppState>,
) -> Result<Json<ReadinessResponse>, (StatusCode, Json<ReadinessResponse>)> {
    // Check all components are ready
    let checks = vec![
        // Parser check - always available (stateless)
        ComponentCheck::healthy("parser"),
        // Detector check - always available (stateless)
        ComponentCheck::healthy("detector"),
        // Redactor check - always available (stateless)
        ComponentCheck::healthy("redactor"),
    ];

    let all_healthy = checks.iter().all(|c| c.status == HealthStatus::Healthy);

    if all_healthy {
        Ok(Json(ReadinessResponse::ready(checks)))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadinessResponse::not_ready(checks)),
        ))
    }
}

/// GET /health/live - Liveness probe for orchestrators.
pub async fn liveness_check() -> StatusCode {
    // Simple liveness - if we can respond, we're alive
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Instant;

    fn test_state() -> AppState {
        let config = crate::config::ServerConfig::default();
        let rate_limiter = crate::RateLimiter::new(config.rate_limit.clone());
        Arc::new(crate::AppStateInner {
            config,
            start_time: Instant::now(),
            rate_limiter,
        })
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = test_state();
        let response = health_check(State(state)).await;
        assert_eq!(response.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let state = test_state();
        let result = readiness_check(State(state)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let status = liveness_check().await;
        assert_eq!(status, StatusCode::OK);
    }
}
