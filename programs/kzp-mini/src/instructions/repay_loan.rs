//! Accounts for [`crate::repay_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::error::PoolError;
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
