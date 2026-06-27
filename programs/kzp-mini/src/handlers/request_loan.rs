//! Handler for [`crate::request_loan`].

use anchor_lang::prelude::*;

use crate::instructions::RequestLoan;
use crate::operations::validate_loan_request;
use crate::state::LoanStatus;

/// Validates loan rules and initializes a pending loan account.
pub fn handle(ctx: Context<RequestLoan>, _loan_nonce: u64, amount: u64) -> Result<()> {
    let borrower = &ctx.accounts.member_account;
    validate_loan_request(
        amount,
        ctx.accounts.borrower.key(),
        borrower.savings_balance,
        borrower.active_loan.is_some(),
        borrower.pending_loan.is_some(),
        ctx.accounts.guarantor_a.key(),
        ctx.accounts.guarantor_b.key(),
        ctx.accounts.guarantor_a_member.active_loan.is_some(),
        ctx.accounts.guarantor_b_member.active_loan.is_some(),
        ctx.accounts.guarantor_a_member.active_guarantees.len()
            + ctx.accounts.guarantor_a_member.pending_guarantees.len(),
        ctx.accounts.guarantor_b_member.active_guarantees.len()
            + ctx.accounts.guarantor_b_member.pending_guarantees.len(),
    )?;

    let loan = &mut ctx.accounts.loan;
    loan.pool = ctx.accounts.pool.key();
    loan.borrower = ctx.accounts.borrower.key();
    loan.principal = amount;
    loan.outstanding = amount;
    loan.guarantor_a = ctx.accounts.guarantor_a.key();
    loan.guarantor_b = ctx.accounts.guarantor_b.key();
    loan.guarantor_a_signed = false;
    loan.guarantor_b_signed = false;
    loan.status = LoanStatus::Pending;
    loan.bump = ctx.bumps.loan;
    loan.vault_bump = ctx.accounts.pool.vault_bump;

    ctx.accounts.member_account.pending_loan = Some(loan.key());

    Ok(())
}
