//! Cached detector registry for efficient repeated scans.
//!
//! The detector registry is expensive to create (regex compilation),
//! so we cache it for the lifetime of the WASM module.

use std::sync::OnceLock;
use veil_detect::DetectorRegistry;

/// Global cached detector registry.
static REGISTRY: OnceLock<DetectorRegistry> = OnceLock::new();

/// Get the cached detector registry.
///
/// This initializes the registry on first access and returns
/// the same instance for all subsequent calls.
pub fn get_registry() -> &'static DetectorRegistry {
    REGISTRY.get_or_init(DetectorRegistry::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_caching() {
        let reg1 = get_registry();
        let reg2 = get_registry();

        // Should return the same instance
        assert!(std::ptr::eq(reg1, reg2));
    }

    #[test]
    fn test_registry_functional() {
        let registry = get_registry();
        let segment = veil_parsers::TextSegment {
            content: "Contact: test@example.com".to_string().into(),
            position: veil_parsers::Position::Text {
                line: 1,
                column: 1,
                byte_offset: 0,
                byte_length: 25,
            },
        };

        let findings = registry.detect_all(&[segment]);
        assert!(!findings.is_empty());
    }
}
