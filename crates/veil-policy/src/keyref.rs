//! Key reference resolution for encryption keys.

use std::env;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors resolving key references.
#[derive(Debug, Error)]
pub enum KeyRefError {
    /// Invalid key reference format.
    #[error("Invalid key reference format: {0}")]
    InvalidFormat(String),

    /// Unknown URI scheme.
    #[error("Unknown key reference scheme: {0}")]
    UnknownScheme(String),

    /// Environment variable not found.
    #[error("Environment variable not found: {0}")]
    EnvNotFound(String),

    /// Key file not found.
    #[error("Key file not found: {0}")]
    FileNotFound(String),

    /// Failed to read key file.
    #[error("Failed to read key file: {0}")]
    FileReadError(String),

    /// Invalid key data.
    #[error("Invalid key data: {0}")]
    InvalidKey(String),
}

/// Key reference URI schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRefScheme {
    /// Environment variable: env://VAR_NAME
    Env,
    /// File path: file:///path/to/key
    File,
}

impl KeyRefScheme {
    /// Parse scheme from string.
    fn from_str(s: &str) -> Result<Self, KeyRefError> {
        match s.to_lowercase().as_str() {
            "env" => Ok(KeyRefScheme::Env),
            "file" => Ok(KeyRefScheme::File),
            other => Err(KeyRefError::UnknownScheme(other.to_string())),
        }
    }

    /// Get scheme as string.
    fn as_str(&self) -> &'static str {
        match self {
            KeyRefScheme::Env => "env",
            KeyRefScheme::File => "file",
        }
    }
}

/// Reference to an encryption key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRef {
    /// URI scheme (env, file).
    scheme: KeyRefScheme,
    /// Path or variable name.
    path: String,
}

impl KeyRef {
    /// Create a new key reference.
    pub fn new(scheme: KeyRefScheme, path: String) -> Self {
        Self { scheme, path }
    }

    /// Create an environment variable reference.
    pub fn env(var_name: &str) -> Self {
        Self {
            scheme: KeyRefScheme::Env,
            path: var_name.to_string(),
        }
    }

    /// Create a file reference.
    pub fn file(path: &str) -> Self {
        Self {
            scheme: KeyRefScheme::File,
            path: path.to_string(),
        }
    }

    /// Parse a key reference from URI string.
    ///
    /// Supported formats:
    /// - `env://VAR_NAME` - Environment variable
    /// - `file:///path/to/key` - File path
    pub fn parse(uri: &str) -> Result<Self, KeyRefError> {
        let parts: Vec<&str> = uri.splitn(2, "://").collect();
        if parts.len() != 2 {
            return Err(KeyRefError::InvalidFormat(format!(
                "Expected scheme://path, got: {}",
                uri
            )));
        }

        let scheme = KeyRefScheme::from_str(parts[0])?;
        let path = parts[1].to_string();

        if path.is_empty() {
            return Err(KeyRefError::InvalidFormat(
                "Path cannot be empty".to_string(),
            ));
        }

        Ok(Self { scheme, path })
    }

    /// Resolve the key reference to bytes.
    pub fn resolve(&self) -> Result<Vec<u8>, KeyRefError> {
        match self.scheme {
            KeyRefScheme::Env => self.resolve_env(),
            KeyRefScheme::File => self.resolve_file(),
        }
    }

    /// Resolve environment variable.
    fn resolve_env(&self) -> Result<Vec<u8>, KeyRefError> {
        let value =
            env::var(&self.path).map_err(|_| KeyRefError::EnvNotFound(self.path.clone()))?;

        // Try base64 decode first, fall back to raw bytes
        if let Ok(decoded) = base64_decode(&value) {
            Ok(decoded)
        } else {
            Ok(value.into_bytes())
        }
    }

    /// Resolve file path.
    fn resolve_file(&self) -> Result<Vec<u8>, KeyRefError> {
        let path = if self.path.starts_with('/') {
            Path::new(&self.path)
        } else {
            // Handle file:// without leading slash on Windows
            Path::new(&self.path)
        };

        if !path.exists() {
            return Err(KeyRefError::FileNotFound(self.path.clone()));
        }

        fs::read(path).map_err(|e| KeyRefError::FileReadError(e.to_string()))
    }

    /// Get the scheme.
    pub fn scheme(&self) -> KeyRefScheme {
        self.scheme
    }

    /// Get the path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Convert to URI string.
    pub fn to_uri(&self) -> String {
        format!("{}://{}", self.scheme.as_str(), self.path)
    }
}

/// Decode base64 string.
fn base64_decode(s: &str) -> Result<Vec<u8>, ()> {
    use veil_crypto::OutputFormat;
    OutputFormat::Base64.decode(s).map_err(|_| ())
}

impl std::fmt::Display for KeyRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_uri())
    }
}

impl Serialize for KeyRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_uri())
    }
}

impl<'de> Deserialize<'de> for KeyRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        KeyRef::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyref_scheme_from_str() {
        assert_eq!(KeyRefScheme::from_str("env").unwrap(), KeyRefScheme::Env);
        assert_eq!(KeyRefScheme::from_str("file").unwrap(), KeyRefScheme::File);
        assert_eq!(KeyRefScheme::from_str("ENV").unwrap(), KeyRefScheme::Env);
        assert!(KeyRefScheme::from_str("unknown").is_err());
    }

    #[test]
    fn test_keyref_parse_env() {
        let keyref = KeyRef::parse("env://MY_KEY").unwrap();
        assert_eq!(keyref.scheme, KeyRefScheme::Env);
        assert_eq!(keyref.path, "MY_KEY");
    }

    #[test]
    fn test_keyref_parse_file() {
        let keyref = KeyRef::parse("file:///etc/veil/key").unwrap();
        assert_eq!(keyref.scheme, KeyRefScheme::File);
        assert_eq!(keyref.path, "/etc/veil/key");
    }

    #[test]
    fn test_keyref_parse_invalid_format() {
        assert!(KeyRef::parse("invalid").is_err());
        assert!(KeyRef::parse("env://").is_err());
    }

    #[test]
    fn test_keyref_to_uri() {
        let keyref = KeyRef::env("MY_KEY");
        assert_eq!(keyref.to_uri(), "env://MY_KEY");

        let keyref = KeyRef::file("/path/to/key");
        assert_eq!(keyref.to_uri(), "file:///path/to/key");
    }

    #[test]
    fn test_keyref_resolve_env() {
        env::set_var("TEST_VEIL_KEY", "test-key-value");
        let keyref = KeyRef::env("TEST_VEIL_KEY");
        let resolved = keyref.resolve().unwrap();
        assert_eq!(resolved, b"test-key-value");
        env::remove_var("TEST_VEIL_KEY");
    }

    #[test]
    fn test_keyref_resolve_env_not_found() {
        let keyref = KeyRef::env("NONEXISTENT_KEY_12345");
        let result = keyref.resolve();
        assert!(matches!(result, Err(KeyRefError::EnvNotFound(_))));
    }

    #[test]
    fn test_keyref_resolve_file_not_found() {
        let keyref = KeyRef::file("/nonexistent/path/to/key");
        let result = keyref.resolve();
        assert!(matches!(result, Err(KeyRefError::FileNotFound(_))));
    }

    #[test]
    fn test_keyref_serde_roundtrip() {
        let keyref = KeyRef::env("MY_KEY");
        let json = serde_json::to_string(&keyref).unwrap();
        assert_eq!(json, "\"env://MY_KEY\"");

        let parsed: KeyRef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, keyref);
    }
}
