//! Handler for [`crate::cancel_loan`].

use anchor_lang::prelude::*;

use crate::instructions::CancelLoan;
use crate::operations::clear_guarantee_refs;

/// Clears guarantor pending refs before the loan account is closed.
pub fn handle(ctx: Context<CancelLoan>) -> Result<()> {
    let loan_key = ctx.accounts.loan.key();
    clear_guarantee_refs(&mut ctx.accounts.guarantor_a_member, &loan_key);
    clear_guarantee_refs(&mut ctx.accounts.guarantor_b_member, &loan_key);
    Ok(())
}
