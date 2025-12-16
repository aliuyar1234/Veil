//! Built-in PII detection patterns.

mod credit_card;
mod email;
mod iban;
mod phone;

pub use credit_card::CreditCardDetector;
pub use email::EmailDetector;
pub use iban::IbanDetector;
pub use phone::PhoneDetector;
