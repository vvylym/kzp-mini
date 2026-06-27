//! Accounts for [`crate::co_sign_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::PoolError;
use crate::operations::{
    guarantor_slot, push_active_guarantee, push_pending_guarantee, reserve_savings,
    split_outstanding_50_50, validate_vault_liquidity, GuarantorSlot,
};
use crate::state::{Loan, Member, Pool};
use crate::utils::seeds::{MEMBER_SEED, VAULT_SEED};

/// Accounts required for a guarantor to co-sign and optionally trigger disbursement.
#[derive(Accounts)]
pub struct CoSignLoan<'info> {
    /// Co-signing guarantor (must be `loan.guarantor_a` or `loan.guarantor_b`).
    #[account(mut)]
    pub guarantor: Signer<'info>,

    /// Pending or activating loan account.
    #[account(mut)]
    pub loan: Box<Account<'info, Loan>>,

    /// Pool whose counters and vault are updated on disbursement.
    #[account(
        mut,
        constraint = pool.key() == loan.pool,
    )]
    pub pool: Account<'info, Pool>,

    /// Member account for guarantor A.
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_a.as_ref()],
        bump = guarantor_a_member.bump,
        constraint = guarantor_a_member.owner == loan.guarantor_a,
        constraint = guarantor_a_member.pool == loan.pool,
    )]
    pub guarantor_a_member: Box<Account<'info, Member>>,

    /// Member account for guarantor B.
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.guarantor_b.as_ref()],
        bump = guarantor_b_member.bump,
        constraint = guarantor_b_member.owner == loan.guarantor_b,
        constraint = guarantor_b_member.pool == loan.pool,
    )]
    pub guarantor_b_member: Box<Account<'info, Member>>,

    /// Pool vault debited when both guarantors have signed.
    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump,
        constraint = vault.key() == pool.vault @ PoolError::InvalidVaultAccount,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// Borrower's member account (receives `active_loan` on disbursement).
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), loan.borrower.as_ref()],
        bump = borrower_member.bump,
        constraint = borrower_member.owner == loan.borrower,
        constraint = borrower_member.pool == loan.pool,
    )]
    pub borrower_member: Box<Account<'info, Member>>,

    /// Borrower's token account receiving principal on disbursement.
    #[account(
        mut,
        constraint = borrower_token_account.mint == vault.mint,
        constraint = borrower_token_account.owner == loan.borrower,
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}

/// Records partial co-sign or disburses principal when both guarantors have signed.
pub fn handle(ctx: Context<CoSignLoan>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    require!(
        loan.status == crate::state::LoanStatus::Pending,
        PoolError::LoanNotPending
    );

    let guarantor_key = ctx.accounts.guarantor.key();
    let loan_key = loan.key();
    match guarantor_slot(guarantor_key, loan.guarantor_a, loan.guarantor_b)? {
        GuarantorSlot::A => {
            require!(!loan.guarantor_a_signed, PoolError::AlreadyCoSigned);
            loan.guarantor_a_signed = true;
            push_pending_guarantee(&mut ctx.accounts.guarantor_a_member, loan_key)?;
        }
        GuarantorSlot::B => {
            require!(!loan.guarantor_b_signed, PoolError::AlreadyCoSigned);
            loan.guarantor_b_signed = true;
            push_pending_guarantee(&mut ctx.accounts.guarantor_b_member, loan_key)?;
        }
    }

    if loan.guarantor_a_signed && loan.guarantor_b_signed {
        validate_vault_liquidity(ctx.accounts.vault.amount, loan.principal)?;

        let borrower_member = &ctx.accounts.borrower_member;
        require!(
            borrower_member.active_loan.is_none(),
            PoolError::ExistingActiveLoan
        );
        require!(
            borrower_member.pending_loan == Some(loan_key),
            PoolError::ExistingPendingLoan
        );
        require!(
            ctx.accounts.guarantor_a_member.active_loan.is_none()
                && ctx.accounts.guarantor_b_member.active_loan.is_none(),
            PoolError::GuarantorHasActiveLoan
        );
        let (share_a, share_b) = split_outstanding_50_50(loan.principal);
        reserve_savings(&mut ctx.accounts.guarantor_a_member, share_a)?;
        reserve_savings(&mut ctx.accounts.guarantor_b_member, share_b)?;

        loan.status = crate::state::LoanStatus::Active;

        let borrower_member = &mut ctx.accounts.borrower_member;
        borrower_member.pending_loan = None;
        borrower_member.active_loan = Some(loan_key);

        let guarantor_a = &mut ctx.accounts.guarantor_a_member;
        let guarantor_b = &mut ctx.accounts.guarantor_b_member;
        guarantor_a.pending_guarantees.retain(|g| g != &loan_key);
        guarantor_b.pending_guarantees.retain(|g| g != &loan_key);
        push_active_guarantee(guarantor_a, loan_key)?;
        push_active_guarantee(guarantor_b, loan_key)?;

        let pool = &mut ctx.accounts.pool;
        pool.total_outstanding_loans = pool
            .total_outstanding_loans
            .checked_add(loan.principal)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let pool_key = loan.pool;
        let vault_bump = loan.vault_bump;
        let principal = loan.principal;
        let seeds = &[VAULT_SEED, pool_key.as_ref(), &[vault_bump]];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.borrower_token_account.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                signer_seeds,
            ),
            principal,
        )?;
    }

    Ok(())
}
