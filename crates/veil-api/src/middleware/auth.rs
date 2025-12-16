//! JWT authentication middleware.

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::AuthConfig;

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID or client ID).
    pub sub: String,
    /// Expiration time (as Unix timestamp).
    pub exp: usize,
    /// Issued at time (as Unix timestamp).
    #[serde(default)]
    pub iat: usize,
    /// Issuer.
    #[serde(default)]
    pub iss: Option<String>,
    /// Audience.
    #[serde(default)]
    pub aud: Option<String>,
    /// Scopes/permissions.
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Authentication error response.
fn auth_error(message: &str) -> Response {
    let body = json!({
        "error": "unauthorized",
        "message": message
    });

    (
        StatusCode::UNAUTHORIZED,
        [(header::CONTENT_TYPE, "application/json")],
        Json(body),
    )
        .into_response()
}

/// Extract bearer token from Authorization header.
fn extract_bearer_token(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()
        .and_then(|h| h.strip_prefix("Bearer "))
}

/// JWT authentication middleware.
///
/// Validates the JWT token from the Authorization header.
/// If authentication is disabled in config, all requests pass through.
pub async fn jwt_auth(config: AuthConfig, mut request: Request<Body>, next: Next) -> Response {
    // Skip auth if disabled
    if !config.enabled {
        return next.run(request).await;
    }

    // Get the secret key
    let secret = match &config.jwt_secret {
        Some(s) => s,
        None => {
            tracing::warn!("JWT authentication enabled but no secret configured");
            return auth_error("Server misconfiguration");
        }
    };

    // Extract the token
    let token = match extract_bearer_token(&request) {
        Some(t) => t,
        None => {
            return auth_error("Missing or invalid Authorization header");
        }
    };

    // Build validation rules
    let mut validation = Validation::default();
    validation.validate_exp = true;

    if let Some(ref issuer) = config.jwt_issuer {
        validation.set_issuer(&[issuer]);
    }

    if let Some(ref audience) = config.jwt_audience {
        validation.set_audience(&[audience]);
    }

    // Decode and validate the token
    let key = DecodingKey::from_secret(secret.as_bytes());
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => {
            // Store claims in request extensions for handlers to use
            request.extensions_mut().insert(token_data.claims);
            next.run(request).await
        }
        Err(e) => {
            tracing::debug!("JWT validation failed: {}", e);
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    auth_error("Token has expired")
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => auth_error("Invalid token format"),
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    auth_error("Invalid token signature")
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                    auth_error("Invalid token issuer")
                }
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    auth_error("Invalid token audience")
                }
                _ => auth_error("Token validation failed"),
            }
        }
    }
}

/// Create a JWT token for testing.
#[cfg(test)]
pub fn create_test_token(secret: &str, claims: &Claims) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn current_timestamp() -> usize {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
    }

    #[test]
    fn test_create_and_decode_token() {
        let secret = "test_secret_key";
        let claims = Claims {
            sub: "user123".to_string(),
            exp: current_timestamp() + 3600,
            iat: current_timestamp(),
            iss: None,
            aud: None,
            scopes: vec!["read".to_string(), "write".to_string()],
        };

        let token = create_test_token(secret, &claims);
        assert!(!token.is_empty());

        // Decode it back
        let key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::default();
        validation.validate_aud = false;
        let decoded = decode::<Claims>(&token, &key, &validation).unwrap();

        assert_eq!(decoded.claims.sub, "user123");
        assert_eq!(decoded.claims.scopes, vec!["read", "write"]);
    }

    #[test]
    fn test_expired_token() {
        let secret = "test_secret_key";
        let claims = Claims {
            sub: "user123".to_string(),
            exp: current_timestamp() - 3600, // Expired 1 hour ago
            iat: current_timestamp() - 7200,
            iss: None,
            aud: None,
            scopes: vec![],
        };

        let token = create_test_token(secret, &claims);
        let key = DecodingKey::from_secret(secret.as_bytes());

        let result = decode::<Claims>(&token, &key, &Validation::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature() {
        let claims = Claims {
            sub: "user123".to_string(),
            exp: current_timestamp() + 3600,
            iat: current_timestamp(),
            iss: None,
            aud: None,
            scopes: vec![],
        };

        let token = create_test_token("secret1", &claims);
        let key = DecodingKey::from_secret(b"different_secret");

        let result = decode::<Claims>(&token, &key, &Validation::default());
        assert!(result.is_err());
    }
}
