//! Pseudonymization with fake data generation.

use fake::faker::address::en::*;
use fake::faker::company::en::*;
use fake::faker::internet::en::*;
use fake::faker::name::en::*;
use fake::faker::phone_number::en::*;
use fake::Fake;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::CryptoError;
use crate::types::{CryptoMetadata, CryptoResult, ProtectionMode};

/// Type of data to generate for pseudonymization.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PseudonymDataType {
    /// Full name (first + last).
    #[default]
    Name,
    /// First name only.
    FirstName,
    /// Last name only.
    LastName,
    /// Email address.
    Email,
    /// Phone number.
    Phone,
    /// Full address.
    Address,
    /// City name.
    City,
    /// Postal/ZIP code.
    PostalCode,
    /// Company name.
    Company,
    /// Generic alphanumeric string.
    Generic,
}

impl PseudonymDataType {
    /// Get the data type as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            PseudonymDataType::Name => "name",
            PseudonymDataType::FirstName => "first_name",
            PseudonymDataType::LastName => "last_name",
            PseudonymDataType::Email => "email",
            PseudonymDataType::Phone => "phone",
            PseudonymDataType::Address => "address",
            PseudonymDataType::City => "city",
            PseudonymDataType::PostalCode => "postal_code",
            PseudonymDataType::Company => "company",
            PseudonymDataType::Generic => "generic",
        }
    }
}

/// Configuration for pseudonymization operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PseudonymConfig {
    /// Locale for generating fake data (e.g., "de_DE", "en_US").
    pub locale: String,
    /// Seed for deterministic pseudonymization (optional).
    pub seed: Option<u64>,
    /// Data type for generating appropriate fake data.
    pub data_type: PseudonymDataType,
    /// Whether to maintain consistency (same input → same output).
    pub consistent: bool,
}

impl Default for PseudonymConfig {
    fn default() -> Self {
        Self {
            locale: "en_US".to_string(),
            seed: None,
            data_type: PseudonymDataType::Name,
            consistent: false,
        }
    }
}

impl PseudonymConfig {
    /// Create a new config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the locale.
    pub fn with_locale(mut self, locale: &str) -> Self {
        self.locale = locale.to_string();
        self
    }

    /// Set the data type.
    pub fn with_type(mut self, data_type: PseudonymDataType) -> Self {
        self.data_type = data_type;
        self
    }

    /// Enable consistent mode with a seed.
    pub fn consistent_with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self.consistent = true;
        self
    }

    /// Enable consistent mode (seed will be derived from input).
    pub fn consistent(mut self) -> Self {
        self.consistent = true;
        self
    }
}

/// Thread-safe cache for consistent pseudonymization.
struct ConsistencyCache {
    cache: Mutex<HashMap<String, String>>,
}

impl ConsistencyCache {
    fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        self.cache.lock().ok()?.get(key).cloned()
    }

    fn insert(&self, key: String, value: String) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, value);
        }
    }
}

thread_local! {
    static CACHE: ConsistencyCache = ConsistencyCache::new();
}

/// Pseudonymize data by replacing it with fake data.
///
/// # Arguments
/// * `value` - Original value to pseudonymize
/// * `config` - Pseudonymization configuration
///
/// # Returns
/// * `Ok(CryptoResult)` - Pseudonymized value
/// * `Err(CryptoError)` - If pseudonymization fails
///
/// # Example
/// ```rust,ignore
/// use veil_crypto::{pseudonymize, PseudonymConfig, PseudonymDataType};
///
/// let config = PseudonymConfig::new()
///     .with_locale("de_DE")
///     .with_type(PseudonymDataType::Name);
/// let result = pseudonymize("Max Müller", &config)?;
/// ```
pub fn pseudonymize(value: &str, config: &PseudonymConfig) -> Result<CryptoResult, CryptoError> {
    // Create cache key for consistency
    let cache_key = if config.consistent {
        format!(
            "{}:{}:{}:{}",
            value,
            config.locale,
            config.data_type.as_str(),
            config.seed.unwrap_or(0)
        )
    } else {
        String::new()
    };

    // Check cache first for consistent mode
    if config.consistent {
        if let Some(cached) = CACHE.with(|c| c.get(&cache_key)) {
            let metadata = CryptoMetadata {
                data_type: Some(config.data_type.as_str().to_string()),
                ..Default::default()
            };
            return Ok(CryptoResult::with_metadata(
                cached,
                ProtectionMode::Pseudonymize,
                metadata,
            ));
        }
    }

    // Determine seed for RNG
    let seed = if config.consistent {
        config.seed.unwrap_or_else(|| derive_seed_from_input(value))
    } else {
        // Random seed
        rand::random()
    };

    // Generate fake data
    let fake_value = generate_fake_data(config.data_type, &config.locale, seed)?;

    // Cache for consistency
    if config.consistent {
        CACHE.with(|c| c.insert(cache_key, fake_value.clone()));
    }

    let metadata = CryptoMetadata {
        data_type: Some(config.data_type.as_str().to_string()),
        ..Default::default()
    };

    Ok(CryptoResult::with_metadata(
        fake_value,
        ProtectionMode::Pseudonymize,
        metadata,
    ))
}

