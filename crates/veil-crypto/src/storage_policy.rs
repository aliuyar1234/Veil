//! Storage policy configuration.

const PLAINTEXT_STORAGE_ENV: &str = "VEIL_ALLOW_PLAINTEXT_STORAGE";

/// Policy controlling whether plaintext storage backends are allowed.
///
/// Plaintext storage is intentionally disabled by default.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaintextStoragePolicy {
    allow_plaintext_storage: bool,
}

impl PlaintextStoragePolicy {
    /// Explicitly allow plaintext storage (intended for development only).
    pub fn allow_insecure() -> Self {
        Self {
            allow_plaintext_storage: true,
        }
    }

    /// Explicitly forbid plaintext storage (default).
    pub fn forbid() -> Self {
        Self {
            allow_plaintext_storage: false,
        }
    }

    /// Build a policy from the `VEIL_ALLOW_PLAINTEXT_STORAGE` environment variable.
    pub fn from_env() -> Self {
        Self {
            allow_plaintext_storage: parse_env_bool(PLAINTEXT_STORAGE_ENV),
        }
    }

    pub(crate) fn is_allowed(self) -> bool {
        self.allow_plaintext_storage
    }
}

fn parse_env_bool(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false)
}
