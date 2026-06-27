//! Shared constants and error-code helpers for integration tests.

use kzp_mini::error::PoolError;

/// Default pool name used across tests (matches spec examples).
pub const POOL_NAME: &str = "Fabryka Lodz KZP";

/// Default entry fee configured at pool initialization.
pub const ENTRY_FEE: u64 = 50;

/// Default loan term used by non-default-focused tests (30 days).
pub const DEFAULT_LOAN_TERM_SECONDS: i64 = 30 * 24 * 60 * 60;

/// SOL balance airdropped to each test wallet for rent and fees.
pub const LAMPORTS: u64 = 10_000_000_000;

/// Maps a program `PoolError` to the Anchor custom error code (`6000 + discriminant`).
pub fn anchor_error_code(err: PoolError) -> u32 {
    6000 + err as u32
}
