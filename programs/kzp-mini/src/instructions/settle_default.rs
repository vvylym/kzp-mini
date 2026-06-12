//! Accounts for [`crate::settle_default`].

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::error::PoolError;
use crate::state::{Loan, Member, Pool};
use crate::utils::seeds::{MEMBER_SEED, VAULT_SEED};

/// Accounts required for admin default settlement with 50/50 guarantor coverage.
#[derive(Accounts)]
pub struct SettleDefault<'info> {
    /// Pool admin wallet (must match `pool.admin`; initiates settlement).
    #[account(
        constraint = admin.key() == pool.admin @ PoolError::NotPoolAdmin,
    )]
    pub admin: Signer<'info>,

    /// Pool whose counters are updated.
    #[account(
        mut,
        constraint = pool.key() == loan.pool,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// Active loan being marked defaulted.
    #[account(
        mut,
        constraint = loan.status == crate::state::LoanStatus::Active @ PoolError::LoanNotActive,
    )]
    pub loan: Box<Account<'info, Loan>>,

    /// Borrower's member account (`active_loan` cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.borrower.as_ref()],
        bump = borrower_member.bump,
        constraint = borrower_member.owner == loan.borrower,
    )]
    pub borrower_member: Box<Account<'info, Member>>,

    /// Guarantor A wallet; must sign SPL transfer to vault.
    #[account(
        constraint = guarantor_a.key() == loan.guarantor_a @ PoolError::NotNominatedGuarantor,
    )]
    pub guarantor_a: Signer<'info>,

    /// Guarantor B wallet; must sign SPL transfer to vault.
    #[account(
        constraint = guarantor_b.key() == loan.guarantor_b @ PoolError::NotNominatedGuarantor,
    )]
    pub guarantor_b: Signer<'info>,

    /// Guarantor A member account (savings debited and guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Guarantor B member account (savings debited and guarantee cleared).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,

    /// Pool vault receiving default share transfers.
    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump,
        constraint = vault.key() == pool.vault @ PoolError::InvalidVaultAccount,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// Guarantor A token account debited for their default share.
    #[account(
        mut,
        constraint = guarantor_a_token.mint == vault.mint,
        constraint = guarantor_a_token.owner == loan.guarantor_a,
    )]
    pub guarantor_a_token: Box<Account<'info, TokenAccount>>,

    /// Guarantor B token account debited for their default share.
    #[account(
        mut,
        constraint = guarantor_b_token.mint == vault.mint,
        constraint = guarantor_b_token.owner == loan.guarantor_b,
    )]
    pub guarantor_b_token: Box<Account<'info, TokenAccount>>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}
