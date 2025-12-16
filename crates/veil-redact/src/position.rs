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
}
