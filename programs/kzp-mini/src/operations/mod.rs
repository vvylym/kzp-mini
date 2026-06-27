//! Pure business logic and validation, separated from Anchor account wiring.

/// Default settlement arithmetic (50/50 guarantor split).
pub mod default_ops;
/// Member exit eligibility checks.
pub mod exit_checks;
/// Guarantor obligation bookkeeping.
pub mod guarantee_ops;
/// Loan request and co-sign validation logic.
pub mod loan_checks;
/// Pool-level validation helpers.
pub mod pool_ops;
/// Repayment arithmetic and borrower validation.
pub mod repay_ops;

pub use default_ops::{split_outstanding_50_50, validate_guarantor_default_coverage};
pub use guarantee_ops::{
    clear_guarantee_refs, push_active_guarantee, push_pending_guarantee, release_savings,
    reserve_savings, unlocked_savings,
};

pub use exit_checks::*;
pub use loan_checks::*;
pub use pool_ops::*;
pub use repay_ops::*;
