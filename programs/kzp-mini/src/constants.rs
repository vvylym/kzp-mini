//! Protocol-wide numeric limits.

/// Maximum loan principal as a multiple of member savings (3×).
pub const MAX_LOAN_MULTIPLIER: u64 = 3;

/// Minimum UTF-8 length for a pool name.
pub const MIN_POOL_NAME_LEN: usize = 3;

/// Maximum UTF-8 length for a pool name (single PDA seed limit).
pub const MAX_POOL_NAME_LEN: usize = 32;
