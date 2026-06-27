//! Accounts for [`crate::request_loan`].

use anchor_lang::prelude::*;

use crate::constants::MAX_LOAN_MULTIPLIER;
use crate::error::PoolError;
use crate::state::LoanStatus;
use crate::state::{Loan, Member, Pool};
use crate::utils::seeds::{LOAN_SEED, MEMBER_SEED};

/// Accounts required to open a pending loan with two nominated guarantors.
#[derive(Accounts)]
#[instruction(loan_nonce: u64)]
pub struct RequestLoan<'info> {
    /// Borrower wallet; pays loan account rent and must be a pool member.
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// Borrower's member account used for limit checks.
    #[account(
        mut,
        seeds = [MEMBER_SEED, pool.key().as_ref(), borrower.key().as_ref()],
        bump = member_account.bump,
        constraint = member_account.owner == borrower.key(),
        constraint = member_account.pool == pool.key(),
    )]
    pub member_account: Account<'info, Member>,

    /// Pool the loan belongs to.
    pub pool: Account<'info, Pool>,

    /// Loan state PDA: `["loan", pool, borrower, loan_nonce_le]`.
    #[account(
        init,
        payer = borrower,
        space = Loan::LEN,
        seeds = [
            LOAN_SEED,
            pool.key().as_ref(),
            borrower.key().as_ref(),
            &loan_nonce.to_le_bytes(),
        ],
        bump,
    )]
    pub loan: Account<'info, Loan>,

    /// First guarantor's member account (must exist in this pool).
    #[account(
        seeds = [MEMBER_SEED, pool.key().as_ref(), guarantor_a.key().as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == guarantor_a.key() @ PoolError::GuarantorNotMember,
        constraint = guarantor_a_member.pool == pool.key() @ PoolError::GuarantorNotMember,
    )]
    pub guarantor_a_member: Account<'info, Member>,

    /// First guarantor wallet pubkey (used for PDA derivation only).
    /// CHECK: validated via `guarantor_a_member`.
    pub guarantor_a: UncheckedAccount<'info>,

    /// Second guarantor's member account (must exist in this pool).
    #[account(
        seeds = [MEMBER_SEED, pool.key().as_ref(), guarantor_b.key().as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == guarantor_b.key() @ PoolError::GuarantorNotMember,
        constraint = guarantor_b_member.pool == pool.key() @ PoolError::GuarantorNotMember,
    )]
    pub guarantor_b_member: Account<'info, Member>,

    /// Second guarantor wallet pubkey (used for PDA derivation only).
    /// CHECK: validated via `guarantor_b_member`.
    pub guarantor_b: UncheckedAccount<'info>,

    /// System program for loan account creation.
    pub system_program: Program<'info, System>,
}

