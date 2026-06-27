//! Co-signer slot resolution shared by co-sign and withdraw flows.

use std::result::Result;

use anchor_lang::prelude::Pubkey;

use crate::error::PoolError;

/// Returns which guarantor slot the signer occupies, if any.
///
/// # Arguments
///
/// * `signer` - Wallet invoking a co-sign or withdraw instruction.
/// * `guarantor_a`, `guarantor_b` - Nominated guarantors stored on the loan account.
pub fn guarantor_slot(
    signer: Pubkey,
    guarantor_a: Pubkey,
    guarantor_b: Pubkey,
) -> Result<GuarantorSlot, PoolError> {
    if signer == guarantor_a {
        Ok(GuarantorSlot::A)
    } else if signer == guarantor_b {
        Ok(GuarantorSlot::B)
    } else {
        Err(PoolError::NotNominatedGuarantor)
    }
}

/// Guarantor position on a pending loan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuarantorSlot {
    /// First nominated guarantor (`loan.guarantor_a`).
    A,
    /// Second nominated guarantor (`loan.guarantor_b`).
    B,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guarantor_slot_resolution() {
        let a = Pubkey::new_unique();
        let b = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        assert_eq!(guarantor_slot(a, a, b).unwrap(), GuarantorSlot::A);
        assert_eq!(guarantor_slot(b, a, b).unwrap(), GuarantorSlot::B);
        assert_eq!(
            guarantor_slot(other, a, b).unwrap_err(),
            PoolError::NotNominatedGuarantor
        );
    }
}
