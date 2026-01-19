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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn with_env_var<T>(name: &str, value: Option<&str>, f: impl FnOnce() -> T) -> T {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        let old = std::env::var_os(name);
        match value {
            Some(v) => std::env::set_var(name, v),
            None => std::env::remove_var(name),
        }

        let result = f();

        match old {
            Some(v) => std::env::set_var(name, v),
            None => std::env::remove_var(name),
        }

        result
    }

    #[test]
    fn test_parse_env_bool_truthy_values() {
        let name = "VEIL_TEST_PARSE_ENV_BOOL_TRUTHY";
        for value in ["1", "true", "TRUE", " yes ", "YeS"] {
            with_env_var(name, Some(value), || {
                assert!(parse_env_bool(name), "expected truthy for {value:?}");
            });
        }
    }

    #[test]
    fn test_parse_env_bool_falsey_values() {
        let name = "VEIL_TEST_PARSE_ENV_BOOL_FALSEY";
        for value in ["0", "false", "no", ""] {
            with_env_var(name, Some(value), || {
                assert!(!parse_env_bool(name), "expected falsey for {value:?}");
            });
        }

        with_env_var(name, None, || {
            assert!(!parse_env_bool(name));
        });
    }

    #[test]
    fn test_plaintext_storage_policy_from_env() {
        with_env_var(PLAINTEXT_STORAGE_ENV, None, || {
            assert!(!PlaintextStoragePolicy::from_env().is_allowed());
        });

        with_env_var(PLAINTEXT_STORAGE_ENV, Some("true"), || {
            assert!(PlaintextStoragePolicy::from_env().is_allowed());
        });

        with_env_var(PLAINTEXT_STORAGE_ENV, Some("no"), || {
            assert!(!PlaintextStoragePolicy::from_env().is_allowed());
        });
    }

    #[test]
    fn test_plaintext_storage_policy_explicit_constructors() {
        assert!(PlaintextStoragePolicy::allow_insecure().is_allowed());
        assert!(!PlaintextStoragePolicy::forbid().is_allowed());
    }
}
