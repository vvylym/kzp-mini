//! Accounts for [`crate::repay_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::PoolError;
use crate::operations::release_active_guarantee;
use crate::state::{Loan, LoanStatus, Member, Pool};
use crate::utils::seeds::{MEMBER_SEED, VAULT_SEED};

/// Accounts required for a borrower to repay an active loan.
#[derive(Accounts)]
pub struct RepayLoan<'info> {
    /// Borrower wallet; must match `loan.borrower`.
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// Active loan being repaid.
    #[account(mut)]
    pub loan: Box<Account<'info, Loan>>,

    /// Pool whose `total_outstanding_loans` is decremented.
    #[account(
        mut,
        constraint = pool.key() == loan.pool,
    )]
    pub pool: Account<'info, Pool>,

    /// Pool vault receiving the repayment.
    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump,
        constraint = vault.key() == pool.vault @ PoolError::InvalidVaultAccount,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// Borrower's token account debited for the repayment.
    #[account(
        mut,
        constraint = borrower_token_account.mint == vault.mint,
        constraint = borrower_token_account.owner == borrower.key(),
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}

/// Transfers repayment to vault and updates loan, pool, and guarantee state.
pub fn handle_repay_loan<'info>(ctx: Context<'info, RepayLoan<'info>>, amount: u64) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    validate_borrower(ctx.accounts.borrower.key(), loan.borrower)?;
    validate_repayment(loan.status.clone(), loan.outstanding, amount)?;
    let (new_outstanding, fully_repaid) = apply_repayment(loan.outstanding, amount)?;
    if fully_repaid {
        validate_repayment_finalization_accounts(ctx.program_id, loan, ctx.remaining_accounts)?;
    }

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.borrower_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.borrower.to_account_info(),
            },
        ),
        amount,
    )?;

    let loan = &mut ctx.accounts.loan;
    loan.outstanding = new_outstanding;

    let pool = &mut ctx.accounts.pool;
    pool.total_outstanding_loans = pool
        .total_outstanding_loans
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if fully_repaid {
        finalize_repayment(ctx)?;
    }

    Ok(())
}

fn validate_repayment_finalization_accounts<'info>(
    program_id: &Pubkey,
    loan: &Loan,
    remaining_accounts: &'info [AccountInfo<'info>],
) -> Result<()> {
    let [
        borrower_member_info,
        guarantor_a_member_info,
        guarantor_b_member_info,
    ] = remaining_accounts
    else {
        return err!(PoolError::MissingRepaymentFinalizationAccounts);
    };

    let borrower_member = Account::<Member>::try_from(borrower_member_info)?;
    let guarantor_a_member = Account::<Member>::try_from(guarantor_a_member_info)?;
    let guarantor_b_member = Account::<Member>::try_from(guarantor_b_member_info)?;

    validate_member_account(
        program_id,
        &borrower_member,
        loan.pool,
        loan.borrower,
        PoolError::InvalidRemainingAccounts,
    )?;
    validate_member_account(
        program_id,
        &guarantor_a_member,
        loan.pool,
        loan.guarantor_a,
        PoolError::InvalidRemainingAccounts,
    )?;
    validate_member_account(
        program_id,
        &guarantor_b_member,
        loan.pool,
        loan.guarantor_b,
        PoolError::InvalidRemainingAccounts,
    )?;

    Ok(())
}

fn validate_member_account(
    program_id: &Pubkey,
    member: &Account<Member>,
    pool: Pubkey,
    owner: Pubkey,
    error: PoolError,
) -> Result<()> {
    let (expected_member, _) =
        Pubkey::find_program_address(&[MEMBER_SEED, pool.as_ref(), owner.as_ref()], program_id);
    require_keys_eq!(member.key(), expected_member, error);
    require!(
        member.owner == owner && member.pool == pool,
        PoolError::InvalidRemainingAccounts
    );
    Ok(())
}

fn finalize_repayment<'info>(ctx: Context<'info, RepayLoan<'info>>) -> Result<()> {
    let [
        borrower_member_info,
        guarantor_a_member_info,
        guarantor_b_member_info,
    ] = ctx.remaining_accounts
    else {
        return err!(PoolError::MissingRepaymentFinalizationAccounts);
    };

    let mut borrower_member = Account::<Member>::try_from(borrower_member_info)?;
    let mut guarantor_a_member = Account::<Member>::try_from(guarantor_a_member_info)?;
    let mut guarantor_b_member = Account::<Member>::try_from(guarantor_b_member_info)?;

    let loan = &mut ctx.accounts.loan;
    loan.status = crate::state::LoanStatus::Repaid;

    let loan_key = loan.key();
    if borrower_member.active_loan == Some(loan_key) {
        borrower_member.active_loan = None;
    }
    release_active_guarantee(&mut guarantor_a_member, loan.guarantor_a_locked_savings)?;
    release_active_guarantee(&mut guarantor_b_member, loan.guarantor_b_locked_savings)?;
    loan.guarantor_a_locked_savings = 0;
    loan.guarantor_b_locked_savings = 0;

    borrower_member.exit(ctx.program_id)?;
    guarantor_a_member.exit(ctx.program_id)?;
    guarantor_b_member.exit(ctx.program_id)?;
    loan.close(ctx.accounts.borrower.to_account_info())?;

    Ok(())
}

fn validate_borrower(signer: Pubkey, loan_borrower: Pubkey) -> std::result::Result<(), PoolError> {
    if signer != loan_borrower {
        return Err(PoolError::NotLoanBorrower);
    }
    Ok(())
}

fn validate_repayment(
    status: LoanStatus,
    outstanding: u64,
    amount: u64,
) -> std::result::Result<(), PoolError> {
    if status != LoanStatus::Active {
        return Err(PoolError::LoanNotActive);
    }
    if amount == 0 {
        return Err(PoolError::RepaymentAmountMustBePositive);
    }
    if amount > outstanding {
        return Err(PoolError::RepaymentExceedsOutstanding);
    }
    Ok(())
}

fn apply_repayment(
    outstanding: u64,
    amount: u64,
) -> std::result::Result<(u64, bool), ProgramError> {
    let new_outstanding = outstanding
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok((new_outstanding, new_outstanding == 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrower_check() {
        let borrower = Pubkey::new_unique();
        assert!(validate_borrower(borrower, borrower).is_ok());
        assert_eq!(
            validate_borrower(Pubkey::new_unique(), borrower).unwrap_err(),
            PoolError::NotLoanBorrower
        );
    }

    #[test]
    fn repayment_validation() {
        assert!(validate_repayment(LoanStatus::Active, 1_000, 500).is_ok());
        assert_eq!(
            validate_repayment(LoanStatus::Repaid, 0, 1).unwrap_err(),
            PoolError::LoanNotActive
        );
        assert_eq!(
            validate_repayment(LoanStatus::Active, 100, 200).unwrap_err(),
            PoolError::RepaymentExceedsOutstanding
        );
        assert_eq!(
            validate_repayment(LoanStatus::Active, 100, 0).unwrap_err(),
            PoolError::RepaymentAmountMustBePositive
        );
    }

    #[test]
    fn apply_repayment_math() {
        assert_eq!(apply_repayment(1_000, 400).unwrap(), (600, false));
        assert_eq!(apply_repayment(500, 500).unwrap(), (0, true));
    }
}
