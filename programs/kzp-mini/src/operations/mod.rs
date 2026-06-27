//! Pure business logic and validation, separated from Anchor account wiring.

/// Default settlement arithmetic (50/50 guarantor split).
pub mod default_ops;
/// Guarantor obligation bookkeeping.
pub mod guarantee_ops;
/// Shared vault liquidity checks.
pub mod liquidity_ops;
/// Loan co-sign validation shared by co-sign and withdraw flows.
pub mod loan_checks;

pub use default_ops::split_outstanding_50_50;
pub use guarantee_ops::{
    clear_pending_guarantee, push_pending_guarantee, release_active_guarantee,
};

pub use liquidity_ops::validate_vault_liquidity;
pub use loan_checks::*;
