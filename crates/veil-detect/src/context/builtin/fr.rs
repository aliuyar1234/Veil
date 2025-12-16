//! French context rules.

use crate::category::PiiCategory;
use crate::context::rule::{ContextAction, ContextRule};

/// Get built-in French context rules.
pub fn french_rules() -> Vec<ContextRule> {
    vec![
        // Honorifics
        ContextRule::new(
            r"(?i)\b(monsieur|madame|mademoiselle|m\.|mme|mlle|dr|prof)\s+",
            ContextAction::Boost,
            0.3,
        )
        .with_language("fr")
        .with_description("French honorific before name"),

        // Greetings
        ContextRule::new(
            r"(?i)\b(cher|chère|bonjour|salut)\s+",
            ContextAction::Boost,
            0.25,
        )
        .with_language("fr")
        .with_description("French greeting"),

        // Labels
        ContextRule::new(
            r"(?i)(personne\s+de\s+contact|signé\s+par|de|à|nom|nom\s+du\s+patient|nom\s+du\s+client)\s*[:]\s*",
            ContextAction::Boost,
            0.35,
        )
        .with_language("fr")
        .with_description("French label before name"),

        // Email context
        ContextRule::new(
            r"(?i)(e-?mail|courriel|mail)\s*[:]\s*",
            ContextAction::Boost,
            0.4,
        )
        .with_language("fr")
        .with_category(PiiCategory::Email)
        .with_description("French email label"),

        // Phone context
        ContextRule::new(
            r"(?i)(téléphone|tél|portable|mobile)\s*[:]\s*",
            ContextAction::Boost,
            0.4,
        )
        .with_language("fr")
        .with_category(PiiCategory::Phone)
        .with_description("French phone label"),

        // Version suppression
        ContextRule::new(
            r"(?i)\b(version|ver|v)\s+\d",
            ContextAction::Suppress,
            0.6,
        )
        .with_language("fr")
        .with_category(PiiCategory::Ipv4)
        .with_description("Version, not IP"),

        // Order number suppression
        ContextRule::new(
            r"(?i)\b(commande|numéro\s+de\s+commande|n°\s+commande)",
            ContextAction::Suppress,
            0.7,
        )
        .with_language("fr")
        .with_category(PiiCategory::CreditCard)
        .with_description("French order number, not credit card"),

        // Address components
        ContextRule::new(
            r"(?i)(rue|avenue|av|boulevard|bd|place|pl|chemin|allée)\s*[:.]?\s*",
            ContextAction::Boost,
            0.3,
        )
        .with_language("fr")
        .with_description("French street address"),

        ContextRule::new(
            r"(?i)(adresse|domicile|résidence)\s*[:]\s*",
            ContextAction::Boost,
            0.35,
        )
        .with_language("fr")
        .with_description("French address label"),
    ]
}
