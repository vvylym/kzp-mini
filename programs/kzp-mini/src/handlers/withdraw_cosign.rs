//! Handler for [`crate::withdraw_cosign`].

use anchor_lang::prelude::*;

use crate::error::PoolError;
use crate::instructions::WithdrawCosign;
use crate::operations::{clear_guarantee_refs, guarantor_slot, GuarantorSlot};

/// Revokes the signer's partial co-sign and clears their pending guarantee.
pub fn handle(ctx: Context<WithdrawCosign>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let loan_key = loan.key();
    let guarantor_key = ctx.accounts.guarantor.key();

    match guarantor_slot(guarantor_key, loan.guarantor_a, loan.guarantor_b)? {
        GuarantorSlot::A => {
            require!(loan.guarantor_a_signed, PoolError::NotCoSigned);
            loan.guarantor_a_signed = false;
            clear_guarantee_refs(&mut ctx.accounts.guarantor_a_member, &loan_key);
        }
        GuarantorSlot::B => {
            require!(loan.guarantor_b_signed, PoolError::NotCoSigned);
            loan.guarantor_b_signed = false;
            clear_guarantee_refs(&mut ctx.accounts.guarantor_b_member, &loan_key);
        }
    }

    Ok(())
}
