//! Accounts for [`crate::co_sign_loan`].

use anchor_lang::prelude::*;
use anchor_spl::token::{self, ID as TOKEN_PROGRAM_ID, TokenAccount, Transfer};

use crate::error::PoolError;
use crate::operations::{
    GuarantorSlot, clear_pending_guarantee, guarantor_slot, push_pending_guarantee,
    split_outstanding_50_50, validate_vault_liquidity,
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

    /// Member account for the signing guarantor.
    #[account(
        mut,
        seeds = [MEMBER_SEED, loan.pool.as_ref(), guarantor.key().as_ref()],
        bump = guarantor_member.bump,
        constraint = guarantor_member.owner == guarantor.key(),
        constraint = guarantor_member.pool == loan.pool,
    )]
    pub guarantor_member: Box<Account<'info, Member>>,
}

/// Records partial co-sign or disburses principal when both guarantors have signed.
pub fn handle_co_sign_loan<'info>(ctx: Context<'info, CoSignLoan<'info>>) -> Result<()> {
    let (slot, should_activate) = {
        let loan = &mut ctx.accounts.loan;
        require!(
            loan.status == crate::state::LoanStatus::Pending,
            PoolError::LoanNotPending
        );

        let guarantor_key = ctx.accounts.guarantor.key();
        let slot = guarantor_slot(guarantor_key, loan.guarantor_a, loan.guarantor_b)?;
        match slot {
            GuarantorSlot::A => {
                require!(!loan.guarantor_a_signed, PoolError::AlreadyCoSigned);
                loan.guarantor_a_signed = true;
            }
            GuarantorSlot::B => {
                require!(!loan.guarantor_b_signed, PoolError::AlreadyCoSigned);
                loan.guarantor_b_signed = true;
            }
        }
        push_pending_guarantee(&mut ctx.accounts.guarantor_member)?;

        (slot, loan.guarantor_a_signed && loan.guarantor_b_signed)
    };

    if should_activate {
        activate_loan(ctx, slot)?;
    }

    Ok(())
}

