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
    /// German Tax ID (Steueridentifikationsnummer)
    TaxIdDe,
    /// Swiss AHV Number (AHVN13)
    AhvCh,
    /// German National ID (Personalausweis)
    NationalIdDe,
    /// EU VAT Number
    VatNumber,
    /// IPv4 address
    Ipv4,
    /// IPv6 address
    Ipv6,
    /// MAC address
    MacAddress,
    /// US Social Security Number
    Ssn,
    /// Passport number
    Passport,
    /// Driver's license number
    DriversLicense,
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
            PiiCategory::TaxIdDe => write!(f, "TAX_ID_DE"),
            PiiCategory::AhvCh => write!(f, "AHV_CH"),
            PiiCategory::NationalIdDe => write!(f, "NATIONAL_ID_DE"),
            PiiCategory::VatNumber => write!(f, "VAT_NUMBER"),
            PiiCategory::Ipv4 => write!(f, "IPV4"),
            PiiCategory::Ipv6 => write!(f, "IPV6"),
            PiiCategory::MacAddress => write!(f, "MAC_ADDRESS"),
            PiiCategory::Ssn => write!(f, "SSN"),
            PiiCategory::Passport => write!(f, "PASSPORT"),
            PiiCategory::DriversLicense => write!(f, "DRIVERS_LICENSE"),
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
            PiiCategory::TaxIdDe => "tax_id_de",
            PiiCategory::AhvCh => "ahv_ch",
            PiiCategory::NationalIdDe => "national_id_de",
            PiiCategory::VatNumber => "vat_number",
            PiiCategory::Ipv4 => "ipv4",
            PiiCategory::Ipv6 => "ipv6",
            PiiCategory::MacAddress => "mac_address",
            PiiCategory::Ssn => "ssn",
            PiiCategory::Passport => "passport",
            PiiCategory::DriversLicense => "drivers_license",
            PiiCategory::Custom(name) => name,
        }
    }
}
