//! Rate limiting middleware.

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::config::RateLimitConfig;

/// Rate limiter state for a single client.
#[derive(Debug, Clone)]
struct ClientState {
    /// Number of requests in the current window.
    count: u32,
    /// Window start time.
    window_start: Instant,
}

/// In-memory rate limiter.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Configuration.
    pub(crate) config: RateLimitConfig,
    /// Per-client state.
    clients: Arc<RwLock<HashMap<String, ClientState>>>,
    /// Parsed trusted proxy IPs for efficient lookup.
    trusted_proxy_ips: Arc<Vec<std::net::IpAddr>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        let trusted_proxy_ips: Vec<std::net::IpAddr> = config
            .trusted_proxies
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            trusted_proxy_ips: Arc::new(trusted_proxy_ips),
        }
    }

    /// Check if an IP is a trusted proxy.
    pub fn is_trusted_proxy(&self, ip: &std::net::IpAddr) -> bool {
        self.trusted_proxy_ips.contains(ip)
    }

    /// Check if a request is allowed for the given client key.
    ///
    /// Returns (allowed, remaining, reset_after_secs).
    pub fn check(&self, client_key: &str) -> (bool, u32, u64) {
        if !self.config.enabled {
            return (true, self.config.requests_per_period, 0);
        }

        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.period_secs);

        // Handle poisoned lock gracefully - allow request but log warning
        let mut clients = match self.clients.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Rate limiter lock poisoned, recovering");
                poisoned.into_inner()
            }
        };

        let state = clients
            .entry(client_key.to_string())
            .or_insert(ClientState {
                count: 0,
                window_start: now,
            });

        // Check if window has expired
        if now.duration_since(state.window_start) >= window_duration {
            // Reset window
            state.count = 0;
            state.window_start = now;
        }

        let reset_after = window_duration
            .checked_sub(now.duration_since(state.window_start))
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if state.count >= self.config.requests_per_period {
            let remaining = 0;
            return (false, remaining, reset_after);
        }

        state.count += 1;
        let remaining = self.config.requests_per_period.saturating_sub(state.count);
        (true, remaining, reset_after)
    }

    /// Clean up expired entries.
    pub fn cleanup(&self) {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.period_secs);

        // Handle poisoned lock gracefully
        let mut clients = match self.clients.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Rate limiter lock poisoned during cleanup, recovering");
                poisoned.into_inner()
            }
        };
        clients.retain(|_, state| now.duration_since(state.window_start) < window_duration * 2);
    }
}

/// Rate limit error response.
fn rate_limit_error(reset_after: u64) -> Response {
    let body = json!({
        "error": "rate_limit_exceeded",
        "message": "Too many requests",
        "retry_after_secs": reset_after
    });

    (
        StatusCode::TOO_MANY_REQUESTS,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::RETRY_AFTER, &reset_after.to_string()),
        ],
        Json(body),
    )
        .into_response()
}

/// Extract client identifier from request.
///
/// Only trusts X-Forwarded-For and X-Real-IP headers when the direct connection
/// comes from a trusted proxy IP. This prevents IP spoofing attacks.
///
/// # Arguments
/// * `request` - The incoming HTTP request
/// * `connection_ip` - The IP address of the direct connection (from socket)
/// * `limiter` - The rate limiter with trusted proxy configuration
fn get_client_key(
    request: &Request,
    connection_ip: Option<std::net::IpAddr>,
    limiter: &RateLimiter,
) -> String {
    // If we have a connection IP and it's from a trusted proxy, check forwarded headers
    if let Some(conn_ip) = connection_ip {
        if limiter.is_trusted_proxy(&conn_ip) {
            // Connection is from a trusted proxy, so we can trust forwarded headers
            // Try X-Forwarded-For header first
            if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
                if let Ok(value) = forwarded.to_str() {
                    // Take the first IP in the chain (original client)
                    if let Some(ip) = value.split(',').next() {
                        let trimmed = ip.trim();
                        if !trimmed.is_empty() {
                            return trimmed.to_string();
                        }
                    }
                }
            }

            // Try X-Real-IP header
            if let Some(real_ip) = request.headers().get("X-Real-IP") {
                if let Ok(value) = real_ip.to_str() {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }

        // Not from a trusted proxy or no valid forwarded header - use connection IP
        return conn_ip.to_string();
    }

    // No connection IP available - this shouldn't happen in production
    // Log a warning and use a fallback that groups all unknown clients together
    // This is safer than trusting untrusted headers
    tracing::warn!("No connection IP available for rate limiting - grouping as unknown");
    "unknown".to_string()
}

/// Rate limiting middleware layer.
pub async fn rate_limit_layer(
    limiter: RateLimiter,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !limiter.config.enabled {
        return next.run(request).await;
    }

    // Extract connection IP from request extensions (set by axum's ConnectInfo)
    let connection_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip());

    let client_key = get_client_key(&request, connection_ip, &limiter);
    let (allowed, remaining, reset_after) = limiter.check(&client_key);

    if !allowed {
        return rate_limit_error(reset_after);
    }

    // Add rate limit headers to response
    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit",
        limiter.config.requests_per_period.into(),
    );
    headers.insert("X-RateLimit-Remaining", remaining.into());
    headers.insert("X-RateLimit-Reset", reset_after.into());

    response
}

/// Spawn a background task to clean up expired rate limit entries.
pub fn spawn_cleanup_task(limiter: RateLimiter, interval_secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            limiter.cleanup();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_requests() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_period: 5,
            period_secs: 60,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // First 5 requests should be allowed
        for i in 0..5 {
            let (allowed, remaining, _) = limiter.check("client1");
            assert!(allowed, "Request {} should be allowed", i);
            assert_eq!(remaining, 4 - i as u32);
        }

        // 6th request should be blocked
        let (allowed, remaining, _) = limiter.check("client1");
        assert!(!allowed, "Request 6 should be blocked");
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_rate_limiter_separate_clients() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_period: 2,
            period_secs: 60,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // Client 1 uses their quota
        assert!(limiter.check("client1").0);
        assert!(limiter.check("client1").0);
        assert!(!limiter.check("client1").0);

        // Client 2 should still have quota
        assert!(limiter.check("client2").0);
        assert!(limiter.check("client2").0);
        assert!(!limiter.check("client2").0);
    }

    #[test]
    fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            enabled: false,
            requests_per_period: 1,
            period_secs: 60,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // All requests should pass when disabled
        for _ in 0..100 {
            let (allowed, _, _) = limiter.check("client1");
            assert!(allowed);
        }
    }

    #[test]
    fn test_cleanup() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_period: 10,
            period_secs: 1, // 1 second window
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // Make some requests
        limiter.check("client1");
        limiter.check("client2");

        // Should have 2 clients
        assert_eq!(limiter.clients.read().unwrap().len(), 2);

        // Wait for window to expire
        std::thread::sleep(Duration::from_secs(3));

        // Cleanup should remove expired entries
        limiter.cleanup();
        assert_eq!(limiter.clients.read().unwrap().len(), 0);
    }
}
