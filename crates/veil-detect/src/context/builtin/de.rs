//! German context rules.

use crate::category::PiiCategory;
use crate::context::rule::{ContextAction, ContextRule};

/// Get built-in German context rules.
pub fn german_rules() -> Vec<ContextRule> {
    vec![
        // Honorifics that boost person name detection
        ContextRule::new(
            r"(?i)\b(herr|frau|dr|prof)\.\s+",
            ContextAction::Boost,
            0.3,
        )
        .with_language("de")
        .with_description("German honorific before name"),

        // Greetings
        ContextRule::new(
            r"(?i)\b(sehr\s+geehrte[rs]?|liebe[rs]?|hallo)\s+",
            ContextAction::Boost,
            0.25,
        )
        .with_language("de")
        .with_description("German greeting"),

        // Labels
        ContextRule::new(
            r"(?i)(kontaktperson|unterzeichnet\s+von|von|an|name|patientenname|kundenname)\s*[:]\s*",
            ContextAction::Boost,
            0.35,
        )
        .with_language("de")
        .with_description("German label before name"),

        // Email context
        ContextRule::new(
            r"(?i)(e-?mail|mail)\s*[:]\s*",
            ContextAction::Boost,
            0.4,
        )
        .with_language("de")
        .with_category(PiiCategory::Email)
        .with_description("German email label"),

        // Phone context
        ContextRule::new(
            r"(?i)(telefon|tel|mobil|handy)\s*[:]\s*",
            ContextAction::Boost,
            0.4,
        )
        .with_language("de")
        .with_category(PiiCategory::Phone)
        .with_description("German phone label"),

        // Version suppression
        ContextRule::new(
            r"(?i)\b(version|ver|v)\s+\d",
            ContextAction::Suppress,
            0.6,
        )
        .with_language("de")
        .with_category(PiiCategory::Ipv4)
        .with_description("Version, not IP"),

        // Order number suppression
        ContextRule::new(
            r"(?i)\b(bestellung|auftrag|bestell-?nr|auftrags-?nr)",
            ContextAction::Suppress,
            0.7,
        )
        .with_language("de")
        .with_category(PiiCategory::CreditCard)
        .with_description("German order number, not credit card"),

        // Address components
        ContextRule::new(
            r"(?i)(straße|strasse|str|weg|platz|allee|gasse)\s*[:.]?\s*",
            ContextAction::Boost,
            0.3,
        )
        .with_language("de")
        .with_description("German street address"),

        ContextRule::new(
            r"(?i)(adresse|anschrift|wohnort)\s*[:]\s*",
            ContextAction::Boost,
            0.35,
        )
        .with_language("de")
        .with_description("German address label"),
    ]
}
