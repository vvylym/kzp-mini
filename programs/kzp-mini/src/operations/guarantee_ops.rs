//! Guarantor obligation bookkeeping.

use std::result::Result;

use crate::error::PoolError;
use crate::state::Member;

/// Records a pending co-sign obligation before disbursement.
pub fn push_pending_guarantee(member: &mut Member) -> Result<(), PoolError> {
    let total =
        usize::from(member.active_guarantee_count) + usize::from(member.pending_guarantee_count);
    if total >= Member::MAX_GUARANTEES {
        return Err(PoolError::GuarantorLimitReached);
    }
    member.pending_guarantee_count = member
        .pending_guarantee_count
        .checked_add(1)
        .ok_or(PoolError::GuarantorLimitReached)?;
    Ok(())
}

/// Releases a previously reserved guarantor savings amount.
fn release_savings(member: &mut Member, amount: u64) -> Result<(), PoolError> {
    member.locked_savings = member
        .locked_savings
        .checked_sub(amount)
        .ok_or(PoolError::GuarantorInsufficientSavings)?;
    Ok(())
}

/// Removes one pending co-sign obligation.
pub fn clear_pending_guarantee(member: &mut Member) -> Result<(), PoolError> {
    member.pending_guarantee_count = member
        .pending_guarantee_count
        .checked_sub(1)
        .ok_or(PoolError::PendingGuaranteesExist)?;
    Ok(())
}

/// Removes one active guarantee obligation.
fn clear_active_guarantee(member: &mut Member) -> Result<(), PoolError> {
    member.active_guarantee_count = member
        .active_guarantee_count
        .checked_sub(1)
        .ok_or(PoolError::ActiveGuaranteesExist)?;
    Ok(())
}

/// Releases reserved savings and clears an active guarantee reference.
pub fn release_active_guarantee(member: &mut Member, locked_amount: u64) -> Result<(), PoolError> {
    release_savings(member, locked_amount)?;
    clear_active_guarantee(member)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    fn empty_member() -> Member {
        Member {
            pool: Pubkey::new_unique(),
            member_id: 0,
            owner: Pubkey::new_unique(),
            entry_fee_paid: 0,
            savings_balance: 0,
            locked_savings: 0,
            active_loan: None,
            pending_loan: None,
            active_guarantee_count: 0,
            pending_guarantee_count: 0,
            bump: 0,
        }
    }

    #[test]
    fn rejects_pending_when_at_capacity() {
        let mut member = empty_member();
        member.pending_guarantee_count = Member::MAX_GUARANTEES as u8;
        assert_eq!(
            push_pending_guarantee(&mut member).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn rejects_pending_when_active_plus_pending_at_capacity() {
        let mut member = empty_member();
        member.active_guarantee_count = (Member::MAX_GUARANTEES - 1) as u8;
        member.pending_guarantee_count = 1;
        assert_eq!(
            push_pending_guarantee(&mut member).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn clears_pending_and_active_counts_separately() {
        let mut member = empty_member();
        member.pending_guarantee_count = 1;
        member.active_guarantee_count = 1;

        clear_pending_guarantee(&mut member).unwrap();
        assert_eq!(member.pending_guarantee_count, 0);
        assert_eq!(member.active_guarantee_count, 1);

        clear_active_guarantee(&mut member).unwrap();
        assert_eq!(member.active_guarantee_count, 0);
    }

    #[test]
    fn release_active_guarantee_releases_savings_and_reference() {
        let mut member = empty_member();
        member.savings_balance = 1_000;
        member.locked_savings = 500;
        member.active_guarantee_count = 1;

        release_active_guarantee(&mut member, 500).unwrap();

        assert_eq!(member.locked_savings, 0);
        assert_eq!(member.active_guarantee_count, 0);
    }
}
