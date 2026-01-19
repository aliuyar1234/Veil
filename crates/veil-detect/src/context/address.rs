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

/// Supported address formats for detection and metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AddressFormat {
    /// English/US-style addresses.
    En,
    /// German addresses.
    De,
    /// French addresses.
    Fr,
    /// Auto-detect by running all formats.
    Auto,
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
    /// Language/format detected.
    pub format: AddressFormat,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
}

impl AddressBlock {
    /// Create a new address block.
    pub fn new(
        start: usize,
        end: usize,
        full_text: impl Into<String>,
        format: AddressFormat,
    ) -> Self {
        Self {
            components: Vec::new(),
            full_text: full_text.into(),
            start,
            end,
            format,
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
    /// Address format to use for detection.
    format: AddressFormat,
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
    /// Create a new address detector for the given format.
    pub fn new(format: AddressFormat) -> Self {
        Self { format }
    }

    /// Detect all address blocks in the text.
    pub fn detect(&self, text: &str) -> Vec<AddressBlock> {
        match self.format {
            AddressFormat::En => self.detect_english(text),
            AddressFormat::De => self.detect_german(text),
            AddressFormat::Fr => self.detect_french(text),
            AddressFormat::Auto => {
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
        let line_offsets = self.line_offsets(text);

        // Look for multi-line addresses
        for (i, line) in lines.iter().enumerate() {
            // Check for street address
            if let Some(street_match) = US_STREET_REGEX.find(line) {
                let line_start = self.line_offset(&line_offsets, i);
                let mut block = AddressBlock::new(
                    line_start + street_match.start(),
                    0, // Will be updated
                    "",
                    AddressFormat::En,
                );
                let mut country_line: Option<usize> = None;
                let mut uses_next_line = false;

                block.add_component(AddressComponent {
                    component_type: AddressComponentType::Street,
                    text: street_match.as_str().to_string(),
                    start: line_start + street_match.start(),
                    end: line_start + street_match.end(),
                });

                // Prefer city/state/zip on the following line to avoid cross-line captures like
                // "Main Street New York, NY 10001".
                if i + 1 < lines.len() {
                    if let Some(caps) = US_CITY_STATE_ZIP_REGEX.captures(lines[i + 1]) {
                        uses_next_line = true;
                        if let (Some(city), Some(state), Some(zip)) =
                            (caps.get(1), caps.get(2), caps.get(3))
                        {
                            let base_offset = self.line_offset(&line_offsets, i + 1);
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
                        }
                    } else if let Some(caps) = US_CITY_STATE_ZIP_REGEX.captures(line) {
                        if let (Some(city), Some(state), Some(zip)) =
                            (caps.get(1), caps.get(2), caps.get(3))
                        {
                            block.add_component(AddressComponent {
                                component_type: AddressComponentType::City,
                                text: city.as_str().to_string(),
                                start: line_start + city.start(),
                                end: line_start + city.end(),
                            });
                            block.add_component(AddressComponent {
                                component_type: AddressComponentType::State,
                                text: state.as_str().to_string(),
                                start: line_start + state.start(),
                                end: line_start + state.end(),
                            });
                            block.add_component(AddressComponent {
                                component_type: AddressComponentType::PostalCode,
                                text: zip.as_str().to_string(),
                                start: line_start + zip.start(),
                                end: line_start + zip.end(),
                            });
                        }
                    }
                } else if let Some(caps) = US_CITY_STATE_ZIP_REGEX.captures(line) {
                    if let (Some(city), Some(state), Some(zip)) =
                        (caps.get(1), caps.get(2), caps.get(3))
                    {
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: line_start + city.start(),
                            end: line_start + city.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::State,
                            text: state.as_str().to_string(),
                            start: line_start + state.start(),
                            end: line_start + state.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: zip.as_str().to_string(),
                            start: line_start + zip.start(),
                            end: line_start + zip.end(),
                        });
                    }
                }

                // Check for country on following lines
                if i + 2 < lines.len() {
                    if let Some(country_match) = COUNTRY_REGEX.find(lines[i + 2]) {
                        let offset = self.line_offset(&line_offsets, i + 2);
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::Country,
                            text: country_match.as_str().to_string(),
                            start: offset + country_match.start(),
                            end: offset + country_match.end(),
                        });
                        country_line = Some(i + 2);
                    }
                }

                let mut last_line = i;
                if uses_next_line {
                    last_line = i + 1;
                }
                if let Some(cl) = country_line {
                    last_line = cl;
                }
                block.end = self.line_offset(&line_offsets, last_line) + lines[last_line].len();
                block.full_text = text
                    .get(block.start..block.end)
                    .unwrap_or_default()
                    .to_string();

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
                        AddressFormat::En,
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
        let line_offsets = self.line_offsets(text);

        for (i, line) in lines.iter().enumerate() {
            // German: Street with number (Hauptstraße 42)
            if let Some(street_match) = DE_STREET_REGEX.find(line) {
                let line_start = self.line_offset(&line_offsets, i);
                let mut block =
                    AddressBlock::new(line_start + street_match.start(), 0, "", AddressFormat::De);
                let mut uses_next_line = false;
                let mut country_line: Option<usize> = None;

                block.add_component(AddressComponent {
                    component_type: AddressComponentType::Street,
                    text: street_match.as_str().to_string(),
                    start: line_start + street_match.start(),
                    end: line_start + street_match.end(),
                });

                // Prefer PLZ + City on the following line to avoid cross-line captures like
                // "80331 Muenchen Germany".
                if i + 1 < lines.len() {
                    if let Some(caps) = DE_PLZ_CITY_REGEX.captures(lines[i + 1]) {
                        uses_next_line = true;
                        if let (Some(plz), Some(city)) = (caps.get(1), caps.get(2)) {
                            let offset = self.line_offset(&line_offsets, i + 1);
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
                        }
                    } else if let Some(caps) = DE_PLZ_CITY_REGEX.captures(line) {
                        if let (Some(plz), Some(city)) = (caps.get(1), caps.get(2)) {
                            block.add_component(AddressComponent {
                                component_type: AddressComponentType::PostalCode,
                                text: plz.as_str().to_string(),
                                start: line_start + plz.start(),
                                end: line_start + plz.end(),
                            });
                            block.add_component(AddressComponent {
                                component_type: AddressComponentType::City,
                                text: city.as_str().to_string(),
                                start: line_start + city.start(),
                                end: line_start + city.end(),
                            });
                        }
                    }
                } else if let Some(caps) = DE_PLZ_CITY_REGEX.captures(line) {
                    if let (Some(plz), Some(city)) = (caps.get(1), caps.get(2)) {
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: plz.as_str().to_string(),
                            start: line_start + plz.start(),
                            end: line_start + plz.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: line_start + city.start(),
                            end: line_start + city.end(),
                        });
                    }
                }

                // Check for country
                for candidate in [i + 2, i + 1, i] {
                    if candidate >= lines.len() {
                        continue;
                    }
                    if let Some(country_match) = COUNTRY_REGEX.find(lines[candidate]) {
                        let offset = self.line_offset(&line_offsets, candidate);
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::Country,
                            text: country_match.as_str().to_string(),
                            start: offset + country_match.start(),
                            end: offset + country_match.end(),
                        });
                        country_line = Some(candidate);
                        break;
                    }
                }

                let mut last_line = i;
                if uses_next_line {
                    last_line = i + 1;
                }
                if let Some(cl) = country_line {
                    last_line = cl;
                }
                block.end = self.line_offset(&line_offsets, last_line) + lines[last_line].len();
                block.full_text = text
                    .get(block.start..block.end)
                    .unwrap_or_default()
                    .to_string();

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
                        AddressFormat::De,
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
        let line_offsets = self.line_offsets(text);

        for (i, line) in lines.iter().enumerate() {
            // French: Number + street type + name (42 rue de la Paix)
            if let Some(street_match) = FR_STREET_REGEX.find(line) {
                let line_start = self.line_offset(&line_offsets, i);
                let mut block =
                    AddressBlock::new(line_start + street_match.start(), 0, "", AddressFormat::Fr);
                let mut uses_next_line = false;
                let mut country_line: Option<usize> = None;

                block.add_component(AddressComponent {
                    component_type: AddressComponentType::Street,
                    text: street_match.as_str().to_string(),
                    start: line_start + street_match.start(),
                    end: line_start + street_match.end(),
                });

                // Prefer code postal + city on the following line to avoid cross-line captures like
                // "75002 Paris Ignore".
                if i + 1 < lines.len() {
                    if let Some(caps) = FR_CP_CITY_REGEX.captures(lines[i + 1]) {
                        uses_next_line = true;
                        if let (Some(cp), Some(city)) = (caps.get(1), caps.get(2)) {
                            let offset = self.line_offset(&line_offsets, i + 1);
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
                        }
                    } else if let Some(caps) = FR_CP_CITY_REGEX.captures(line) {
                        if let (Some(cp), Some(city)) = (caps.get(1), caps.get(2)) {
                            block.add_component(AddressComponent {
                                component_type: AddressComponentType::PostalCode,
                                text: cp.as_str().to_string(),
                                start: line_start + cp.start(),
                                end: line_start + cp.end(),
                            });
                            block.add_component(AddressComponent {
                                component_type: AddressComponentType::City,
                                text: city.as_str().to_string(),
                                start: line_start + city.start(),
                                end: line_start + city.end(),
                            });
                        }
                    }
                } else if let Some(caps) = FR_CP_CITY_REGEX.captures(line) {
                    if let (Some(cp), Some(city)) = (caps.get(1), caps.get(2)) {
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::PostalCode,
                            text: cp.as_str().to_string(),
                            start: line_start + cp.start(),
                            end: line_start + cp.end(),
                        });
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::City,
                            text: city.as_str().to_string(),
                            start: line_start + city.start(),
                            end: line_start + city.end(),
                        });
                    }
                }

                // Check for country
                if i + 2 < lines.len() {
                    if let Some(country_match) = COUNTRY_REGEX.find(lines[i + 2]) {
                        let offset = self.line_offset(&line_offsets, i + 2);
                        block.add_component(AddressComponent {
                            component_type: AddressComponentType::Country,
                            text: country_match.as_str().to_string(),
                            start: offset + country_match.start(),
                            end: offset + country_match.end(),
                        });
                        country_line = Some(i + 2);
                    }
                }

                let mut last_line = i;
                if uses_next_line {
                    last_line = i + 1;
                }
                if let Some(cl) = country_line {
                    last_line = cl;
                }
                block.end = self.line_offset(&line_offsets, last_line) + lines[last_line].len();
                block.full_text = text
                    .get(block.start..block.end)
                    .unwrap_or_default()
                    .to_string();

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
                        AddressFormat::Fr,
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

    fn line_offsets(&self, text: &str) -> Vec<usize> {
        let mut offsets = Vec::new();
        offsets.push(0);
        for (idx, b) in text.bytes().enumerate() {
            if b == b'\n' {
                offsets.push(idx + 1);
            }
        }
        offsets
    }

    fn line_offset(&self, line_offsets: &[usize], line_index: usize) -> usize {
        line_offsets.get(line_index).copied().unwrap_or(0)
    }
}

