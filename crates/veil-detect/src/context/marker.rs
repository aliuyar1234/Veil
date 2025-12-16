//! Context marker detection and types.

use serde::{Deserialize, Serialize};

use super::rule::ContextAction;

/// Type of context marker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerType {
    /// Honorific (Mr., Dr., Herr, etc.)
    Honorific,
    /// Label (Name:, Email:, Phone:, etc.)
    Label,
    /// Suppression marker (version, order #, ISBN, etc.)
    Suppression,
    /// Table header
    TableHeader,
    /// Address component
    AddressComponent,
    /// Custom marker type
    Custom(String),
}

/// A detected context marker in text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMarker {
    /// Type of marker.
    pub marker_type: MarkerType,

    /// The matched text.
    pub text: String,

    /// Start position in the text (byte offset).
    pub start: usize,

    /// End position in the text (byte offset).
    pub end: usize,

    /// Action suggested by this marker.
    pub action: ContextAction,

    /// Weight of the action.
    pub weight: f32,

    /// Language of the marker (if language-specific).
    pub language: Option<String>,
}

impl ContextMarker {
    /// Create a new context marker.
    pub fn new(
        marker_type: MarkerType,
        text: impl Into<String>,
        start: usize,
        end: usize,
        action: ContextAction,
        weight: f32,
    ) -> Self {
        Self {
            marker_type,
            text: text.into(),
            start,
            end,
            action,
            weight,
            language: None,
        }
    }

    /// Set the language for this marker.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Check if this marker's range overlaps with the given range.
    pub fn overlaps(&self, start: usize, end: usize) -> bool {
        !(self.end <= start || self.start >= end)
    }

    /// Check if this marker is near the given position (within distance).
    pub fn is_near(&self, position: usize, distance: usize) -> bool {
        let marker_center = (self.start + self.end) / 2;
        position.abs_diff(marker_center) <= distance
    }

    /// Check if the given range is within this marker's context window.
    pub fn in_context_window(&self, start: usize, end: usize, window_size: usize) -> bool {
        let match_start = start;
        let match_end = end;
        let marker_start = self.start.saturating_sub(window_size);
        let marker_end = self.end + window_size;

        // Check if match is within the context window
        match_start >= marker_start && match_end <= marker_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_creation() {
        let marker = ContextMarker::new(
            MarkerType::Honorific,
            "Mr.",
            10,
            13,
            ContextAction::Boost,
            0.3,
        )
        .with_language("en");

        assert_eq!(marker.text, "Mr.");
        assert_eq!(marker.start, 10);
        assert_eq!(marker.end, 13);
        assert_eq!(marker.language, Some("en".to_string()));
    }

    #[test]
    fn test_marker_overlap() {
        let marker = ContextMarker::new(
            MarkerType::Label,
            "Email:",
            10,
            16,
            ContextAction::Boost,
            0.4,
        );

        assert!(marker.overlaps(10, 20));
        assert!(marker.overlaps(5, 15));
        assert!(!marker.overlaps(16, 20));
        assert!(!marker.overlaps(0, 10));
    }

    #[test]
    fn test_marker_proximity() {
        let marker = ContextMarker::new(
            MarkerType::Honorific,
            "Dr.",
            10,
            13,
            ContextAction::Boost,
            0.3,
        );

        // Marker center is at position 11-12
        assert!(marker.is_near(11, 5));
        assert!(marker.is_near(15, 5));
        assert!(!marker.is_near(20, 5));
    }

    #[test]
    fn test_context_window() {
        let marker = ContextMarker::new(
            MarkerType::Label,
            "Name:",
            10,
            15,
            ContextAction::Boost,
            0.4,
        );

        // Window size of 50 chars
        // Marker at 10-15, so window is approximately -40 to +65
        assert!(marker.in_context_window(20, 30, 50));
        assert!(marker.in_context_window(60, 65, 50));
        assert!(!marker.in_context_window(100, 110, 50));
    }
}
