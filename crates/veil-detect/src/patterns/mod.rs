//! Built-in PII detection patterns.

mod credit_card;
mod drivers_license;
mod email;
mod iban;
mod passport;
mod phone;
mod ssn;

pub use credit_card::CreditCardDetector;
pub use drivers_license::DriversLicenseDetector;
pub use email::EmailDetector;
pub use iban::IbanDetector;
pub use passport::PassportDetector;
pub use phone::PhoneDetector;
pub use ssn::SsnDetector;
