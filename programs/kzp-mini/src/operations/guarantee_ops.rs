//! Guarantor obligation bookkeeping.

use std::result::Result;

use anchor_lang::prelude::Pubkey;

use crate::error::PoolError;
use crate::state::Member;

/// Records a pending co-sign obligation before disbursement.
///
/// # Arguments
///
/// * `member` - Guarantor member account to update.
/// * `loan_key` - Pending loan PDA pubkey.
pub fn push_pending_guarantee(member: &mut Member, loan_key: Pubkey) -> Result<(), PoolError> {
    if member.pending_guarantees.contains(&loan_key) {
        return Ok(());
    }
    let total = member.active_guarantees.len() + member.pending_guarantees.len();
    if total >= Member::MAX_GUARANTEES {
        return Err(PoolError::GuarantorLimitReached);
    }
    member.pending_guarantees.push(loan_key);
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
///
/// # Arguments
///
/// * `member` - Guarantor member account to update.
/// * `loan_key` - Disbursed loan PDA pubkey.
pub fn push_active_guarantee(member: &mut Member, loan_key: Pubkey) -> Result<(), PoolError> {
    if member.active_guarantees.len() >= Member::MAX_GUARANTEES {
        return Err(PoolError::GuarantorLimitReached);
    }
    member.active_guarantees.push(loan_key);
    Ok(())
}

/// Removes a loan from pending and active guarantee lists.
///
/// # Arguments
///
/// * `member` - Guarantor member account to update.
/// * `loan_key` - Loan PDA to remove from both guarantee vectors.
pub fn clear_guarantee_refs(member: &mut Member, loan_key: &Pubkey) {
    member.pending_guarantees.retain(|g| g != loan_key);
    member.active_guarantees.retain(|g| g != loan_key);
}

#[cfg(test)]
mod tests {
    use super::*;

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
            active_guarantees: Vec::new(),
            pending_guarantees: Vec::new(),
            bump: 0,
        }
    }

    #[test]
    fn rejects_active_when_at_capacity() {
        let mut member = empty_member();
        member.active_guarantees = vec![Pubkey::new_unique(); Member::MAX_GUARANTEES];
        assert_eq!(
            push_active_guarantee(&mut member, Pubkey::new_unique()).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn rejects_pending_when_at_capacity() {
        let mut member = empty_member();
        member.pending_guarantees = vec![Pubkey::new_unique(); Member::MAX_GUARANTEES];
        assert_eq!(
            push_pending_guarantee(&mut member, Pubkey::new_unique()).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn rejects_pending_when_active_plus_pending_at_capacity() {
        let mut member = empty_member();
        member.active_guarantees = vec![Pubkey::new_unique(); Member::MAX_GUARANTEES - 1];
        member.pending_guarantees = vec![Pubkey::new_unique()];
        assert_eq!(
            push_pending_guarantee(&mut member, Pubkey::new_unique()).unwrap_err(),
            PoolError::GuarantorLimitReached
        );
    }

    #[test]
    fn test_clear_guarantee_refs() {
        let loan = Pubkey::new_unique();
        let mut member = empty_member();
        member.pending_guarantees.push(loan);
        member.active_guarantees.push(loan);
        clear_guarantee_refs(&mut member, &loan);
        assert!(member.pending_guarantees.is_empty());
        assert!(member.active_guarantees.is_empty());
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
}