/// Derive a seed from input string using hash.
fn derive_seed_from_input(input: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    u64::from_le_bytes(hash[0..8].try_into().unwrap_or([0; 8]))
}

/// Generate fake data based on type and locale.
fn generate_fake_data(
    data_type: PseudonymDataType,
    locale: &str,
    seed: u64,
) -> Result<String, CryptoError> {
    let mut rng = StdRng::seed_from_u64(seed);

    let result = if locale.starts_with("de") {
        generate_german(&mut rng, data_type)
    } else {
        generate_english(&mut rng, data_type)
    };

    Ok(result)
}

/// Generate German locale fake data.
fn generate_german(rng: &mut StdRng, data_type: PseudonymDataType) -> String {
    match data_type {
        PseudonymDataType::Name => {
            let first: String = FirstName().fake_with_rng(rng);
            let last: String = LastName().fake_with_rng(rng);
            format!("{} {}", first, last)
        }
        PseudonymDataType::FirstName => FirstName().fake_with_rng(rng),
        PseudonymDataType::LastName => LastName().fake_with_rng(rng),
        PseudonymDataType::Email => {
            let first: String = FirstName().fake_with_rng(rng);
            let last: String = LastName().fake_with_rng(rng);
            format!(
                "{}.{}@beispiel.de",
                first.to_lowercase(),
                last.to_lowercase()
            )
        }
        PseudonymDataType::Phone => "+49 ".to_string() + &generate_phone_digits(rng),
        PseudonymDataType::Address => {
            let street: String = StreetName().fake_with_rng(rng);
            let num: u32 = (1..200).fake_with_rng(rng);
            let city: String = CityName().fake_with_rng(rng);
            let zip: String = generate_german_zip(rng);
            format!("{} {}, {} {}", street, num, zip, city)
        }
        PseudonymDataType::City => CityName().fake_with_rng(rng),
        PseudonymDataType::PostalCode => generate_german_zip(rng),
        PseudonymDataType::Company => CompanyName().fake_with_rng(rng),
        PseudonymDataType::Generic => generate_generic(rng),
    }
}

/// Generate English locale fake data.
fn generate_english(rng: &mut StdRng, data_type: PseudonymDataType) -> String {
    match data_type {
        PseudonymDataType::Name => {
            let first: String = FirstName().fake_with_rng(rng);
            let last: String = LastName().fake_with_rng(rng);
            format!("{} {}", first, last)
        }
        PseudonymDataType::FirstName => FirstName().fake_with_rng(rng),
        PseudonymDataType::LastName => LastName().fake_with_rng(rng),
        PseudonymDataType::Email => FreeEmail().fake_with_rng(rng),
        PseudonymDataType::Phone => PhoneNumber().fake_with_rng(rng),
        PseudonymDataType::Address => {
            let street: String = StreetName().fake_with_rng(rng);
            let num: u32 = (1..9999).fake_with_rng(rng);
            let city: String = CityName().fake_with_rng(rng);
            let zip: String = PostCode().fake_with_rng(rng);
            format!("{} {}, {}, {}", num, street, city, zip)
        }
        PseudonymDataType::City => CityName().fake_with_rng(rng),
        PseudonymDataType::PostalCode => PostCode().fake_with_rng(rng),
        PseudonymDataType::Company => CompanyName().fake_with_rng(rng),
        PseudonymDataType::Generic => generate_generic(rng),
    }
}

/// Generate German postal code (5 digits).
fn generate_german_zip(rng: &mut StdRng) -> String {
    use rand::Rng;
    format!("{:05}", rng.gen_range(10000..99999))
}

