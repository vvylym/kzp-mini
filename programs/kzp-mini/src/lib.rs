//! # KZP Minimal
//!
//! On-chain workplace mutual-aid pool: members pay an entry fee, deposit savings,
//! request co-signed loans (two guarantors), repay, and exit when obligations are clear.
//!
//! ## Instructions
//!
//! | Instruction | Purpose |
//! |-------------|---------|
//! | `initialize_pool` | Admin creates pool + SPL token vault |
//! | `join_pool` | Pay one-time entry fee and become a member |
//! | `deposit_savings` | Add tokens to member savings |
//! | `request_loan` | Open a pending loan with two guarantors |
//! | `co_sign_loan` | Guarantor approves; disburses when both sign |
//! | `repay_loan` | Partial or full repayment to the vault |
//! | `cancel_loan` | Borrower cancels a pending loan |
//! | `withdraw_cosign` | Guarantor revokes partial co-sign |
//! | `settle_default` | Admin marks due default; 50/50 from reserved guarantor savings |
//! | `exit_pool` | Withdraw savings and close the member account |
//!
//! ## PDAs
//!
//! - Pool: `["pool", admin, pool_name]`
//! - Vault: `["vault", pool]`
//! - Member: `["member", pool, owner]`
//! - Loan: `["loan", pool, borrower, loan_nonce_le]`
//!
//! Architecture and design: repository `README.md`. Instruction/state reference: `docs/`.

/// Protocol-wide numeric limits and naming constraints.
pub mod constants;
/// Custom [`PoolError`] codes returned by instruction handlers.
pub mod error;
/// Anchor [`Accounts`](anchor_lang::Accounts) contexts and handlers for each instruction.
pub mod instructions;
/// Pure validation and arithmetic separated from account wiring.
pub mod operations;
/// On-chain account layouts: [`Pool`](state::Pool), [`Member`](state::Member), [`Loan`](state::Loan).
pub mod state;
/// PDA seed constants, derivation helpers, and logging macros.
pub mod utils;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("GsjUnBFvYtcxNwCrydPUjQTngGTqdx5v7APnWahnqwkx");

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "KZP Minimal",
    project_url: "https://github.com/vvylym/kzp-mini",
    contacts: "email:security@example.com",
    policy: "https://github.com/vvylym/kzp-mini/security/policy",
    preferred_languages: "en",
    auditors: "N/A"
}

/// Anchor program module - on-chain instruction dispatch entry point.
#[program]
pub mod kzp_mini {
    use super::*;

    /// Creates a new savings pool and its token vault.
    ///
    /// * `pool_name` - UTF-8 identifier (min 3 bytes); part of the pool PDA seeds.
    /// * `required_entry_fee` - SPL amount each new member pays in [`join_pool`].
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        pool_name: String,
        required_entry_fee: u64,
    ) -> Result<()> {
        handle_initialize_pool(ctx, pool_name, required_entry_fee)
    }

    /// Pays the required entry fee and initializes a member account.
    ///
    /// * `entry_fee` - Must equal `pool.required_entry_fee`.
    pub fn join_pool(ctx: Context<JoinPool>, entry_fee: u64) -> Result<()> {
        handle_join_pool(ctx, entry_fee)
    }

    /// Deposits tokens from the member ATA into the pool vault.
    ///
    /// * `amount` - SPL tokens to credit to `member.savings_balance` (must be > 0).
    pub fn deposit_savings(ctx: Context<DepositSavings>, amount: u64) -> Result<()> {
        handle_deposit_savings(ctx, amount)
    }

    /// Requests a new loan pending two guarantor co-signatures.
    ///
    /// * `loan_nonce` - Disambiguates multiple loans per borrower; part of the loan PDA seeds.
    /// * `amount` - Requested principal (max [`constants::MAX_LOAN_MULTIPLIER`] × savings).
    pub fn request_loan(
        ctx: Context<RequestLoan>,
        loan_nonce: u64,
        amount: u64,
        loan_term_seconds: i64,
    ) -> Result<()> {
        handle_request_loan(ctx, loan_nonce, amount, loan_term_seconds)
    }

    /// Co-signs a pending loan; disburses principal when both guarantors sign.
    ///
    /// The signing guarantor must be `loan.guarantor_a` or `loan.guarantor_b`.
    /// Disbursement runs automatically once both flags are set and the vault has liquidity.
    pub fn co_sign_loan<'info>(ctx: Context<'info, CoSignLoan<'info>>) -> Result<()> {
        handle_co_sign_loan(ctx)
    }

    /// Repays an active loan partially or in full.
    ///
    /// * `amount` - SPL tokens sent to the vault (must be > 0 and ≤ outstanding).
    pub fn repay_loan<'info>(ctx: Context<'info, RepayLoan<'info>>, amount: u64) -> Result<()> {
        handle_repay_loan(ctx, amount)
    }

    /// Cancels a pending loan (borrower only); clears guarantor pending obligations.
    ///
    /// The loan account is closed and rent returned to the borrower.
    pub fn cancel_loan(ctx: Context<CancelLoan>) -> Result<()> {
        handle_cancel_loan(ctx)
    }

    /// Withdraws a partial co-sign before both guarantors approve disbursement.
    ///
    /// Only the guarantor who previously co-signed may call this while the loan is pending.
    pub fn withdraw_cosign(ctx: Context<WithdrawCosign>) -> Result<()> {
        handle_withdraw_cosign(ctx)
    }

    /// Admin marks a due active loan defaulted; guarantors cover 50/50 from reserved savings ledger.
    ///
    /// Requires `pool.admin`; guarantor consent was captured when liability was reserved on activation.
    pub fn settle_default(ctx: Context<SettleDefault>) -> Result<()> {
        handle_settle_default(ctx)
    }

    /// Withdraws savings and closes the member account when obligations are clear.
    ///
    /// Fails if the member has an active loan, active guarantees, pending co-signs,
    /// or if the vault SPL balance is below `member.savings_balance`.
    pub fn exit_pool(ctx: Context<ExitPool>) -> Result<()> {
        handle_exit_pool(ctx)
    }
}
