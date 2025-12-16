//! PII category definitions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Categories of personally identifiable information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PiiCategory {
    /// Email address
    Email,
    /// International Bank Account Number
    Iban,
    /// Phone number
    Phone,
    /// Credit card number
    CreditCard,
    /// Austrian social security number (SVNr)
    SvnrAt,
    /// German social security number
    SvnrDe,
    /// Tax identification number
    TaxId,
    /// IPv4 address
    Ipv4,
    /// IPv6 address
    Ipv6,
    /// MAC address
    MacAddress,
    /// Custom category
    Custom(String),
}

impl fmt::Display for PiiCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PiiCategory::Email => write!(f, "EMAIL"),
            PiiCategory::Iban => write!(f, "IBAN"),
            PiiCategory::Phone => write!(f, "PHONE"),
            PiiCategory::CreditCard => write!(f, "CREDIT_CARD"),
            PiiCategory::SvnrAt => write!(f, "SVNR_AT"),
            PiiCategory::SvnrDe => write!(f, "SVNR_DE"),
            PiiCategory::TaxId => write!(f, "TAX_ID"),
            PiiCategory::Ipv4 => write!(f, "IPV4"),
            PiiCategory::Ipv6 => write!(f, "IPV6"),
            PiiCategory::MacAddress => write!(f, "MAC_ADDRESS"),
            PiiCategory::Custom(name) => write!(f, "{}", name.to_uppercase()),
        }
    }
}

impl PiiCategory {
    /// Get the category name as a lowercase string.
    pub fn as_str(&self) -> &str {
        match self {
            PiiCategory::Email => "email",
            PiiCategory::Iban => "iban",
            PiiCategory::Phone => "phone",
            PiiCategory::CreditCard => "credit_card",
            PiiCategory::SvnrAt => "svnr_at",
            PiiCategory::SvnrDe => "svnr_de",
            PiiCategory::TaxId => "tax_id",
            PiiCategory::Ipv4 => "ipv4",
            PiiCategory::Ipv6 => "ipv6",
            PiiCategory::MacAddress => "mac_address",
            PiiCategory::Custom(name) => name,
        }
    }
}
