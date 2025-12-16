//! Word boundary detection for dictionary matching.

use unicode_segmentation::UnicodeSegmentation;

/// A word with its position in the source text.
#[derive(Debug, Clone)]
pub struct WordSpan {
    /// The word text.
    pub word: String,
    /// Start byte offset in source text.
    pub start: usize,
    /// End byte offset in source text (exclusive).
    pub end: usize,
}

/// Extract words with their byte positions from text.
///
/// Uses Unicode word segmentation to properly handle:
/// - German compound words
/// - Hyphenated names (Anna-Maria)
/// - Apostrophes (O'Brien)
pub fn extract_words(text: &str) -> Vec<WordSpan> {
    let mut words = Vec::new();
    let mut byte_offset = 0;

    for word in text.unicode_words() {
        // Find the word in the remaining text
        if let Some(pos) = text[byte_offset..].find(word) {
            let start = byte_offset + pos;
            let end = start + word.len();

            words.push(WordSpan {
                word: word.to_string(),
                start,
                end,
            });

            byte_offset = end;
        }
    }

    words
}

/// Check if a match at the given position has word boundaries.
///
/// Returns true if:
/// - Start is at beginning of text or preceded by non-word character
/// - End is at end of text or followed by non-word character
pub fn has_word_boundaries(text: &str, start: usize, end: usize) -> bool {
    // Check start boundary
    let start_ok = if start == 0 {
        true
    } else {
        // Get character before start
        text[..start]
            .chars()
            .last()
            .map(|c| !c.is_alphanumeric())
            .unwrap_or(true)
    };

    // Check end boundary
    let end_ok = if end >= text.len() {
        true
    } else {
        // Get character after end
        text[end..]
            .chars()
            .next()
            .map(|c| !c.is_alphanumeric())
            .unwrap_or(true)
    };

    start_ok && end_ok
}

/// Find all occurrences of a term in text that respect word boundaries.
pub fn find_with_boundaries(
    text: &str,
    term: &str,
    require_boundaries: bool,
) -> Vec<(usize, usize)> {
    let text_lower = text.to_lowercase();
    let term_lower = term.to_lowercase();
    let mut matches = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = text_lower[search_start..].find(&term_lower) {
        let start = search_start + pos;
        let end = start + term.len();

        if !require_boundaries || has_word_boundaries(text, start, end) {
            matches.push((start, end));
        }

        search_start = start + 1;
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_words() {
        let words = extract_words("Hello World");
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].word, "Hello");
        assert_eq!(words[1].word, "World");
    }

    #[test]
    fn test_extract_words_positions() {
        let text = "Hello World";
        let words = extract_words(text);

        assert_eq!(&text[words[0].start..words[0].end], "Hello");
        assert_eq!(&text[words[1].start..words[1].end], "World");
    }

    #[test]
    fn test_extract_words_german() {
        let words = extract_words("Maximilian Müller aus Wien");
        assert_eq!(words.len(), 4);
        assert_eq!(words[0].word, "Maximilian");
        assert_eq!(words[1].word, "Müller");
    }

    #[test]
    fn test_extract_words_hyphenated() {
        let words = extract_words("Anna-Maria ist hier");
        // unicode_words treats hyphenated as separate words
        assert!(words.iter().any(|w| w.word == "Anna"));
        assert!(words.iter().any(|w| w.word == "Maria"));
    }

    #[test]
    fn test_has_word_boundaries() {
        let text = "Max is here";

        // "Max" at position 0-3
        assert!(has_word_boundaries(text, 0, 3));

        // "is" at position 4-6
        assert!(has_word_boundaries(text, 4, 6));
    }

    #[test]
    fn test_no_word_boundaries() {
        let text = "Maximum value";

        // "Max" at position 0-3 is part of "Maximum"
        assert!(!has_word_boundaries(text, 0, 3));
    }

    #[test]
    fn test_find_with_boundaries() {
        let text = "Max and Maximum both contain Max";

        let matches = find_with_boundaries(text, "Max", true);
        // Should find "Max" at start and end, but not in "Maximum"
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 3));
        assert_eq!(matches[1], (29, 32));
    }

    #[test]
    fn test_find_without_boundaries() {
        let text = "Max and Maximum both contain Max";

        let matches = find_with_boundaries(text, "Max", false);
        // Should find all three occurrences
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_find_case_insensitive() {
        let text = "max and MAX and Max";

        let matches = find_with_boundaries(text, "Max", true);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_boundary_at_punctuation() {
        let text = "Hello, Maria!";

        let matches = find_with_boundaries(text, "Maria", true);
        assert_eq!(matches.len(), 1);
    }
}
