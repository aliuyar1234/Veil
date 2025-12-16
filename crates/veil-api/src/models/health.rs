//! Health check models.

use serde::{Deserialize, Serialize};

/// Health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy.
    Healthy,
    /// Service is degraded but functional.
    Degraded,
    /// Service is unhealthy.
    Unhealthy,
}

/// Basic health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status.
    pub status: HealthStatus,
    /// Server version.
    pub version: String,
    /// Uptime in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_secs: Option<u64>,
}

/// Readiness check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessResponse {
    /// Overall readiness status.
    pub status: HealthStatus,
    /// Individual component checks.
    pub checks: Vec<ComponentCheck>,
}

/// Individual component health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCheck {
    /// Component name.
    pub name: String,
    /// Component status.
    pub status: HealthStatus,
    /// Optional message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl HealthResponse {
    /// Create a healthy response.
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: None,
        }
    }

    /// Create a healthy response with uptime.
    pub fn healthy_with_uptime(uptime_secs: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: Some(uptime_secs),
        }
    }
}

impl ReadinessResponse {
    /// Create a ready response.
    pub fn ready(checks: Vec<ComponentCheck>) -> Self {
        let all_healthy = checks.iter().all(|c| c.status == HealthStatus::Healthy);
        Self {
            status: if all_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded
            },
            checks,
        }
    }

    /// Create a not ready response.
    pub fn not_ready(checks: Vec<ComponentCheck>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            checks,
        }
    }
}

impl ComponentCheck {
    /// Create a healthy component check.
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
        }
    }

    /// Create an unhealthy component check.
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
        }
    }
}
