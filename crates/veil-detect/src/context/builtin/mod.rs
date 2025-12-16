//! Built-in context rules for multiple languages.

mod de;
mod en;
mod fr;

pub use de::german_rules;
pub use en::english_rules;
pub use fr::french_rules;

use super::rule::ContextRule;

/// Get all built-in context rules for all languages.
pub fn all_builtin_rules() -> Vec<ContextRule> {
    let mut rules = Vec::new();
    rules.extend(english_rules());
    rules.extend(german_rules());
    rules.extend(french_rules());
    rules
}

/// Get built-in rules for a specific language.
pub fn rules_for_language(lang: &str) -> Vec<ContextRule> {
    match lang.to_lowercase().as_str() {
        "en" | "english" => english_rules(),
        "de" | "german" => german_rules(),
        "fr" | "french" => french_rules(),
        _ => Vec::new(),
    }
}