fn activate_loan<'info>(ctx: Context<'info, CoSignLoan<'info>>, slot: GuarantorSlot) -> Result<()> {
    let [
        pool_info,
        other_guarantor_member_info,
        borrower_member_info,
        vault_info,
        borrower_token_account_info,
        token_program_info,
    ] = ctx.remaining_accounts
    else {
        return err!(PoolError::MissingActivationAccounts);
    };

    require_keys_eq!(
        *token_program_info.key,
        TOKEN_PROGRAM_ID,
        PoolError::InvalidRemainingAccounts
    );

    let mut pool = Account::<Pool>::try_from(pool_info)?;
    let mut other_guarantor_member = Account::<Member>::try_from(other_guarantor_member_info)?;
    let mut borrower_member = Account::<Member>::try_from(borrower_member_info)?;
    let vault = Account::<TokenAccount>::try_from(vault_info)?;
    let borrower_token_account = Account::<TokenAccount>::try_from(borrower_token_account_info)?;

    let loan = &mut ctx.accounts.loan;
    require_keys_eq!(pool.key(), loan.pool, PoolError::InvalidRemainingAccounts);

    let other_guarantor = match slot {
        GuarantorSlot::A => loan.guarantor_b,
        GuarantorSlot::B => loan.guarantor_a,
    };
    let (expected_other_member, _) = Pubkey::find_program_address(
        &[MEMBER_SEED, loan.pool.as_ref(), other_guarantor.as_ref()],
        ctx.program_id,
    );
    let (expected_borrower_member, _) = Pubkey::find_program_address(
        &[MEMBER_SEED, loan.pool.as_ref(), loan.borrower.as_ref()],
        ctx.program_id,
    );
    let (expected_vault, _) =
        Pubkey::find_program_address(&[VAULT_SEED, loan.pool.as_ref()], ctx.program_id);

    require_keys_eq!(
        other_guarantor_member.key(),
        expected_other_member,
        PoolError::InvalidRemainingAccounts
    );
    require_keys_eq!(
        borrower_member.key(),
        expected_borrower_member,
        PoolError::InvalidRemainingAccounts
    );
    require_keys_eq!(
        vault.key(),
        expected_vault,
        PoolError::InvalidRemainingAccounts
    );
    require_keys_eq!(vault.key(), pool.vault, PoolError::InvalidVaultAccount);
    require_keys_eq!(
        borrower_token_account.owner,
        loan.borrower,
        PoolError::InvalidRemainingAccounts
    );
    require_keys_eq!(
        borrower_token_account.mint,
        vault.mint,
        PoolError::InvalidRemainingAccounts
    );
    require!(
        other_guarantor_member.owner == other_guarantor
            && other_guarantor_member.pool == loan.pool
            && borrower_member.owner == loan.borrower
            && borrower_member.pool == loan.pool,
        PoolError::InvalidRemainingAccounts
    );

    validate_vault_liquidity(vault.amount, loan.principal)?;
    require!(
        borrower_member.active_loan.is_none(),
        PoolError::ExistingActiveLoan
    );
    require!(
        borrower_member.pending_loan == Some(loan.key()),
        PoolError::ExistingPendingLoan
    );
    require!(
        ctx.accounts.guarantor_member.active_loan.is_none()
            && other_guarantor_member.active_loan.is_none(),
        PoolError::GuarantorHasActiveLoan
    );

    let (share_a, share_b) = split_outstanding_50_50(loan.principal);
    match slot {
        GuarantorSlot::A => {
            reserve_savings(&mut ctx.accounts.guarantor_member, share_a)?;
            reserve_savings(&mut other_guarantor_member, share_b)?;
        }
        GuarantorSlot::B => {
            reserve_savings(&mut other_guarantor_member, share_a)?;
            reserve_savings(&mut ctx.accounts.guarantor_member, share_b)?;
        }
    }
    loan.guarantor_a_locked_savings = share_a;
    loan.guarantor_b_locked_savings = share_b;
    loan.status = crate::state::LoanStatus::Active;

    borrower_member.pending_loan = None;
    borrower_member.active_loan = Some(loan.key());
    activate_pending_guarantee(&mut ctx.accounts.guarantor_member)?;
    activate_pending_guarantee(&mut other_guarantor_member)?;

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
            *token_program_info.key,
            Transfer {
                from: vault_info.clone(),
                to: borrower_token_account_info.clone(),
                authority: vault_info.clone(),
            },
            signer_seeds,
        ),
        principal,
    )?;

    pool.exit(ctx.program_id)?;
    other_guarantor_member.exit(ctx.program_id)?;
    borrower_member.exit(ctx.program_id)?;

    Ok(())
}

fn unlocked_savings(member: &Member) -> Result<u64> {
    member
        .savings_balance
        .checked_sub(member.locked_savings)
        .ok_or(PoolError::GuarantorInsufficientSavings.into())
}

fn reserve_savings(member: &mut Member, amount: u64) -> Result<()> {
    if unlocked_savings(member)? < amount {
        return err!(PoolError::GuarantorInsufficientSavings);
    }
    member.locked_savings = member
        .locked_savings
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok(())
}

fn activate_pending_guarantee(member: &mut Member) -> Result<()> {
    if usize::from(member.active_guarantee_count) >= Member::MAX_GUARANTEES {
        return err!(PoolError::GuarantorLimitReached);
    }
    member.active_guarantee_count = member
        .active_guarantee_count
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    clear_pending_guarantee(member)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member_with_savings(savings_balance: u64) -> Member {
        Member {
            pool: Pubkey::new_unique(),
            member_id: 0,
            owner: Pubkey::new_unique(),
            entry_fee_paid: 0,
            savings_balance,
            locked_savings: 0,
            active_loan: None,
            pending_loan: None,
            active_guarantee_count: 0,
            pending_guarantee_count: 0,
            bump: 0,
        }
    }

    #[test]
    fn reserve_uses_unlocked_savings() {
        let mut member = member_with_savings(1_000);
        reserve_savings(&mut member, 600).unwrap();
        assert_eq!(member.locked_savings, 600);
        assert_eq!(unlocked_savings(&member).unwrap(), 400);
        assert_eq!(
            reserve_savings(&mut member, 401).unwrap_err(),
            PoolError::GuarantorInsufficientSavings.into()
        );
    }

    #[test]
    fn activation_moves_pending_to_active() {
        let mut member = member_with_savings(1_000);
        member.pending_guarantee_count = 1;

        activate_pending_guarantee(&mut member).unwrap();

        assert_eq!(member.pending_guarantee_count, 0);
        assert_eq!(member.active_guarantee_count, 1);
    }
}
