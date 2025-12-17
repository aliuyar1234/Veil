//! Server configuration.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to.
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Maximum request body size in bytes.
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,

    /// Request timeout in seconds.
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    /// Authentication configuration.
    #[serde(default)]
    pub auth: AuthConfig,

    /// Rate limiting configuration.
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// CORS configuration.
    #[serde(default)]
    pub cors: CorsConfig,
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is enabled.
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,

    /// JWT secret key (for HS256).
    #[serde(default)]
    pub jwt_secret: Option<String>,

    /// JWT issuer to validate.
    #[serde(default)]
    pub jwt_issuer: Option<String>,

    /// JWT audience to validate.
    #[serde(default)]
    pub jwt_audience: Option<String>,
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled.
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,

    /// Maximum requests per period.
    #[serde(default = "default_requests_per_period")]
    pub requests_per_period: u32,

    /// Period duration in seconds.
    #[serde(default = "default_period_secs")]
    pub period_secs: u64,

    /// Trusted proxy IPs that are allowed to set X-Forwarded-For headers.
    /// Only requests from these IPs will have X-Forwarded-For headers respected.
    /// Empty list means no proxies are trusted (use direct connection IP only).
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
}

/// CORS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Whether CORS is enabled.
    #[serde(default = "default_cors_enabled")]
    pub enabled: bool,

    /// Allowed origins (empty = all origins).
    #[serde(default)]
    pub allowed_origins: Vec<String>,

    /// Allowed methods.
    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,

    /// Allowed headers.
    #[serde(default = "default_allowed_headers")]
    pub allowed_headers: Vec<String>,

    /// Max age for preflight cache in seconds.
    #[serde(default = "default_max_age")]
    pub max_age_secs: u64,
}

// Default value functions
fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_max_body_size() -> usize {
    100 * 1024 * 1024 // 100MB
}

fn default_request_timeout() -> u64 {
    300 // 5 minutes
}

fn default_auth_enabled() -> bool {
    true
}

fn default_rate_limit_enabled() -> bool {
    false
}

fn default_requests_per_period() -> u32 {
    100
}

fn default_period_secs() -> u64 {
    60
}

fn default_cors_enabled() -> bool {
    true
}

fn default_allowed_methods() -> Vec<String> {
    vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()]
}

fn default_allowed_headers() -> Vec<String> {
    vec![
        "Content-Type".to_string(),
        "Authorization".to_string(),
        "X-Request-Id".to_string(),
    ]
}

fn default_max_age() -> u64 {
    3600
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            max_body_size: default_max_body_size(),
            request_timeout_secs: default_request_timeout(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            cors: CorsConfig::default(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: default_auth_enabled(),
            jwt_secret: None,
            jwt_issuer: None,
            jwt_audience: None,
        }
    }
}

/// Minimum recommended JWT secret length (256 bits = 32 bytes).
const MIN_JWT_SECRET_LENGTH: usize = 32;

impl AuthConfig {
    /// Validate the authentication configuration.
    /// Returns an error message if the configuration is invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            match &self.jwt_secret {
                None => {
                    return Err(
                        "Authentication is enabled but no JWT secret is configured. \
                         Set VEIL_JWT_SECRET environment variable or jwt_secret in config."
                            .to_string(),
                    );
                }
                Some(secret) if secret.len() < MIN_JWT_SECRET_LENGTH => {
                    return Err(format!(
                        "JWT secret is too short ({} bytes). \
                         Use at least {} bytes for security.",
                        secret.len(),
                        MIN_JWT_SECRET_LENGTH
                    ));
                }
                Some(_) => {}
            }
        }
        Ok(())
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_rate_limit_enabled(),
            requests_per_period: default_requests_per_period(),
            period_secs: default_period_secs(),
            trusted_proxies: Vec::new(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: default_cors_enabled(),
            // Default to localhost only for security - production should configure explicitly
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:8080".to_string(),
            ],
            allowed_methods: default_allowed_methods(),
            allowed_headers: default_allowed_headers(),
            max_age_secs: default_max_age(),
        }
    }
}

impl ServerConfig {
    /// Get the socket address.
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], self.port)))
    }

    /// Load configuration from a TOML file.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("VEIL_HOST") {
            config.host = host;
        }
        if let Ok(port) = std::env::var("VEIL_PORT") {
            if let Ok(p) = port.parse() {
                config.port = p;
            }
        }
        if let Ok(max_size) = std::env::var("VEIL_MAX_BODY_SIZE") {
            if let Ok(s) = max_size.parse() {
                config.max_body_size = s;
            }
        }
        if let Ok(secret) = std::env::var("VEIL_JWT_SECRET") {
            config.auth.enabled = true;
            config.auth.jwt_secret = Some(secret);
        }
        if let Ok(issuer) = std::env::var("VEIL_JWT_ISSUER") {
            config.auth.jwt_issuer = Some(issuer);
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.max_body_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_socket_addr() {
        let config = ServerConfig::default();
        let addr = config.socket_addr();
        assert_eq!(addr.port(), 3000);
    }

    #[test]
    fn test_toml_parsing() {
        let toml = r#"
            host = "127.0.0.1"
            port = 8080
            max_body_size = 10485760

            [auth]
            enabled = true
            jwt_secret = "secret"

            [rate_limit]
            enabled = true
            requests_per_period = 50
        "#;

        let config: ServerConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.auth.enabled);
        assert_eq!(config.rate_limit.requests_per_period, 50);
    }
}
