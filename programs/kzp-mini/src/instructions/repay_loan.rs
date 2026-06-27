//! Accounts for [`crate::repay_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::PoolError;
use crate::operations::{
    apply_repayment, release_active_guarantee, split_outstanding_50_50, validate_borrower,
    validate_repayment,
};
use crate::state::{Loan, Member, Pool};
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

    /// Borrower's member account (clears `active_loan` when fully repaid).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), borrower.key().as_ref()],
        bump = borrower_member.bump,
        constraint = borrower_member.owner == borrower.key(),
        constraint = borrower_member.pool == loan.pool,
    )]
    pub borrower_member: Box<Account<'info, Member>>,

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

    /// Guarantor A member account (guarantee cleared on full repayment).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
        constraint = guarantor_a_member.pool == loan.pool,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Guarantor B member account (guarantee cleared on full repayment).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
        constraint = guarantor_b_member.pool == loan.pool,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}

/// Transfers repayment to vault and updates loan, pool, and guarantee state.
pub fn handle(ctx: Context<RepayLoan>, amount: u64) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    validate_borrower(ctx.accounts.borrower.key(), loan.borrower)?;
    validate_repayment(loan.status.clone(), loan.outstanding, amount)?;

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

    let (new_outstanding, fully_repaid) = apply_repayment(loan.outstanding, amount)?;
    loan.outstanding = new_outstanding;

    let pool = &mut ctx.accounts.pool;
    pool.total_outstanding_loans = pool
        .total_outstanding_loans
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if fully_repaid {
        loan.status = crate::state::LoanStatus::Repaid;

        let loan_key = loan.key();
        if ctx.accounts.borrower_member.active_loan == Some(loan_key) {
            ctx.accounts.borrower_member.active_loan = None;
        }
        let (share_a, share_b) = split_outstanding_50_50(loan.principal);
        release_active_guarantee(&mut ctx.accounts.guarantor_a_member, &loan_key, share_a)?;
        release_active_guarantee(&mut ctx.accounts.guarantor_b_member, &loan_key, share_b)?;
    }

    Ok(())
}
