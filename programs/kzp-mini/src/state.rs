//! On-chain account layouts for pools, members, and loans.

use anchor_lang::prelude::*;

/// Global pool configuration and aggregate counters.
#[account]
pub struct Pool {
    /// Wallet authorized to administer the pool (e.g. call `settle_default`).
    pub admin: Pubkey,
    /// SPL mint for all pool deposits, loans, and vault transfers.
    pub token_mint: Pubkey,
    /// Token vault PDA that custodies pool SPL tokens.
    pub vault: Pubkey,
    /// One-time fee each new member must pay when joining.
    pub required_entry_fee: u64,
    /// Number of member accounts currently open in this pool.
    pub total_members: u64,
    /// Sum of all member `savings_balance` values (ledger, not vault balance).
    pub total_savings: u64,
    /// Sum of outstanding principal on active loans (updated on disburse, repay, default).
    pub total_outstanding_loans: u64,
    /// Bump seed for the pool PDA.
    pub bump: u8,
    /// Bump seed for the vault PDA (stored for CPI signing without passing `pool`).
    pub vault_bump: u8,
}

impl Pool {
    /// Account size in bytes including the 8-byte Anchor discriminator.
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1;
}

/// Per-member savings, loan, and guarantee state within a pool.
#[account]
pub struct Member {
    /// Pool this member belongs to.
    pub pool: Pubkey,
    /// Zero-based join order index assigned at `join_pool`.
    pub member_id: u64,
    /// Wallet that owns this member account.
    pub owner: Pubkey,
    /// Entry fee amount recorded at join (equals `pool.required_entry_fee`).
    pub entry_fee_paid: u64,
    /// Ledger balance of savings the member may withdraw on `exit_pool`.
    pub savings_balance: u64,
    /// Loan PDA pubkey while the member is borrower on a disbursed loan, else `None`.
    pub active_loan: Option<Pubkey>,
    /// Loan PDA pubkey while the member has a pending loan request, else `None`.
    pub pending_loan: Option<Pubkey>,
    /// Disbursed loans this member guarantees (blocks exit until cleared).
    pub active_guarantees: Vec<Pubkey>,
    /// Pending loans where this member co-signed before disbursement.
    pub pending_guarantees: Vec<Pubkey>,
    /// Bump seed for the member PDA.
    pub bump: u8,
}

impl Member {
    /// Maximum concurrent loans a member may guarantee (active + pending combined).
    pub const MAX_GUARANTEES: usize = 5;

    /// Account size in bytes including the 8-byte Anchor discriminator.
    pub const LEN: usize = 8
        + 32
        + 8
        + 32
        + 8
        + 8
        + (1 + 32)
        + (1 + 32)
        + 4
        + (Self::MAX_GUARANTEES * 32)
        + 4
        + (Self::MAX_GUARANTEES * 32)
        + 1;
}

/// A co-signed loan between a borrower and two guarantors.
#[account]
pub struct Loan {
    /// Pool that originated this loan.
    pub pool: Pubkey,
    /// Borrower wallet; must sign repayments and pending cancellation.
    pub borrower: Pubkey,
    /// Original principal requested and disbursed on activation.
    pub principal: u64,
    /// Remaining principal owed to the vault (zero when repaid or defaulted).
    pub outstanding: u64,
    /// First nominated guarantor wallet.
    pub guarantor_a: Pubkey,
    /// Second nominated guarantor wallet (must differ from `guarantor_a`).
    pub guarantor_b: Pubkey,
    /// Whether guarantor A has co-signed while the loan is pending.
    pub guarantor_a_signed: bool,
    /// Whether guarantor B has co-signed while the loan is pending.
    pub guarantor_b_signed: bool,
    /// Current lifecycle state of the loan.
    pub status: LoanStatus,
    /// Bump seed for the loan PDA.
    pub bump: u8,
    /// Vault bump copied from the pool at request time for disbursement CPI signing.
    pub vault_bump: u8,
}

impl Loan {
    /// Account size in bytes including the 8-byte Anchor discriminator.
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 32 + 32 + 1 + 1 + 1 + 1 + 1;
}

/// Lifecycle state of a loan account.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum LoanStatus {
    /// Awaiting both guarantor co-signatures; no tokens disbursed.
    Pending,
    /// Disbursed to the borrower; repayments accepted.
    Active,
    /// Fully repaid; guarantor obligations cleared.
    Repaid,
    /// Marked defaulted by admin; outstanding zeroed, guarantors charged.
    Defaulted,
}
