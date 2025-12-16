//! Table and CSV column header context detection.
//!
//! This module detects tabular data structures and uses column headers
//! to boost confidence for PII detection in those columns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::category::PiiCategory;

/// A detected table header that maps to a PII category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHeader {
    /// Column name from the header.
    pub name: String,
    /// Inferred PII category based on header name.
    pub inferred_category: Option<PiiCategory>,
    /// Column index (0-based).
    pub column_index: usize,
    /// Confidence boost for values in this column.
    pub boost: f32,
    /// Start position in original text.
    pub start: usize,
    /// End position in original text.
    pub end: usize,
}

/// Context information for tabular data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableContext {
    /// Detected headers.
    pub headers: Vec<TableHeader>,
    /// Delimiter detected (comma, tab, pipe, etc.).
    pub delimiter: char,
    /// Whether the table has a header row.
    pub has_header: bool,
    /// Number of columns detected.
    pub column_count: usize,
    /// Start position of the table in the text.
    pub start: usize,
    /// End position of the table in the text.
    pub end: usize,
}

impl TableContext {
    /// Create a new table context.
    pub fn new(delimiter: char, start: usize, end: usize) -> Self {
        Self {
            headers: Vec::new(),
            delimiter,
            has_header: false,
            column_count: 0,
            start,
            end,
        }
    }

    /// Add a header to this table context.
    pub fn add_header(&mut self, header: TableHeader) {
        self.column_count = self.column_count.max(header.column_index + 1);
        self.headers.push(header);
    }

    /// Get the header for a specific column index.
    pub fn get_header(&self, column_index: usize) -> Option<&TableHeader> {
        self.headers.iter().find(|h| h.column_index == column_index)
    }

    /// Get the PII category for a column index.
    pub fn get_column_category(&self, column_index: usize) -> Option<&PiiCategory> {
        self.get_header(column_index)
            .and_then(|h| h.inferred_category.as_ref())
    }

    /// Get the confidence boost for a column.
    pub fn get_column_boost(&self, column_index: usize) -> f32 {
        self.get_header(column_index)
            .map(|h| h.boost)
            .unwrap_or(0.0)
    }
}

/// Detector for table/CSV structure and column headers.
pub struct TableDetector {
    /// Mapping from header names to PII categories.
    header_mappings: HashMap<String, (PiiCategory, f32)>,
}

impl TableDetector {
    /// Create a new table detector with default header mappings.
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Email headers
        for name in &[
            "email",
            "e-mail",
            "email_address",
            "emailaddress",
            "mail",
            "correo",
        ] {
            mappings.insert(name.to_string(), (PiiCategory::Email, 0.4));
        }

        // Phone headers
        for name in &[
            "phone",
            "telephone",
            "tel",
            "mobile",
            "cell",
            "phone_number",
            "phonenumber",
            "telefon",
            "téléphone",
        ] {
            mappings.insert(name.to_string(), (PiiCategory::Phone, 0.4));
        }

        // Name headers (use Custom for PersonName)
        for name in &[
            "name",
            "full_name",
            "fullname",
            "customer_name",
            "customername",
            "first_name",
            "firstname",
            "last_name",
            "lastname",
            "nom",
            "prénom",
            "vorname",
            "nachname",
        ] {
            mappings.insert(
                name.to_string(),
                (PiiCategory::Custom("PersonName".to_string()), 0.35),
            );
        }

        // SSN headers
        for name in &[
            "ssn",
            "social_security",
            "socialsecurity",
            "ss_number",
            "ssnumber",
        ] {
            mappings.insert(
                name.to_string(),
                (PiiCategory::Custom("SSN".to_string()), 0.5),
            );
        }

        // Credit card headers
        for name in &[
            "credit_card",
            "creditcard",
            "card_number",
            "cardnumber",
            "cc_number",
            "ccnumber",
            "payment_card",
        ] {
            mappings.insert(name.to_string(), (PiiCategory::CreditCard, 0.5));
        }

        // Address headers
        for name in &[
            "address",
            "street",
            "street_address",
            "streetaddress",
            "addr",
            "adresse",
            "anschrift",
        ] {
            mappings.insert(
                name.to_string(),
                (PiiCategory::Custom("Address".to_string()), 0.35),
            );
        }

