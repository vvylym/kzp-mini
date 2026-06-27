//! Accounting invariant assertions for integration tests.

use kzp_mini::utils::pda::vault_pda;
use solana_sdk::pubkey::Pubkey;

use super::app::TestApp;

impl TestApp {
    /// Asserts the vault can cover the liquid savings ledger after outstanding loans.
    ///
    /// Entry fees are intentionally excluded from `pool.total_savings`, so the vault may
    /// exceed this minimum by the accumulated entry-fee buffer.
    pub async fn assert_vault_covers_liquid_savings(&mut self, pool: &Pubkey) {
        let pool_state = self.fetch_pool(pool).await;
        let (vault, _) = vault_pda(pool);
        let vault_amount = self.token_balance(&vault).await;
        let liquid_savings = pool_state
            .total_savings
            .saturating_sub(pool_state.total_outstanding_loans);

        assert!(
            vault_amount >= liquid_savings,
            "vault {} below liquid savings {} for pool {}",
            vault_amount,
            liquid_savings,
            pool
        );
    }
}
