//! Anchor account contexts for each instruction.
//!
//! The `Accounts` derive also emits helper types (`*Bumps`, instruction-arg views).
//! Those macro-generated items are intentionally not documented here; see each
//! `*Accounts` struct and its fields for the public account layout.

#![allow(missing_docs)] // Anchor `Accounts` / `#[instruction]` generated items.

/// Accounts for [`crate::cancel_loan`].
pub mod cancel_loan;
/// Accounts for [`crate::co_sign_loan`].
pub mod co_sign_loan;
/// Accounts for [`crate::deposit_savings`].
pub mod deposit_savings;
/// Accounts for [`crate::exit_pool`].
pub mod exit_pool;
/// Accounts for [`crate::initialize_pool`].
pub mod initialize_pool;
/// Accounts for [`crate::join_pool`].
pub mod join_pool;
/// Accounts for [`crate::repay_loan`].
pub mod repay_loan;
/// Accounts for [`crate::request_loan`].
pub mod request_loan;
/// Accounts for [`crate::settle_default`].
pub mod settle_default;
/// Accounts for [`crate::withdraw_cosign`].
pub mod withdraw_co_sign;

pub use cancel_loan::*;
pub use co_sign_loan::*;
pub use deposit_savings::*;
pub use exit_pool::*;
pub use initialize_pool::*;
pub use join_pool::*;
pub use repay_loan::*;
pub use request_loan::*;
pub use settle_default::*;
pub use withdraw_co_sign::*;