        // IP address headers
        for name in &[
            "ip",
            "ip_address",
            "ipaddress",
            "client_ip",
            "clientip",
            "source_ip",
            "sourceip",
        ] {
            mappings.insert(name.to_string(), (PiiCategory::Ipv4, 0.4));
        }

        // ID headers (generic, lower boost since "ID" is ambiguous)
        for name in &["id", "user_id", "userid", "customer_id", "customerid"] {
            mappings.insert(
                name.to_string(),
                (PiiCategory::Custom("ID".to_string()), 0.15),
            );
        }

        // Date of birth
        for name in &[
            "dob",
            "date_of_birth",
            "dateofbirth",
            "birthdate",
            "birth_date",
            "geburtsdatum",
        ] {
            mappings.insert(
                name.to_string(),
                (PiiCategory::Custom("DateOfBirth".to_string()), 0.35),
            );
        }

        Self {
            header_mappings: mappings,
        }
    }

    /// Add a custom header mapping.
    pub fn add_mapping(&mut self, header: impl Into<String>, category: PiiCategory, boost: f32) {
        self.header_mappings
            .insert(header.into().to_lowercase(), (category, boost));
    }

    /// Detect table structure in text.
    pub fn detect(&self, text: &str) -> Option<TableContext> {
        // Try different delimiters in order of preference
        for delimiter in &[',', '\t', '|', ';'] {
            if let Some(context) = self.try_detect_with_delimiter(text, *delimiter) {
                return Some(context);
            }
        }
        None
    }

    /// Try to detect a table with a specific delimiter.
    fn try_detect_with_delimiter(&self, text: &str, delimiter: char) -> Option<TableContext> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return None;
        }

        // Check first line as potential header
        let first_line = lines[0];
        let columns: Vec<&str> = first_line.split(delimiter).map(|s| s.trim()).collect();

        // Need at least 2 columns to be considered a table
        if columns.len() < 2 {
            return None;
        }

        // Check if second line (if exists) has the same number of columns
        if lines.len() > 1 {
            let second_line_cols: Vec<&str> = lines[1].split(delimiter).map(|s| s.trim()).collect();
            if second_line_cols.len() != columns.len() {
                return None;
            }
        }

        // Check if first row looks like headers (mostly text, minimal digits)
        let header_score = columns
            .iter()
            .filter(|col| {
                let is_text = col
                    .chars()
                    .all(|c| c.is_alphabetic() || c == '_' || c == ' ' || c == '-');
                let is_short = col.len() <= 30;
                is_text && is_short && !col.is_empty()
            })
            .count();

        // If most columns look like headers, treat first row as header
        let has_header = header_score >= columns.len() / 2;

        if !has_header {
            return None;
        }

        let mut context = TableContext::new(delimiter, 0, text.len());
        context.has_header = true;
        context.column_count = columns.len();

        // Build header mappings
        let mut current_pos = 0;
        for (idx, col_name) in columns.iter().enumerate() {
            let normalized = col_name.to_lowercase().replace([' ', '-'], "_");

            // Find position in original text
            let col_start = first_line[current_pos..]
                .find(col_name)
                .map(|p| current_pos + p)
                .unwrap_or(current_pos);
            let col_end = col_start + col_name.len();
            current_pos = col_end;

            let (inferred_category, boost) = self
                .header_mappings
                .get(&normalized)
                .cloned()
                .unwrap_or((PiiCategory::Custom("Unknown".to_string()), 0.0));

            // Only add header if we have a meaningful mapping
            let header = TableHeader {
                name: col_name.to_string(),
                inferred_category: if boost > 0.0 {
                    Some(inferred_category)
                } else {
                    None
                },
                column_index: idx,
                boost,
                start: col_start,
                end: col_end,
            };

            context.add_header(header);
        }

        Some(context)
    }

    /// Get the column index for a position in a CSV line.
    pub fn get_column_for_position(
        &self,
        line: &str,
        position: usize,
        delimiter: char,
    ) -> Option<usize> {
        let mut current_pos = 0;

        for (column, part) in line.split(delimiter).enumerate() {
            let part_end = current_pos + part.len();

            if position >= current_pos && position < part_end {
                return Some(column);
            }

            current_pos = part_end + 1; // +1 for delimiter
        }

        None
    }

    /// Calculate confidence boost based on column context.
    pub fn calculate_boost(&self, text: &str, position: usize, _category: &PiiCategory) -> f32 {
        if let Some(context) = self.detect(text) {
            // Find which line contains the position
            let lines: Vec<&str> = text.lines().collect();
            let mut line_start = 0;

            for (line_idx, line) in lines.iter().enumerate() {
                let line_end = line_start + line.len();

                if position >= line_start && position < line_end {
                    // Skip header row
                    if line_idx == 0 && context.has_header {
                        return 0.0;
                    }

                    // Get column for this position
                    let pos_in_line = position - line_start;
                    if let Some(col_idx) =
                        self.get_column_for_position(line, pos_in_line, context.delimiter)
                    {
                        return context.get_column_boost(col_idx);
                    }
                }

                line_start = line_end + 1; // +1 for newline
            }
        }

        0.0
    }
}