impl Default for AddressDetector {
    fn default() -> Self {
        Self::new(AddressFormat::En)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx_eq(actual: f32, expected: f32) {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-6,
            "expected {expected}, got {actual} (diff {diff})"
        );
    }

    fn find_component<'a>(
        block: &'a AddressBlock,
        component_type: AddressComponentType,
    ) -> &'a AddressComponent {
        block
            .components
            .iter()
            .find(|c| c.component_type == component_type)
            .expect("missing component")
    }

    #[test]
    fn test_confidence_scores() {
        let mut block = AddressBlock::new(0, 0, "", AddressFormat::En);
        block.add_component(AddressComponent {
            component_type: AddressComponentType::Street,
            text: "123 Main Street".to_string(),
            start: 0,
            end: 0,
        });
        block.calculate_confidence();
        assert_approx_eq(block.confidence, 0.45);
        assert!(!block.is_valid());

        let mut block = AddressBlock::new(0, 0, "", AddressFormat::En);
        block.add_component(AddressComponent {
            component_type: AddressComponentType::City,
            text: "New York".to_string(),
            start: 0,
            end: 0,
        });
        block.calculate_confidence();
        assert_approx_eq(block.confidence, 0.4);

        let mut block = AddressBlock::new(0, 0, "", AddressFormat::En);
        block.add_component(AddressComponent {
            component_type: AddressComponentType::PostalCode,
            text: "10001".to_string(),
            start: 0,
            end: 0,
        });
        block.calculate_confidence();
        assert_approx_eq(block.confidence, 0.4);

        let mut block = AddressBlock::new(0, 0, "", AddressFormat::En);
        block.add_component(AddressComponent {
            component_type: AddressComponentType::Country,
            text: "United States".to_string(),
            start: 0,
            end: 0,
        });
        block.calculate_confidence();
        assert_approx_eq(block.confidence, 0.35);

        let mut block = AddressBlock::new(0, 0, "", AddressFormat::En);
        block.add_component(AddressComponent {
            component_type: AddressComponentType::Street,
            text: "123 Main Street".to_string(),
            start: 0,
            end: 0,
        });
        block.add_component(AddressComponent {
            component_type: AddressComponentType::City,
            text: "New York".to_string(),
            start: 0,
            end: 0,
        });
        block.add_component(AddressComponent {
            component_type: AddressComponentType::PostalCode,
            text: "10001".to_string(),
            start: 0,
            end: 0,
        });
        block.add_component(AddressComponent {
            component_type: AddressComponentType::Country,
            text: "United States".to_string(),
            start: 0,
            end: 0,
        });
        block.calculate_confidence();
        assert_approx_eq(block.confidence, 1.0);
        assert!(block.is_valid());
    }

    #[test]
    fn test_is_valid_requires_minimum_confidence() {
        let mut block = AddressBlock::new(0, 0, "", AddressFormat::En);
        block.add_component(AddressComponent {
            component_type: AddressComponentType::State,
            text: "NY".to_string(),
            start: 0,
            end: 0,
        });
        block.add_component(AddressComponent {
            component_type: AddressComponentType::FullLine,
            text: "not a strong address".to_string(),
            start: 0,
            end: 0,
        });
        block.calculate_confidence();
        assert_approx_eq(block.confidence, 0.2);
        assert!(!block.is_valid());
    }

    #[test]
    fn test_line_offsets_handle_crlf_and_lf() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "a\r\nb\nc";
        let offsets = detector.line_offsets(text);
        assert_eq!(offsets, vec![0, 3, 5]);
        assert_eq!(detector.line_offset(&offsets, 0), 0);
        assert_eq!(detector.line_offset(&offsets, 1), 3);
        assert_eq!(detector.line_offset(&offsets, 2), 5);
    }

    #[test]
    fn test_us_address_detection_no_country_no_duplicates() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "123 Main Street\nNew York, NY 10001";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.format, AddressFormat::En);
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, text);

        let street = find_component(block, AddressComponentType::Street);
        assert_eq!(street.text, "123 Main Street");
        assert_eq!(
            street.start,
            text.find("123 Main Street").expect("street start")
        );
        assert_eq!(street.end, street.start + street.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "New York");
        assert_eq!(city.start, text.find("New York").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());

        let state = find_component(block, AddressComponentType::State);
        assert_eq!(state.text, "NY");
        assert_eq!(state.start, text.find("NY").expect("state start"));
        assert_eq!(state.end, state.start + state.text.len());

        let zip = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(zip.text, "10001");
        assert_eq!(zip.start, text.find("10001").expect("zip start"));
        assert_eq!(zip.end, zip.start + zip.text.len());

        assert!(block
            .components
            .iter()
            .all(|c| c.start >= block.start && c.end <= block.end));
    }

    #[test]
    fn test_us_address_detection_city_line_with_prefix_offsets() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "123 Main Street\nAttn: New York, NY 10001";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "New York");
        assert_eq!(city.start, text.find("New York").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());

        let state = find_component(block, AddressComponentType::State);
        assert_eq!(state.text, "NY");
        assert_eq!(state.start, text.find("NY").expect("state start"));
        assert_eq!(state.end, state.start + state.text.len());

        let zip = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(zip.text, "10001");
        assert_eq!(zip.start, text.find("10001").expect("zip start"));
        assert_eq!(zip.end, zip.start + zip.text.len());
    }

    #[test]
    fn test_us_address_detection_same_line_city_state_zip_branch() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "123 Main Street, New York, NY 10001\nNote: keep this out of the address block";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.find('\n').expect("newline"));
        assert_eq!(block.full_text, "123 Main Street, New York, NY 10001");

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "New York");
        assert_eq!(city.start, text.find("New York").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());

        let state = find_component(block, AddressComponentType::State);
        assert_eq!(state.text, "NY");
        assert_eq!(state.start, text.find("NY").expect("state start"));
        assert_eq!(state.end, state.start + state.text.len());

        let zip = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(zip.text, "10001");
        assert_eq!(zip.start, text.find("10001").expect("zip start"));
        assert_eq!(zip.end, zip.start + zip.text.len());
    }

    #[test]
    fn test_us_address_detection_single_line_street_and_city_state_zip() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "123 Main Street, New York, NY 10001";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.format, AddressFormat::En);
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, text);

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "New York");
        assert_eq!(city.start, text.find("New York").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());

        let state = find_component(block, AddressComponentType::State);
        assert_eq!(state.text, "NY");
        assert_eq!(state.start, text.find("NY").expect("state start"));
        assert_eq!(state.end, state.start + state.text.len());

        let zip = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(zip.text, "10001");
        assert_eq!(zip.start, text.find("10001").expect("zip start"));
        assert_eq!(zip.end, zip.start + zip.text.len());
    }

    #[test]
    fn test_us_address_with_country_line_prefix_offsets() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "123 Main Street\nNew York, NY 10001\nCountry: United States";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.format, AddressFormat::En);
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, text);

        let country = find_component(block, AddressComponentType::Country);
        assert_eq!(country.text, "United States");
        assert_eq!(
            country.start,
            text.find("United States").expect("country start")
        );
        assert_eq!(country.end, country.start + country.text.len());
    }

    #[test]
    fn test_us_single_line_city_state_zip_not_suppressed_by_later_block() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "Header: New York, NY 10001\n123 Main Street\nNew York, NY 10001";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 2);

        let header_match_start = text.find("New York, NY 10001").expect("header match");
        let street_match_start = text.find("123 Main Street").expect("street match");

        assert!(blocks.iter().any(|b| b.start == header_match_start));
        assert!(blocks.iter().any(|b| b.start == street_match_start));
    }

    #[test]
    fn test_us_street_not_at_line_start() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "Ship to: 123 Main Street\nNew York, NY 10001";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(
            block.start,
            text.find("123 Main Street").expect("street start")
        );
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, "123 Main Street\nNew York, NY 10001");

        let street = find_component(block, AddressComponentType::Street);
        assert_eq!(street.text, "123 Main Street");
        assert_eq!(street.start, block.start);
        assert_eq!(street.end, street.start + street.text.len());
    }

    #[test]
    fn test_us_street_on_last_line_is_safe() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "Hello\n123 Main Street";
        let blocks = detector.detect(text);

        assert!(blocks.is_empty());
    }

    #[test]
    fn test_us_address_with_country_offsets_and_full_text() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "123 Main Street\nNew York, NY 10001\nUnited States";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.format, AddressFormat::En);
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, text);

        let country = find_component(block, AddressComponentType::Country);
        assert_eq!(country.text, "United States");
        assert_eq!(
            country.start,
            text.find("United States").expect("country start")
        );
        assert_eq!(country.end, country.start + country.text.len());
    }

    #[test]
    fn test_single_line_city_state_zip() {
        let detector = AddressDetector::new(AddressFormat::En);
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
    fn test_german_address_with_country_crlf_offsets_and_full_text() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Hauptstr. 42\r\n80331 Muenchen\r\nGermany";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.format, AddressFormat::De);
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, text);

        let street = find_component(block, AddressComponentType::Street);
        assert_eq!(street.text, "Hauptstr. 42");
        assert_eq!(
            street.start,
            text.find("Hauptstr. 42").expect("street start")
        );
        assert_eq!(street.end, street.start + street.text.len());

        let plz = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(plz.text, "80331");
        assert_eq!(plz.start, text.find("80331").expect("plz start"));
        assert_eq!(plz.end, plz.start + plz.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "Muenchen");
        assert_eq!(city.start, text.find("Muenchen").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());

        let country = find_component(block, AddressComponentType::Country);
        assert_eq!(country.text, "Germany");
        assert_eq!(country.start, text.find("Germany").expect("country start"));
        assert_eq!(country.end, country.start + country.text.len());
    }

    #[test]
    fn test_german_address_without_country_uses_next_line_for_end() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Hauptstr. 42\r\n80331 Muenchen";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, text);
    }

    #[test]
    fn test_german_plz_city_on_same_line_with_following_line() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Hauptstr. 42 80331 Muenchen\r\nGermany";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];

        let plz = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(plz.text, "80331");
        assert_eq!(plz.start, text.find("80331").expect("plz start"));
        assert_eq!(plz.end, plz.start + plz.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "Muenchen");
        assert_eq!(city.start, text.find("Muenchen").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());
    }

    #[test]
    fn test_german_street_and_country_two_lines() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Hauptstr. 42\nGermany";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        let country = find_component(block, AddressComponentType::Country);
        assert_eq!(country.text, "Germany");
    }

    #[test]
    fn test_german_country_line_prefix_offsets() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Hauptstr. 42\r\n80331 Muenchen\r\nCountry: Germany";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        let country = find_component(block, AddressComponentType::Country);
        assert_eq!(country.text, "Germany");
        assert_eq!(country.start, text.find("Germany").expect("country start"));
        assert_eq!(country.end, country.start + country.text.len());
    }

    #[test]
    fn test_german_street_not_at_line_start_and_plz_line_prefix_offsets() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Header\r\nAddr: Hauptstr. 42\r\nPLZ: 80331 Muenchen";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(
            block.start,
            text.find("Hauptstr. 42").expect("street match")
        );
        assert_eq!(block.full_text, "Hauptstr. 42\r\nPLZ: 80331 Muenchen");

        let street = find_component(block, AddressComponentType::Street);
        assert_eq!(street.text, "Hauptstr. 42");
        assert_eq!(street.start, block.start);
        assert_eq!(street.end, street.start + street.text.len());

        let plz = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(plz.text, "80331");
        assert_eq!(plz.start, text.find("80331").expect("plz start"));
        assert_eq!(plz.end, plz.start + plz.text.len());
    }

    #[test]
    fn test_german_single_line_street_and_plz_city() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Hauptstr. 42 80331 Muenchen";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.full_text, text);

        let plz = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(plz.text, "80331");
        assert_eq!(plz.start, text.find("80331").expect("plz start"));
        assert_eq!(plz.end, plz.start + plz.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "Muenchen");
        assert_eq!(city.start, text.find("Muenchen").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());
    }

    #[test]
    fn test_german_street_on_last_line_is_safe() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Hello\r\nHauptstr. 42";
        let blocks = detector.detect(text);

        assert!(blocks.is_empty());
    }

    #[test]
    fn test_german_single_line_plz_city_not_suppressed_by_later_block() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "Header: 10115 Berlin\nHauptstr. 42\n80331 Muenchen";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 2);

        let header_match_start = text.find("10115 Berlin").expect("header match");
        let street_match_start = text.find("Hauptstr. 42").expect("street match");

        assert!(blocks.iter().any(|b| b.start == header_match_start));
        assert!(blocks.iter().any(|b| b.start == street_match_start));
    }

    #[test]
    fn test_french_address_with_country_crlf_offsets_and_full_text() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "42 rue de la Paix\r\n75002 Paris\r\nFrance";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.format, AddressFormat::Fr);
        assert_eq!(block.start, 0);
        assert_eq!(block.end, text.len());
        assert_eq!(block.full_text, text);

        let street = find_component(block, AddressComponentType::Street);
        assert_eq!(street.text, "42 rue de la Paix");
        assert_eq!(
            street.start,
            text.find("42 rue de la Paix").expect("street start")
        );
        assert_eq!(street.end, street.start + street.text.len());

        let cp = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(cp.text, "75002");
        assert_eq!(cp.start, text.find("75002").expect("cp start"));
        assert_eq!(cp.end, cp.start + cp.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "Paris");
        assert_eq!(city.start, text.find("Paris").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());

        let country = find_component(block, AddressComponentType::Country);
        assert_eq!(country.text, "France");
        assert_eq!(country.start, text.find("France").expect("country start"));
        assert_eq!(country.end, country.start + country.text.len());
    }

    #[test]
    fn test_french_street_not_at_line_start() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "Header\r\nAddr: 42 rue de la Paix\r\n75002 Paris";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(
            block.start,
            text.find("42 rue de la Paix").expect("street match")
        );
        assert_eq!(block.full_text, "42 rue de la Paix\r\n75002 Paris");

        let street = find_component(block, AddressComponentType::Street);
        assert_eq!(street.text, "42 rue de la Paix");
        assert_eq!(street.start, block.start);
        assert_eq!(street.end, street.start + street.text.len());
    }

    #[test]
    fn test_french_cp_city_line_with_prefix_offsets() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "42 rue de la Paix\nCP: 75002 Paris";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];

        let cp = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(cp.text, "75002");
        assert_eq!(cp.start, text.find("75002").expect("cp start"));
        assert_eq!(cp.end, cp.start + cp.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "Paris");
        assert_eq!(city.start, text.find("Paris").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());
    }

    #[test]
    fn test_french_single_line_street_and_cp_city() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "42 rue de la Paix 75002 Paris";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.full_text, text);

        let cp = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(cp.text, "75002");
        assert_eq!(cp.start, text.find("75002").expect("cp start"));
        assert_eq!(cp.end, cp.start + cp.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "Paris");
        assert_eq!(city.start, text.find("Paris").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());
    }

    #[test]
    fn test_french_street_on_last_line_is_safe() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "Hello\r\n42 rue de la Paix";
        let blocks = detector.detect(text);

        assert!(blocks.is_empty());
    }

    #[test]
    fn test_french_address_without_country_street_second_last_is_safe() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "Header\n42 rue de la Paix\n75002 Paris";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(
            block.start,
            text.find("42 rue de la Paix").expect("street match")
        );
        assert_eq!(block.full_text, "42 rue de la Paix\n75002 Paris");
    }

    #[test]
    fn test_french_cp_city_on_same_line_with_following_line() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "42 rue de la Paix 75002 Paris\r\nIgnore this line";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(block.full_text, "42 rue de la Paix 75002 Paris");

        let cp = find_component(block, AddressComponentType::PostalCode);
        assert_eq!(cp.text, "75002");
        assert_eq!(cp.start, text.find("75002").expect("cp start"));
        assert_eq!(cp.end, cp.start + cp.text.len());

        let city = find_component(block, AddressComponentType::City);
        assert_eq!(city.text, "Paris");
        assert_eq!(city.start, text.find("Paris").expect("city start"));
        assert_eq!(city.end, city.start + city.text.len());
    }

    #[test]
    fn test_french_country_line_prefix_offsets() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "42 rue de la Paix\r\n75002 Paris\r\nCountry: France";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        let country = find_component(block, AddressComponentType::Country);
        assert_eq!(country.text, "France");
        assert_eq!(country.start, text.find("France").expect("country start"));
        assert_eq!(country.end, country.start + country.text.len());
    }

    #[test]
    fn test_french_single_line_cp_city_not_suppressed_by_later_block() {
        let detector = AddressDetector::new(AddressFormat::Fr);
        let text = "Header: 75002 Paris\n42 rue de la Paix\n75002 Paris";
        let blocks = detector.detect(text);

        assert_eq!(blocks.len(), 2);

        let header_match_start = text.find("75002 Paris").expect("header match");
        let street_match_start = text.find("42 rue de la Paix").expect("street match");

        assert!(blocks.iter().any(|b| b.start == header_match_start));
        assert!(blocks.iter().any(|b| b.start == street_match_start));
    }

    #[test]
    fn test_invalid_address() {
        let detector = AddressDetector::new(AddressFormat::En);
        let text = "Hello world, this is not an address";
        let blocks = detector.detect(text);

        assert!(blocks.is_empty());
    }

    #[test]
    fn test_german_plz_format() {
        let detector = AddressDetector::new(AddressFormat::De);
        let text = "PLZ: 10115 Berlin";
        let blocks = detector.detect(text);

        assert!(!blocks.is_empty());
        let block = &blocks[0];
        assert!(block.components.iter().any(|c| {
            c.component_type == AddressComponentType::PostalCode && c.text == "10115"
        }));
    }
}
