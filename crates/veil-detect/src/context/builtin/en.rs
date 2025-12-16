//! English context rules.

use crate::category::PiiCategory;
use crate::context::rule::{ContextAction, ContextRule};

/// Get built-in English context rules.
pub fn english_rules() -> Vec<ContextRule> {
    vec![
        // Honorifics that boost person name detection
        ContextRule::new(
            r"(?i)\b(mr|mrs|ms|miss|dr|prof|professor)\.\s+",
            ContextAction::Boost,
            0.3,
        )
        .with_language("en")
        .with_description("Honorific before name"),

        ContextRule::new(
            r"(?i)\b(sir|madam|lord|lady)\s+",
            ContextAction::Boost,
            0.25,
        )
        .with_language("en")
        .with_description("Formal title before name"),

        // Labels that boost name detection
        ContextRule::new(
            r"(?i)\b(dear|hello|hi)\s+",
            ContextAction::Boost,
            0.2,
        )
        .with_language("en")
        .with_description("Greeting before name"),

        ContextRule::new(
            r"(?i)(contact\s+person|signed\s+by|from|to|cc|name|patient\s+name|customer\s+name|author)\s*[:]\s*",
            ContextAction::Boost,
            0.35,
        )
        .with_language("en")
        .with_description("Label before name"),

        // Email context
        ContextRule::new(
            r"(?i)(email|e-mail|mail)\s*[:]\s*",
            ContextAction::Boost,
            0.4,
        )
        .with_language("en")
        .with_category(PiiCategory::Email)
        .with_description("Email label"),

        // Phone context
        ContextRule::new(
            r"(?i)(phone|tel|telephone|mobile|cell)\s*[:]\s*",
            ContextAction::Boost,
            0.4,
        )
        .with_language("en")
        .with_category(PiiCategory::Phone)
        .with_description("Phone label"),

        // Suppression rules - version numbers (not IP addresses)
        ContextRule::new(
            r"(?i)\b(version|ver|v)\s+\d",
            ContextAction::Suppress,
            0.6,
        )
        .with_language("en")
        .with_category(PiiCategory::Ipv4)
        .with_description("Version number, not IP address"),

        ContextRule::new(
            r"(?i)\binternal\s+(ip|address)",
            ContextAction::Suppress,
            0.3,
        )
        .with_language("en")
        .with_category(PiiCategory::Ipv4)
        .with_description("Internal IP address reference"),

        // Suppression rules - order numbers (not credit cards)
        ContextRule::new(
            r"(?i)\border\s*(#|number|no|nr)",
            ContextAction::Suppress,
            0.7,
        )
        .with_language("en")
        .with_category(PiiCategory::CreditCard)
        .with_description("Order number, not credit card"),

        ContextRule::new(
            r"(?i)\b(sku|product\s+code|item\s+#)",
            ContextAction::Suppress,
            0.6,
        )
        .with_language("en")
        .with_category(PiiCategory::CreditCard)
        .with_description("Product code, not credit card"),

        // ISBN suppression
        ContextRule::new(
            r"(?i)\bisbn[-\s]*(10|13)?",
            ContextAction::Suppress,
            0.8,
        )
        .with_language("en")
        .with_description("ISBN, not PII"),

        // Address components
        ContextRule::new(
            r"(?i)(street|st|avenue|ave|road|rd|boulevard|blvd|lane|ln|drive|dr)\s*[:.]?\s*",
            ContextAction::Boost,
            0.3,
        )
        .with_language("en")
        .with_description("Street address component"),

        ContextRule::new(
            r"(?i)(address|addr|location|residence)\s*[:]\s*",
            ContextAction::Boost,
            0.35,
        )
        .with_language("en")
        .with_description("Address label"),
    ]
}
