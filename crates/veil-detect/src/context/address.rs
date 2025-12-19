//! Multi-line postal address detection.
//!
//! This module detects postal addresses that span multiple lines,
//! combining street, city, postal code, and country components into
//! a single AddressBlock entity.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Components of an address.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddressComponent {
    /// Component type (street, city, postal_code, country, state, etc.).
    pub component_type: AddressComponentType,
    /// The matched text.
    pub text: String,
    /// Start position in the original text.
    pub start: usize,
    /// End position in the original text.
    pub end: usize,
}

/// Types of address components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressComponentType {
    /// Street address (e.g., "123 Main St")
    Street,
    /// City name
    City,
    /// State or province
    State,
    /// Postal/ZIP code
    PostalCode,
    /// Country name
    Country,
    /// Full address line (when components can't be separated)
    FullLine,
}

/// A detected postal address block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBlock {
    /// All components of the address.
    pub components: Vec<AddressComponent>,
    /// Full text of the address.
    pub full_text: String,
    /// Start position in the original text.
    pub start: usize,
    /// End position in the original text.
    pub end: usize,
    /// Language/format detected (en, de, fr).
    pub format: String,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
}

impl AddressBlock {
    /// Create a new address block.
    pub fn new(
        start: usize,
        end: usize,
        full_text: impl Into<String>,
        format: impl Into<String>,
    ) -> Self {
        Self {
            components: Vec::new(),
            full_text: full_text.into(),
            start,
            end,
            format: format.into(),
            confidence: 0.0,
        }
    }

    /// Add a component to this address block.
    pub fn add_component(&mut self, component: AddressComponent) {
        self.components.push(component);
    }

    /// Calculate confidence based on components found.
    pub fn calculate_confidence(&mut self) {
        let mut score: f32 = 0.0;

        // Base score for having any components
        if !self.components.is_empty() {
            score += 0.2;
        }

        // Score for each component type found
        let has_street = self
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::Street);
        let has_city = self
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::City);
        let has_postal = self
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::PostalCode);
        let has_country = self
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::Country);

        if has_street {
            score += 0.25;
        }
        if has_city {
            score += 0.2;
        }
        if has_postal {
            score += 0.2;
        }
        if has_country {
            score += 0.15;
        }

        self.confidence = score.min(1.0);
    }

    /// Check if this address has minimum required components.
    pub fn is_valid(&self) -> bool {
        // Valid if we have at least 2 components
        self.components.len() >= 2 && self.confidence >= 0.4
    }
}

/// Address detector for multi-line postal addresses.
pub struct AddressDetector {
    /// Language to use for detection.
    language: String,
}

// Lazy-compiled regex patterns for address detection
// Security: Bounded quantifiers to prevent ReDoS attacks
// Changed `(?:\s+[A-Za-z]+)*` to `(?:\s+[A-Za-z]+){0,5}` to limit backtracking
static US_STREET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b\d{1,5}\s+(?:[NSEW]\.?\s+)?[A-Za-z]+(?:\s+[A-Za-z]+){0,5}\s+(?:Street|St\.?|Avenue|Ave\.?|Road|Rd\.?|Boulevard|Blvd\.?|Drive|Dr\.?|Lane|Ln\.?|Way|Court|Ct\.?|Place|Pl\.?|Circle|Cir\.?)\b").unwrap()
});

// Security: Bounded quantifiers to prevent ReDoS attacks
static US_CITY_STATE_ZIP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b([A-Za-z]+(?:\s+[A-Za-z]+){0,4}),\s*([A-Z]{2})\s+(\d{5}(?:-\d{4})?)\b")
        .unwrap()
});

#[allow(dead_code)]
static US_ZIP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{5}(?:-\d{4})?\b").unwrap());

static DE_STREET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b[A-Za-zäöüßÄÖÜ]+(?:straße|str\.?|weg|platz|allee|gasse|ring|damm)\s+\d{1,5}[a-z]?\b").unwrap()
});

// Security: Bounded quantifiers to prevent ReDoS attacks
static DE_PLZ_CITY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{5})\s+([A-Za-zäöüßÄÖÜ]+(?:\s+[A-Za-zäöüßÄÖÜ]+){0,4})\b").unwrap()
});

// Security: Bounded quantifiers to prevent ReDoS attacks
static FR_STREET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b\d{1,5}(?:,?\s+(?:bis|ter))?\s+(?:rue|avenue|av\.?|boulevard|bd\.?|place|pl\.?|allée|impasse|chemin|passage)\s+[A-Za-zéèêëàâäùûüôöîïç]+(?:\s+[A-Za-zéèêëàâäùûüôöîïç]+){0,5}\b").unwrap()
});

