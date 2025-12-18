//! Position mapping between original and redacted text.

use serde::{Deserialize, Serialize};

/// Entry in the position map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMapEntry {
    /// Original start position.
    pub original_start: usize,
    /// Original end position.
    pub original_end: usize,
    /// Redacted start position.
    pub redacted_start: usize,
    /// Redacted end position.
    pub redacted_end: usize,
}

/// Map of positions from original to redacted text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionMap {
    entries: Vec<PositionMapEntry>,
}

impl PositionMap {
    /// Create a new empty position map.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the map.
    pub fn add(&mut self, entry: PositionMapEntry) {
        self.entries.push(entry);
    }

    /// Get all entries.
    pub fn entries(&self) -> &[PositionMapEntry] {
        &self.entries
    }

    /// Find the redacted position for an original position.
    pub fn map_position(&self, original_pos: usize) -> Option<usize> {
        let mut offset: i64 = 0;

        for entry in &self.entries {
            if original_pos < entry.original_start {
                return Some((original_pos as i64 + offset) as usize);
            } else if original_pos >= entry.original_start && original_pos < entry.original_end {
                // Position is within a redacted region
                return Some(entry.redacted_start);
            } else {
                // Accumulate the offset change
                let original_len = entry.original_end - entry.original_start;
                let redacted_len = entry.redacted_end - entry.redacted_start;
                offset += redacted_len as i64 - original_len as i64;
            }
        }

        Some((original_pos as i64 + offset) as usize)
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_position_map() {
        let map = PositionMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut map = PositionMap::new();
        map.add(PositionMapEntry {
            original_start: 0,
            original_end: 10,
            redacted_start: 0,
            redacted_end: 5,
        });
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
    }

    #[test]
    fn test_map_position_before_redaction() {
        let mut map = PositionMap::new();
        map.add(PositionMapEntry {
            original_start: 10,
            original_end: 20,
            redacted_start: 10,
            redacted_end: 15, // Redacted text is shorter
        });

        // Position before the redaction should be unchanged
        assert_eq!(map.map_position(5), Some(5));
    }

    #[test]
    fn test_map_position_within_redaction() {
        let mut map = PositionMap::new();
        map.add(PositionMapEntry {
            original_start: 10,
            original_end: 20,
            redacted_start: 10,
            redacted_end: 15,
        });

        // Position within the redacted region should map to redaction start
        assert_eq!(map.map_position(15), Some(10));
    }

    #[test]
    fn test_map_position_after_redaction() {
        let mut map = PositionMap::new();
        map.add(PositionMapEntry {
            original_start: 10,
            original_end: 20,
            redacted_start: 10,
            redacted_end: 15, // Shortened by 5 chars
        });

        // Position after redaction should be offset by the length change
        // Original pos 25 becomes 25 + (15-10) - (20-10) = 25 - 5 = 20
        assert_eq!(map.map_position(25), Some(20));
    }

    #[test]
    fn test_map_position_multiple_redactions() {
        let mut map = PositionMap::new();

        // First redaction: chars 5-10 become "[EMAIL]" (7 chars)
        map.add(PositionMapEntry {
            original_start: 5,
            original_end: 10,
            redacted_start: 5,
            redacted_end: 12, // Expanded by 2
        });

        // Second redaction: original chars 20-25 (after first redaction)
        map.add(PositionMapEntry {
            original_start: 20,
            original_end: 25,
            redacted_start: 22, // 20 + 2 (offset from first)
            redacted_end: 27,   // Same length
        });

        // Position 0 - unchanged
        assert_eq!(map.map_position(0), Some(0));

        // Position 15 - after first redaction, offset by +2
        assert_eq!(map.map_position(15), Some(17));
    }

    #[test]
    fn test_empty_map() {
        let map = PositionMap::new();
        // With no redactions, position should be unchanged
        assert_eq!(map.map_position(100), Some(100));
    }

    #[test]
    fn test_serialization() {
        let mut map = PositionMap::new();
        map.add(PositionMapEntry {
            original_start: 0,
            original_end: 10,
            redacted_start: 0,
            redacted_end: 5,
        });

        let json = serde_json::to_string(&map).unwrap();
        let deserialized: PositionMap = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized.entries()[0].original_start, 0);
    }
}
