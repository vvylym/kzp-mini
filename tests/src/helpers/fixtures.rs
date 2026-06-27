//! Multi-step scenario helpers built on [`TestApp`].

use kzp_mini::utils::pda::{loan_pda, pool_pda};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use super::app::TestApp;
use super::constants::{ENTRY_FEE, LAMPORTS, POOL_NAME};

/// Funds a new wallet with SOL (typical pool admin).
pub async fn funded_admin(app: &mut TestApp) -> Keypair {
    let admin = Keypair::new();
    app.airdrop_sol(&admin.pubkey(), LAMPORTS).await;
    admin
}

/// Returns a funded admin and an initialized pool PDA.
pub async fn initialized_pool(app: &mut TestApp) -> (Keypair, Pubkey) {
    let admin = funded_admin(app).await;
    let pool = app.setup_pool(&admin).await;
    (admin, pool)
}

/// Joins a new member who has already deposited `savings` into the pool.
pub async fn member_with_savings(
    app: &mut TestApp,
    pool: Pubkey,
    savings: u64,
) -> (Keypair, Pubkey) {
    let (user, ata) = app.create_funded_user(savings + ENTRY_FEE).await;
    app.join_with_savings(&user, pool, ata, savings).await;
    (user, ata)
}

impl TestApp {
    /// Initializes a pool with [`POOL_NAME`] and [`ENTRY_FEE`].
    pub async fn setup_pool(&mut self, admin: &Keypair) -> Pubkey {
        let (pool, _) = pool_pda(&admin.pubkey(), POOL_NAME);
        let ix = self.initialize_pool(admin, POOL_NAME, ENTRY_FEE);
        self.process(&[ix], &[admin]).await;
        pool
    }

    /// Joins `member` paying the standard entry fee.
    pub async fn join_member(&mut self, member: &Keypair, pool: Pubkey, member_ata: Pubkey) {
        let ix = self.join_pool(member, pool, member_ata, ENTRY_FEE, None);
        self.process(&[ix], &[member]).await;
    }

    /// Deposits savings for an existing member.
    pub async fn deposit(
        &mut self,
        member: &Keypair,
        pool: Pubkey,
        member_ata: Pubkey,
        amount: u64,
    ) {
        let ix = self.deposit_savings(member, pool, member_ata, amount);
        self.process(&[ix], &[member]).await;
    }

    /// Entry fee join followed by an optional savings deposit.
    pub async fn join_with_savings(
        &mut self,
        member: &Keypair,
        pool: Pubkey,
        member_ata: Pubkey,
        savings: u64,
    ) {
        self.join_member(member, pool, member_ata).await;
        if savings > 0 {
            self.deposit(member, pool, member_ata, savings).await;
        }
    }

    /// Requests a loan and co-signs until disbursed; returns the loan PDA.
    #[allow(clippy::too_many_arguments)]
    pub async fn activate_loan(
        &mut self,
        borrower: &Keypair,
        pool: Pubkey,
        loan_nonce: u64,
        amount: u64,
        guarantor_a: &Keypair,
        guarantor_b: &Keypair,
        borrower_ata: Pubkey,
    ) -> Pubkey {
        self.activate_loan_with_term(
            borrower,
            pool,
            loan_nonce,
            amount,
            guarantor_a,
            guarantor_b,
            borrower_ata,
            super::constants::DEFAULT_LOAN_TERM_SECONDS,
        )
        .await
    }

    /// Requests a loan with a custom term and co-signs until disbursed; returns the loan PDA.
    #[allow(clippy::too_many_arguments)]
    pub async fn activate_loan_with_term(
        &mut self,
        borrower: &Keypair,
        pool: Pubkey,
        loan_nonce: u64,
        amount: u64,
        guarantor_a: &Keypair,
        guarantor_b: &Keypair,
        borrower_ata: Pubkey,
        loan_term_seconds: i64,
    ) -> Pubkey {
        let ix = self.request_loan_with_term(
            borrower,
            pool,
            loan_nonce,
            amount,
            guarantor_a.pubkey(),
            guarantor_b.pubkey(),
            loan_term_seconds,
        );
        self.process(&[ix], &[borrower]).await;
        let (loan, _) = loan_pda(&pool, &borrower.pubkey(), loan_nonce);
        let ix_a = self.co_sign_loan(guarantor_a, loan, borrower_ata).await;
        self.process(&[ix_a], &[guarantor_a]).await;
        let ix_b = self.co_sign_loan(guarantor_b, loan, borrower_ata).await;
        self.process(&[ix_b], &[guarantor_b]).await;
        loan
    }
}