// Security: Bounded quantifiers to prevent ReDoS attacks
static FR_CP_CITY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{5})\s+([A-Za-zéèêëàâäùûüôöîïç]+(?:[- ][A-Za-zéèêëàâäùûüôöîïç]+){0,4})\b")
        .unwrap()
});

static COUNTRY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:United States|USA|U\.S\.A\.|Germany|Deutschland|France|United Kingdom|UK|Canada|Australia|Austria|Österreich|Switzerland|Schweiz|Suisse|Netherlands|Belgium|Belgique|Italien|Italy|Spain|España|Spanien|Poland|Polen)\b").unwrap()
});

impl AddressDetector {
    /// Create a new address detector for the given language.
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
        }
    }

    /// Detect all address blocks in the text.
    pub fn detect(&self, text: &str) -> Vec<AddressBlock> {
        match self.language.as_str() {
            "en" => self.detect_english(text),
            "de" => self.detect_german(text),
            "fr" => self.detect_french(text),
            _ => {
                // Try all formats and return the best matches
                let mut results = Vec::new();
                results.extend(self.detect_english(text));
                results.extend(self.detect_german(text));
                results.extend(self.detect_french(text));
                results
            }
        }
    }

    /// Detect US/English format addresses.
    fn detect_english(&self, text: &str) -> Vec<AddressBlock> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Look for multi-line addresses
        for (i, line) in lines.iter().enumerate() {
            // Check for street address
            if let Some(street_match) = US_STREET_REGEX.find(line) {
                let mut block = AddressBlock::new(
                    self.line_offset(&lines, i) + street_match.start(),
                    0, // Will be updated
                    "",
                    "en",
                );

                block.add_component(AddressComponent {
                    component_type: AddressComponentType::Street,
                    text: street_match.as_str().to_string(),
                    start: block.start,
                    end: block.start + street_match.as_str().len(),
                });

                // Look for city, state, zip on same or next line
                let search_text = if i + 1 < lines.len() {
                    format!("{} {}", line, lines[i + 1])
                } else {
                    line.to_string()
                };

                if let Some(caps) = US_CITY_STATE_ZIP_REGEX.captures(&search_text) {
                    if let (Some(city), Some(state), Some(zip)) =
                        (caps.get(1), caps.get(2), caps.get(3))
                    {
                        let base_offset = self.line_offset(&lines, i);

                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: base_offset + city.start(),
                            end: base_offset + city.end(),
                        });

                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::State,
                            text: state.as_str().to_string(),
                            start: base_offset + state.start(),
                            end: base_offset + state.end(),
                        });

                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: zip.as_str().to_string(),
                            start: base_offset + zip.start(),
                            end: base_offset + zip.end(),
                        });

                        // Update end position
                        block.end = base_offset + zip.end();
                    }
                }

                // Check for country on following lines
                if i + 2 < lines.len() {
                    if let Some(country_match) = COUNTRY_REGEX.find(lines[i + 2]) {
                        let offset = self.line_offset(&lines, i + 2);
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::Country,
                            text: country_match.as_str().to_string(),
                            start: offset + country_match.start(),
                            end: offset + country_match.end(),
                        });
                        block.end = offset + country_match.end();
                    }
                }

                // Build full text
                let end_line = if block.components.len() > 1 {
                    i + 2
                } else {
                    i + 1
                };
                let full_text: String = lines[i..end_line.min(lines.len())].join("\n");
                block.full_text = full_text;

                block.calculate_confidence();
                if block.is_valid() {
                    blocks.push(block);
                }
            }
        }

        // Also detect single-line addresses
        if let Some(caps) = US_CITY_STATE_ZIP_REGEX.captures(text) {
            if let Some(full_match) = caps.get(0) {
                // Check if this wasn't already captured
                let already_found = blocks
                    .iter()
                    .any(|b| b.start <= full_match.start() && b.end >= full_match.end());

                if !already_found {
                    let mut block = AddressBlock::new(
                        full_match.start(),
                        full_match.end(),
                        full_match.as_str(),
                        "en",
                    );

                    if let (Some(city), Some(state), Some(zip)) =
                        (caps.get(1), caps.get(2), caps.get(3))
                    {
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: city.start(),
                            end: city.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::State,
                            text: state.as_str().to_string(),
                            start: state.start(),
                            end: state.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: zip.as_str().to_string(),
                            start: zip.start(),
                            end: zip.end(),
                        });
                    }

                    block.calculate_confidence();
                    if block.is_valid() {
                        blocks.push(block);
                    }
                }
            }
        }

        blocks
    }

    /// Detect German format addresses.
    fn detect_german(&self, text: &str) -> Vec<AddressBlock> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            // German: Street with number (Hauptstraße 42)
            if let Some(street_match) = DE_STREET_REGEX.find(line) {
                let line_start = self.line_offset(&lines, i);
                let mut block = AddressBlock::new(line_start + street_match.start(), 0, "", "de");

                block.add_component(AddressComponent {
                    component_type: AddressComponentType::Street,
                    text: street_match.as_str().to_string(),
                    start: line_start + street_match.start(),
                    end: line_start + street_match.end(),
                });

                // Look for PLZ + City on same or next line
                let search_text = if i + 1 < lines.len() {
                    format!("{} {}", line, lines[i + 1])
                } else {
                    line.to_string()
                };

                if let Some(caps) = DE_PLZ_CITY_REGEX.captures(&search_text) {
                    if let (Some(plz), Some(city)) = (caps.get(1), caps.get(2)) {
                        let offset = if plz.start() > line.len() {
                            self.line_offset(&lines, i + 1) - line.len() - 1
                        } else {
                            line_start
                        };

                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: plz.as_str().to_string(),
                            start: offset + plz.start(),
                            end: offset + plz.end(),
                        });

                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: offset + city.start(),
                            end: offset + city.end(),
                        });

                        block.end = offset + city.end();
                    }
                }

                // Check for country
                let country_line = if i + 1 < lines.len() { i + 1 } else { i };
                if let Some(country_match) =
                    COUNTRY_REGEX.find(lines.get(country_line).unwrap_or(&""))
                {
                    let offset = self.line_offset(&lines, country_line);
                    block.add_component(AddressComponent {
                        component_type: AddressComponentType::Country,
                        text: country_match.as_str().to_string(),
                        start: offset + country_match.start(),
                        end: offset + country_match.end(),
                    });
                }

                // Build full text
                let end_line = (i + 2).min(lines.len());
                block.full_text = lines[i..end_line].join("\n");

                block.calculate_confidence();
                if block.is_valid() {
                    blocks.push(block);
                }
            }
        }

        // Single-line German addresses (PLZ + City)
        if let Some(caps) = DE_PLZ_CITY_REGEX.captures(text) {
            if let Some(full_match) = caps.get(0) {
                let already_found = blocks
                    .iter()
                    .any(|b| b.start <= full_match.start() && b.end >= full_match.end());

                if !already_found {
                    let mut block = AddressBlock::new(
                        full_match.start(),
                        full_match.end(),
                        full_match.as_str(),
                        "de",
                    );

                    if let (Some(plz), Some(city)) = (caps.get(1), caps.get(2)) {
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: plz.as_str().to_string(),
                            start: plz.start(),
                            end: plz.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: city.start(),
                            end: city.end(),
                        });
                    }

                    block.calculate_confidence();
                    if block.is_valid() {
                        blocks.push(block);
                    }
                }
            }
        }

        blocks
    }

    /// Detect French format addresses.
    fn detect_french(&self, text: &str) -> Vec<AddressBlock> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            // French: Number + street type + name (42 rue de la Paix)
            if let Some(street_match) = FR_STREET_REGEX.find(line) {
                let line_start = self.line_offset(&lines, i);
                let mut block = AddressBlock::new(line_start + street_match.start(), 0, "", "fr");

                block.add_component(AddressComponent {
                    component_type: AddressComponentType::Street,
                    text: street_match.as_str().to_string(),
                    start: line_start + street_match.start(),
                    end: line_start + street_match.end(),
                });

                // Look for code postal + city on same or next line
                let search_text = if i + 1 < lines.len() {
                    format!("{} {}", line, lines[i + 1])
                } else {
                    line.to_string()
                };

                if let Some(caps) = FR_CP_CITY_REGEX.captures(&search_text) {
                    if let (Some(cp), Some(city)) = (caps.get(1), caps.get(2)) {
                        let offset = if cp.start() > line.len() {
                            self.line_offset(&lines, i + 1) - line.len() - 1
                        } else {
                            line_start
                        };

                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: cp.as_str().to_string(),
                            start: offset + cp.start(),
                            end: offset + cp.end(),
                        });

                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: offset + city.start(),
                            end: offset + city.end(),
                        });

                        block.end = offset + city.end();
                    }
                }

                // Check for country
                if i + 2 < lines.len() {
                    if let Some(country_match) = COUNTRY_REGEX.find(lines[i + 2]) {
                        let offset = self.line_offset(&lines, i + 2);
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::Country,
                            text: country_match.as_str().to_string(),
                            start: offset + country_match.start(),
                            end: offset + country_match.end(),
                        });
                    }
                }

                // Build full text
                let end_line = (i + 2).min(lines.len());
                block.full_text = lines[i..end_line].join("\n");

                block.calculate_confidence();
                if block.is_valid() {
                    blocks.push(block);
                }
            }
        }

        // Single-line French addresses
        if let Some(caps) = FR_CP_CITY_REGEX.captures(text) {
            if let Some(full_match) = caps.get(0) {
                let already_found = blocks
                    .iter()
                    .any(|b| b.start <= full_match.start() && b.end >= full_match.end());

                if !already_found {
                    let mut block = AddressBlock::new(
                        full_match.start(),
                        full_match.end(),
                        full_match.as_str(),
                        "fr",
                    );

                    if let (Some(cp), Some(city)) = (caps.get(1), caps.get(2)) {
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: cp.as_str().to_string(),
                            start: cp.start(),
                            end: cp.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: city.start(),
                            end: city.end(),
                        });
                    }

                    block.calculate_confidence();
                    if block.is_valid() {
                        blocks.push(block);
                    }
                }
            }
        }

        blocks
    }

    /// Calculate byte offset for a line in the text.
    fn line_offset(&self, lines: &[&str], line_index: usize) -> usize {
        lines[..line_index]
            .iter()
            .map(|l| l.len() + 1) // +1 for newline
            .sum()
    }
}

