//! Instruction handlers - one module per instruction, uniquely named entry points.

/// Handler for [`crate::cancel_loan`].
pub mod cancel_loan;
/// Handler for [`crate::co_sign_loan`].
pub mod co_sign_loan;
/// Handler for [`crate::deposit_savings`].
pub mod deposit_savings;
/// Handler for [`crate::exit_pool`].
pub mod exit_pool;
/// Handler for [`crate::initialize_pool`].
pub mod initialize_pool;
/// Handler for [`crate::join_pool`].
pub mod join_pool;
/// Handler for [`crate::repay_loan`].
pub mod repay_loan;
/// Handler for [`crate::request_loan`].
pub mod request_loan;
/// Handler for [`crate::settle_default`].
pub mod settle_default;
/// Handler for [`crate::withdraw_cosign`].
pub mod withdraw_cosign;
