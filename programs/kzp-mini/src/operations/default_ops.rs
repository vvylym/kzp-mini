//! Default settlement arithmetic (50/50 guarantor split).

use std::result::Result;

use crate::error::PoolError;

/// Splits `outstanding` into two shares (first gets ceiling of half on odd amounts).
///
/// # Arguments
///
/// * `outstanding` - Remaining loan principal to allocate between guarantors.
///
/// # Returns
///
/// `(share_a, share_b)` where `share_a + share_b == outstanding`.
pub fn split_outstanding_50_50(outstanding: u64) -> (u64, u64) {
    let first = outstanding.div_ceil(2);
    let second = outstanding.saturating_sub(first);
    (first, second)
}

/// Validates both guarantors can cover their default shares from savings ledger.
///
/// # Arguments
///
/// * `outstanding` - Remaining loan principal to split 50/50.
/// * `guarantor_a_savings`, `guarantor_b_savings` - Ledger balances debited on default.
///
/// # Returns
///
/// `(share_a, share_b)` - Amount each guarantor owes after [`split_outstanding_50_50`].
pub fn validate_guarantor_default_coverage(
    outstanding: u64,
    guarantor_a_savings: u64,
    guarantor_b_savings: u64,
) -> Result<(u64, u64), PoolError> {
    let (share_a, share_b) = split_outstanding_50_50(outstanding);
    if guarantor_a_savings < share_a || guarantor_b_savings < share_b {
        return Err(PoolError::GuarantorInsufficientSavings);
    }
    Ok((share_a, share_b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_even_and_odd() {
        assert_eq!(split_outstanding_50_50(1_000), (500, 500));
        assert_eq!(split_outstanding_50_50(1_001), (501, 500));
    }

    #[test]
    fn coverage_check() {
        assert!(validate_guarantor_default_coverage(1_000, 500, 500).is_ok());
        assert_eq!(
            validate_guarantor_default_coverage(1_001, 500, 500).unwrap_err(),
            PoolError::GuarantorInsufficientSavings
        );
    }
}