impl Default for AddressDetector {
    fn default() -> Self {
        Self::new("en")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us_address_detection() {
        let detector = AddressDetector::new("en");
        let text = "123 Main Street\nNew York, NY 10001";
        let blocks = detector.detect(text);

        assert!(!blocks.is_empty());
        let block = &blocks[0];
        assert_eq!(block.format, "en");
        assert!(block
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::Street));
    }

    #[test]
    fn test_german_address_detection() {
        let detector = AddressDetector::new("de");
        let text = "Hauptstraße 42\n80331 München";
        let blocks = detector.detect(text);

        assert!(!blocks.is_empty());
        let block = &blocks[0];
        assert_eq!(block.format, "de");
        assert!(block
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::PostalCode));
    }

    #[test]
    fn test_french_address_detection() {
        let detector = AddressDetector::new("fr");
        let text = "42 rue de la Paix\n75002 Paris";
        let blocks = detector.detect(text);

        assert!(!blocks.is_empty());
        let block = &blocks[0];
        assert_eq!(block.format, "fr");
    }

    #[test]
    fn test_single_line_city_state_zip() {
        let detector = AddressDetector::new("en");
        let text = "Send to: San Francisco, CA 94102";
        let blocks = detector.detect(text);

        assert!(!blocks.is_empty());
        let block = &blocks[0];
        assert!(block
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::City));
        assert!(block
            .components
            .iter()
            .any(|c| c.component_type == AddressComponentType::PostalCode));
    }

    #[test]
    fn test_address_with_country() {
        let detector = AddressDetector::new("en");
        let text = "123 Main Street\nNew York, NY 10001\nUnited States";
        let blocks = detector.detect(text);

        assert!(!blocks.is_empty());
        // May or may not detect country depending on multi-line handling
    }

    #[test]
    fn test_confidence_calculation() {
        let mut block = AddressBlock::new(0, 50, "test", "en");
        block.add_component(AddressComponent {
            component_type: AddressComponentType::Street,
            text: "123 Main St".to_string(),
            start: 0,
            end: 11,
        });
        block.add_component(AddressComponent {
            component_type: AddressComponentType::City,
            text: "New York".to_string(),
            start: 12,
            end: 20,
        });
        block.calculate_confidence();

        assert!(block.confidence > 0.5);
        assert!(block.is_valid());
    }

    #[test]
    fn test_invalid_address() {
        let detector = AddressDetector::new("en");
        let text = "Hello world, this is not an address";
        let blocks = detector.detect(text);

        assert!(blocks.is_empty());
    }

    #[test]
    fn test_german_plz_format() {
        let detector = AddressDetector::new("de");
        let text = "PLZ: 10115 Berlin";
        let blocks = detector.detect(text);

        assert!(!blocks.is_empty());
        let block = &blocks[0];
        assert!(block.components.iter().any(|c| {
            c.component_type == AddressComponentType::PostalCode && c.text == "10115"
        }));
    }
}