/// Generate phone number digits.
fn generate_phone_digits(rng: &mut StdRng) -> String {
    let area: u32 = (100..999).fake_with_rng(rng);
    let num: u32 = (1000000..9999999).fake_with_rng(rng);
    format!("{} {}", area, num)
}

/// Generate a generic alphanumeric pseudonym.
fn generate_generic(rng: &mut StdRng) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let len = rng.gen_range(8..16);
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pseudonymize_name_produces_different_name() {
        let config = PseudonymConfig::new().with_type(PseudonymDataType::Name);
        let original = "Max Müller";

        let result = pseudonymize(original, &config).unwrap();

        assert_eq!(result.mode, ProtectionMode::Pseudonymize);
        assert_ne!(result.value, original);
        // Should contain a space (first + last name)
        assert!(result.value.contains(' '));
    }

    #[test]
    fn test_pseudonymize_email_produces_valid_email() {
        let config = PseudonymConfig::new().with_type(PseudonymDataType::Email);
        let original = "max@example.com";

        let result = pseudonymize(original, &config).unwrap();

        assert!(result.value.contains('@'));
        assert!(result.value.contains('.'));
    }

    #[test]
    fn test_locale_affects_output() {
        let config_de = PseudonymConfig::new()
            .with_locale("de_DE")
            .with_type(PseudonymDataType::Phone)
            .consistent_with_seed(42);

        let config_en = PseudonymConfig::new()
            .with_locale("en_US")
            .with_type(PseudonymDataType::Phone)
            .consistent_with_seed(42);

        let result_de = pseudonymize("phone", &config_de).unwrap();
        let result_en = pseudonymize("phone", &config_en).unwrap();

        // German phone should start with +49
        assert!(result_de.value.starts_with("+49"));
        // They should be different formats
        assert_ne!(result_de.value, result_en.value);
    }

    #[test]
    fn test_consistent_mode_returns_same_output() {
        let config = PseudonymConfig::new()
            .with_type(PseudonymDataType::Name)
            .consistent_with_seed(12345);
        let original = "John Doe";

        let result1 = pseudonymize(original, &config).unwrap();
        let result2 = pseudonymize(original, &config).unwrap();

        assert_eq!(result1.value, result2.value);
    }

    #[test]
    fn test_different_seeds_produce_different_output() {
        let config1 = PseudonymConfig::new()
            .with_type(PseudonymDataType::Name)
            .consistent_with_seed(111);

        let config2 = PseudonymConfig::new()
            .with_type(PseudonymDataType::Name)
            .consistent_with_seed(222);

        let original = "Jane Doe";

        let result1 = pseudonymize(original, &config1).unwrap();
        let result2 = pseudonymize(original, &config2).unwrap();

        assert_ne!(result1.value, result2.value);
    }

    #[test]
    fn test_input_derived_seed_consistency() {
        // Without explicit seed, consistent mode should derive from input
        let config = PseudonymConfig::new()
            .with_type(PseudonymDataType::Name)
            .consistent();

        let original = "Consistent Name Test";

        let result1 = pseudonymize(original, &config).unwrap();
        let result2 = pseudonymize(original, &config).unwrap();

        assert_eq!(result1.value, result2.value);
    }

    #[test]
    fn test_metadata_contains_data_type() {
        let config = PseudonymConfig::new().with_type(PseudonymDataType::Email);
        let result = pseudonymize("test", &config).unwrap();

        assert_eq!(result.metadata.data_type, Some("email".to_string()));
    }

    #[test]
    fn test_all_data_types() {
        let types = vec![
            PseudonymDataType::Name,
            PseudonymDataType::FirstName,
            PseudonymDataType::LastName,
            PseudonymDataType::Email,
            PseudonymDataType::Phone,
            PseudonymDataType::Address,
            PseudonymDataType::City,
            PseudonymDataType::PostalCode,
            PseudonymDataType::Company,
            PseudonymDataType::Generic,
        ];

        for data_type in types {
            let config = PseudonymConfig::new().with_type(data_type);
            let result = pseudonymize("test input", &config);
            assert!(result.is_ok(), "Failed for type {:?}", data_type);
            assert!(!result.unwrap().value.is_empty());
        }
    }

    #[test]
    fn test_german_locale_city() {
        let config = PseudonymConfig::new()
            .with_locale("de_DE")
            .with_type(PseudonymDataType::City);

        let result = pseudonymize("Berlin", &config).unwrap();
        assert!(!result.value.is_empty());
    }
}