/// Validates loan rules and initializes a pending loan account.
pub fn handle_request_loan(
    ctx: Context<RequestLoan>,
    _loan_nonce: u64,
    amount: u64,
    loan_term_seconds: i64,
) -> Result<()> {
    require!(loan_term_seconds >= 0, PoolError::InvalidLoanTerm);
    let now = Clock::get()?.unix_timestamp;
    let due_ts = now
        .checked_add(loan_term_seconds)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let borrower = &ctx.accounts.member_account;
    validate_loan_request(LoanRequestChecks {
        amount,
        borrower: ctx.accounts.borrower.key(),
        borrower_savings: borrower.savings_balance,
        borrower_has_active_loan: borrower.active_loan.is_some(),
        borrower_has_pending_loan: borrower.pending_loan.is_some(),
        guarantor_a: ctx.accounts.guarantor_a.key(),
        guarantor_b: ctx.accounts.guarantor_b.key(),
        guarantor_a_has_active_loan: ctx.accounts.guarantor_a_member.active_loan.is_some(),
        guarantor_b_has_active_loan: ctx.accounts.guarantor_b_member.active_loan.is_some(),
        guarantor_a_guarantee_count: usize::from(
            ctx.accounts.guarantor_a_member.active_guarantee_count,
        ) + usize::from(
            ctx.accounts.guarantor_a_member.pending_guarantee_count,
        ),
        guarantor_b_guarantee_count: usize::from(
            ctx.accounts.guarantor_b_member.active_guarantee_count,
        ) + usize::from(
            ctx.accounts.guarantor_b_member.pending_guarantee_count,
        ),
    })?;

    let loan = &mut ctx.accounts.loan;
    loan.pool = ctx.accounts.pool.key();
    loan.borrower = ctx.accounts.borrower.key();
    loan.principal = amount;
    loan.outstanding = amount;
    loan.guarantor_a = ctx.accounts.guarantor_a.key();
    loan.guarantor_b = ctx.accounts.guarantor_b.key();
    loan.guarantor_a_signed = false;
    loan.guarantor_b_signed = false;
    loan.guarantor_a_locked_savings = 0;
    loan.guarantor_b_locked_savings = 0;
    loan.status = LoanStatus::Pending;
    loan.due_ts = due_ts;
    loan.bump = ctx.bumps.loan;
    loan.vault_bump = ctx.accounts.pool.vault_bump;

    ctx.accounts.member_account.pending_loan = Some(loan.key());

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct LoanRequestChecks {
    amount: u64,
    borrower: Pubkey,
    borrower_savings: u64,
    borrower_has_active_loan: bool,
    borrower_has_pending_loan: bool,
    guarantor_a: Pubkey,
    guarantor_b: Pubkey,
    guarantor_a_has_active_loan: bool,
    guarantor_b_has_active_loan: bool,
    guarantor_a_guarantee_count: usize,
    guarantor_b_guarantee_count: usize,
}

fn validate_loan_request(checks: LoanRequestChecks) -> std::result::Result<(), PoolError> {
    if checks.amount == 0 {
        return Err(PoolError::LoanAmountMustBePositive);
    }
    if checks.borrower_has_active_loan {
        return Err(PoolError::ExistingActiveLoan);
    }
    if checks.borrower_has_pending_loan {
        return Err(PoolError::ExistingPendingLoan);
    }

    let max_loan = checks
        .borrower_savings
        .checked_mul(MAX_LOAN_MULTIPLIER)
        .ok_or(PoolError::LoanExceedsMaxMultiplier)?;
    if checks.amount > max_loan {
        return Err(PoolError::LoanExceedsMaxMultiplier);
    }

    if checks.guarantor_a == checks.borrower || checks.guarantor_b == checks.borrower {
        return Err(PoolError::SelfGuaranteeNotAllowed);
    }
    if checks.guarantor_a == checks.guarantor_b {
        return Err(PoolError::DuplicateGuarantors);
    }
    if checks.guarantor_a_has_active_loan || checks.guarantor_b_has_active_loan {
        return Err(PoolError::GuarantorHasActiveLoan);
    }
    if checks.guarantor_a_guarantee_count >= Member::MAX_GUARANTEES
        || checks.guarantor_b_guarantee_count >= Member::MAX_GUARANTEES
    {
        return Err(PoolError::GuarantorLimitReached);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> (Pubkey, Pubkey, Pubkey) {
        (
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        )
    }

    fn valid_request() -> LoanRequestChecks {
        let (borrower, guarantor_a, guarantor_b) = keys();
        LoanRequestChecks {
            amount: 300,
            borrower,
            borrower_savings: 1_000,
            borrower_has_active_loan: false,
            borrower_has_pending_loan: false,
            guarantor_a,
            guarantor_b,
            guarantor_a_has_active_loan: false,
            guarantor_b_has_active_loan: false,
            guarantor_a_guarantee_count: 0,
            guarantor_b_guarantee_count: 0,
        }
    }

    #[test]
    fn rejects_zero_amount() {
        let mut checks = valid_request();
        checks.amount = 0;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::LoanAmountMustBePositive
        );
    }

    #[test]
    fn rejects_active_borrower_loan() {
        let mut checks = valid_request();
        checks.borrower_has_active_loan = true;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::ExistingActiveLoan
        );
    }

    #[test]
    fn rejects_pending_borrower_loan() {
        let mut checks = valid_request();
        checks.borrower_has_pending_loan = true;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::ExistingPendingLoan
        );
    }

    #[test]
    fn rejects_excess_multiplier() {
        let mut checks = valid_request();
        checks.amount = 4_000;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::LoanExceedsMaxMultiplier
        );
    }

    #[test]
    fn rejects_self_guarantee() {
        let mut checks = valid_request();
        checks.guarantor_a = checks.borrower;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::SelfGuaranteeNotAllowed
        );
    }

    #[test]
    fn rejects_duplicate_guarantors() {
        let mut checks = valid_request();
        checks.guarantor_b = checks.guarantor_a;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::DuplicateGuarantors
        );
    }

    #[test]
    fn rejects_guarantor_with_active_loan() {
        let mut checks = valid_request();
        checks.guarantor_a_has_active_loan = true;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::GuarantorHasActiveLoan
        );
    }

    #[test]
    fn rejects_guarantor_limit() {
        let mut checks = valid_request();
        checks.guarantor_a_guarantee_count = Member::MAX_GUARANTEES;
        assert_eq!(
            validate_loan_request(checks).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn accepts_valid_request() {
        assert!(validate_loan_request(valid_request()).is_ok());
    }
}
