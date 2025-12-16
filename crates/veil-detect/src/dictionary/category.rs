//! Dictionary categories and locales.

use serde::{Deserialize, Serialize};

/// Category of dictionary entries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictionaryCategory {
    /// First names (given names).
    FirstName,
    /// Last names (family names).
    LastName,
    /// Full person names (first + last combined).
    PersonName,
    /// City/town names.
    City,
    /// Street names.
    Street,
    /// Company/organization names.
    Company,
    /// Custom user-defined category.
    Custom(String),
}

impl DictionaryCategory {
    /// Returns the category as a string.
    pub fn as_str(&self) -> &str {
        match self {
            DictionaryCategory::FirstName => "first_name",
            DictionaryCategory::LastName => "last_name",
            DictionaryCategory::PersonName => "person_name",
            DictionaryCategory::City => "city",
            DictionaryCategory::Street => "street",
            DictionaryCategory::Company => "company",
            DictionaryCategory::Custom(name) => name,
        }
    }
}

impl std::fmt::Display for DictionaryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Supported locales for dictionary selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// Austria.
    At,
    /// Germany.
    De,
    /// Switzerland.
    Ch,
    /// Generic/International.
    #[default]
    Generic,
}

impl Locale {
    /// Returns the locale as a string.
    pub fn as_str(&self) -> &str {
        match self {
            Locale::At => "at",
            Locale::De => "de",
            Locale::Ch => "ch",
            Locale::Generic => "generic",
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Locale {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "at" | "austria" => Ok(Locale::At),
            "de" | "germany" => Ok(Locale::De),
            "ch" | "switzerland" => Ok(Locale::Ch),
            "generic" | "" => Ok(Locale::Generic),
            _ => Err(format!("Unknown locale: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_as_str() {
        assert_eq!(DictionaryCategory::FirstName.as_str(), "first_name");
        assert_eq!(DictionaryCategory::LastName.as_str(), "last_name");
        assert_eq!(DictionaryCategory::City.as_str(), "city");
        assert_eq!(
            DictionaryCategory::Custom("project".to_string()).as_str(),
            "project"
        );
    }

    #[test]
    fn test_locale_from_str() {
        assert_eq!("at".parse::<Locale>().unwrap(), Locale::At);
        assert_eq!("de".parse::<Locale>().unwrap(), Locale::De);
        assert_eq!("ch".parse::<Locale>().unwrap(), Locale::Ch);
        assert_eq!("generic".parse::<Locale>().unwrap(), Locale::Generic);
    }
}