impl Default for TableDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_detection() {
        let detector = TableDetector::new();
        let csv = "Name,Email,Phone\nJohn Doe,john@example.com,555-1234";

        let context = detector.detect(csv);
        assert!(context.is_some());

        let ctx = context.unwrap();
        assert_eq!(ctx.delimiter, ',');
        assert_eq!(ctx.column_count, 3);
        assert!(ctx.has_header);
    }

    #[test]
    fn test_header_mapping() {
        let detector = TableDetector::new();
        let csv = "Name,Email,Phone\nJohn,john@test.com,123";

        let context = detector.detect(csv).unwrap();

        // Email column should map to Email category
        let email_header = context.get_header(1).unwrap();
        assert_eq!(email_header.inferred_category, Some(PiiCategory::Email));
        assert!(email_header.boost > 0.0);
    }

    #[test]
    fn test_tab_delimited() {
        let detector = TableDetector::new();
        let tsv = "Name\tEmail\tPhone\nJohn\tjohn@test.com\t123";

        let context = detector.detect(tsv);
        assert!(context.is_some());
        assert_eq!(context.unwrap().delimiter, '\t');
    }

    #[test]
    fn test_column_position() {
        let detector = TableDetector::new();
        let line = "John,john@example.com,555-1234";

        assert_eq!(detector.get_column_for_position(line, 0, ','), Some(0));
        assert_eq!(detector.get_column_for_position(line, 5, ','), Some(1));
        assert_eq!(detector.get_column_for_position(line, 25, ','), Some(2));
    }

    #[test]
    fn test_confidence_boost_calculation() {
        let detector = TableDetector::new();
        let csv = "Name,Email,Phone\nJohn,john@example.com,555-1234";

        // Position in second row, email column (starts around position 22)
        let boost = detector.calculate_boost(csv, 22, &PiiCategory::Email);
        assert!(boost > 0.0);
    }

    #[test]
    fn test_no_header_row() {
        let detector = TableDetector::new();
        // First row looks like data, not headers
        let csv = "123,456,789\n111,222,333";

        let context = detector.detect(csv);
        // Should not detect as table with headers
        assert!(context.is_none() || !context.unwrap().has_header);
    }

    #[test]
    fn test_custom_mapping() {
        let mut detector = TableDetector::new();
        detector.add_mapping(
            "patient_id",
            PiiCategory::Custom("MedicalID".to_string()),
            0.5,
        );

        let csv = "patient_id,diagnosis\n12345,cold";
        let context = detector.detect(csv).unwrap();

        let header = context.get_header(0).unwrap();
        assert_eq!(
            header.inferred_category,
            Some(PiiCategory::Custom("MedicalID".to_string()))
        );
    }

    #[test]
    fn test_get_column_category() {
        let detector = TableDetector::new();
        let csv = "Email,Name\ntest@test.com,John";

        let context = detector.detect(csv).unwrap();

        assert_eq!(context.get_column_category(0), Some(&PiiCategory::Email));
    }

    #[test]
    fn test_pipe_delimited() {
        let detector = TableDetector::new();
        let data = "Name|Email|Phone\nJohn|john@test.com|555";

        let context = detector.detect(data);
        assert!(context.is_some());
        assert_eq!(context.unwrap().delimiter, '|');
    }
}
