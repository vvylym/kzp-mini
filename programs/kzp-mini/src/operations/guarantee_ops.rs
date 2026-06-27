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

/// Returns savings not currently reserved for active guarantees.
pub fn unlocked_savings(member: &Member) -> Result<u64, PoolError> {
    member
        .savings_balance
        .checked_sub(member.locked_savings)
        .ok_or(PoolError::GuarantorInsufficientSavings)
}

/// Reserves guarantor savings for an active loan liability.
pub fn reserve_savings(member: &mut Member, amount: u64) -> Result<(), PoolError> {
    if unlocked_savings(member)? < amount {
        return Err(PoolError::GuarantorInsufficientSavings);
    }
    member.locked_savings = member
        .locked_savings
        .checked_add(amount)
        .ok_or(PoolError::GuarantorInsufficientSavings)?;
    Ok(())
}

/// Releases a previously reserved guarantor savings amount.
pub fn release_savings(member: &mut Member, amount: u64) -> Result<(), PoolError> {
    member.locked_savings = member
        .locked_savings
        .checked_sub(amount)
        .ok_or(PoolError::GuarantorInsufficientSavings)?;
    Ok(())
}

/// Records an active guarantee on a disbursed loan, enforcing the per-member cap.
pub fn push_active_guarantee(member: &mut Member) -> Result<(), PoolError> {
    if usize::from(member.active_guarantee_count) >= Member::MAX_GUARANTEES {
        return Err(PoolError::GuarantorLimitReached);
    }
    member.active_guarantee_count = member
        .active_guarantee_count
        .checked_add(1)
        .ok_or(PoolError::GuarantorLimitReached)?;
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
pub fn clear_active_guarantee(member: &mut Member) -> Result<(), PoolError> {
    member.active_guarantee_count = member
        .active_guarantee_count
        .checked_sub(1)
        .ok_or(PoolError::ActiveGuaranteesExist)?;
    Ok(())
}

/// Moves a pending co-sign obligation into the active guarantee list.
pub fn move_pending_to_active(member: &mut Member) -> Result<(), PoolError> {
    push_active_guarantee(member)?;
    clear_pending_guarantee(member)?;
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
    fn rejects_active_when_at_capacity() {
        let mut member = empty_member();
        member.active_guarantee_count = Member::MAX_GUARANTEES as u8;
        assert_eq!(
            push_active_guarantee(&mut member).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
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
    fn moves_pending_to_active() {
        let mut member = empty_member();
        member.pending_guarantee_count = 1;

        move_pending_to_active(&mut member).unwrap();

        assert_eq!(member.pending_guarantee_count, 0);
        assert_eq!(member.active_guarantee_count, 1);
    }

    #[test]
    fn savings_reservation_uses_unlocked_balance() {
        let mut member = empty_member();
        member.savings_balance = 1_000;

        reserve_savings(&mut member, 600).unwrap();
        assert_eq!(member.locked_savings, 600);
        assert_eq!(unlocked_savings(&member).unwrap(), 400);

        assert_eq!(
            reserve_savings(&mut member, 401).unwrap_err(),
            PoolError::GuarantorInsufficientSavings
        );

        release_savings(&mut member, 600).unwrap();
        assert_eq!(member.locked_savings, 0);
    }

    #[test]
    fn release_active_guarantee_releases_savings_and_reference() {
        let mut member = empty_member();
        member.savings_balance = 1_000;
        reserve_savings(&mut member, 500).unwrap();
        push_active_guarantee(&mut member).unwrap();

        release_active_guarantee(&mut member, 500).unwrap();

        assert_eq!(member.locked_savings, 0);
        assert_eq!(member.active_guarantee_count, 0);
    }
}
