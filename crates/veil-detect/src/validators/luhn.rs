//! Luhn algorithm for credit card validation.

/// Validate a credit card number using the Luhn algorithm.
///
/// # Arguments
/// * `number` - The credit card number (with or without spaces/dashes)
///
/// # Returns
/// `true` if the number passes the Luhn check, `false` otherwise
pub fn validate_luhn(number: &str) -> bool {
    // Extract only digits
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();

    // Check length (13-19 digits for most cards)
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    // Luhn algorithm
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                d
            }
        })
        .sum();

    sum.is_multiple_of(10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_visa() {
        assert!(validate_luhn("4111111111111111"));
    }

    #[test]
    fn test_valid_mastercard() {
        assert!(validate_luhn("5500000000000004"));
    }

    #[test]
    fn test_valid_amex() {
        assert!(validate_luhn("340000000000009"));
    }

    #[test]
    fn test_valid_with_spaces() {
        assert!(validate_luhn("4111 1111 1111 1111"));
    }

    #[test]
    fn test_valid_with_dashes() {
        assert!(validate_luhn("4111-1111-1111-1111"));
    }

    #[test]
    fn test_invalid_checksum() {
        assert!(!validate_luhn("4111111111111112"));
    }

    #[test]
    fn test_too_short() {
        assert!(!validate_luhn("411111111111"));
    }

    #[test]
    fn test_random_number() {
        assert!(!validate_luhn("1234567890123456"));
    }
}
