//! Default settlement arithmetic (50/50 guarantor split).

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_even_and_odd() {
        assert_eq!(split_outstanding_50_50(1_000), (500, 500));
        assert_eq!(split_outstanding_50_50(1_001), (501, 500));
    }
}
