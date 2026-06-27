//! Handler for [`crate::repay_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::instructions::RepayLoan;
use crate::operations::{apply_repayment, validate_borrower, validate_repayment};
use crate::state::LoanStatus;

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
        loan.status = LoanStatus::Repaid;

        let loan_key = loan.key();
        if ctx.accounts.borrower_member.active_loan == Some(loan_key) {
            ctx.accounts.borrower_member.active_loan = None;
        }
        ctx.accounts
            .guarantor_a_member
            .active_guarantees
            .retain(|g| g != &loan_key);
        ctx.accounts
            .guarantor_b_member
            .active_guarantees
            .retain(|g| g != &loan_key);
    }

    Ok(())
}
