//! Handler for [`crate::co_sign_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::error::PoolError;
use crate::instructions::CoSignLoan;
use crate::operations::{
    guarantor_slot, push_active_guarantee, push_pending_guarantee, validate_vault_liquidity,
    GuarantorSlot,
};
use crate::state::LoanStatus;
use crate::utils::seeds::VAULT_SEED;

/// Records partial co-sign or disburses principal when both guarantors have signed.
pub fn handle(ctx: Context<CoSignLoan>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    require!(
        loan.status == LoanStatus::Pending,
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

        loan.status = LoanStatus::Active;

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
