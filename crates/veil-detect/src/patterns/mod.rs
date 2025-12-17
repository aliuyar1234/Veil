//! Built-in PII detection patterns.

mod ahv_ch;
mod credit_card;
mod drivers_license;
mod email;
mod iban;
mod national_id_de;
mod passport;
mod phone;
mod ssn;
mod tax_id_de;
mod vat_number;

pub use ahv_ch::AhvChDetector;
pub use credit_card::CreditCardDetector;
pub use drivers_license::DriversLicenseDetector;
pub use email::EmailDetector;
pub use iban::IbanDetector;
pub use national_id_de::NationalIdDeDetector;
pub use passport::PassportDetector;
pub use phone::PhoneDetector;
pub use ssn::SsnDetector;
pub use tax_id_de::TaxIdDeDetector;
pub use vat_number::VatNumberDetector;
