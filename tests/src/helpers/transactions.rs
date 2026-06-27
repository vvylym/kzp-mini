//! Transaction submission and SOL funding on the test harness.

use anchor_lang::solana_program::system_instruction;
use kzp_mini::error::PoolError;
use solana_program_test::BanksClientError;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use super::app::TestApp;
use super::constants::anchor_error_code;
use super::errors::assert_custom_error;
impl TestApp {
    pub(super) async fn refresh_blockhash(&mut self) {
        self.context.last_blockhash = self
            .context
            .banks_client
            .get_latest_blockhash()
            .await
            .unwrap();
    }

    async fn send_transaction(
        &mut self,
        ixs: &[Instruction],
        extra_signers: &[&Keypair],
    ) -> Result<(), BanksClientError> {
        self.refresh_blockhash().await;
        // Owned copy so we can hold `&mut self` without also borrowing `context.payer`.
        let payer = Keypair::try_from(self.context.payer.to_bytes().as_ref()).unwrap();
        let mut signers: Vec<&Keypair> = vec![&payer];
        signers.extend_from_slice(extra_signers);
        let tx = Transaction::new_signed_with_payer(
            ixs,
            Some(&payer.pubkey()),
            &signers,
            self.context.last_blockhash,
        );
        self.context.banks_client.process_transaction(tx).await
    }
    /// Signs with the harness payer plus `extra_signers` and expects success.
    pub async fn process(&mut self, ixs: &[Instruction], extra_signers: &[&Keypair]) {
        self.send_transaction(ixs, extra_signers)
            .await
            .expect("transaction");
    }

    /// Expects any transaction failure (program, SPL, or runtime).
    pub async fn process_expect_err(&mut self, ixs: &[Instruction], extra_signers: &[&Keypair]) {
        assert!(self.send_transaction(ixs, extra_signers).await.is_err());
    }

    /// Expects a specific Anchor custom program error.
    pub async fn process_expect_custom_err(
        &mut self,
        ixs: &[Instruction],
        extra_signers: &[&Keypair],
        expected: PoolError,
    ) {
        let err = self.send_transaction(ixs, extra_signers).await.unwrap_err();
        assert_custom_error(err, anchor_error_code(expected));
    }

    /// Transfers native SOL from the harness payer to `target`.
    pub async fn airdrop_sol(&mut self, target: &Pubkey, amount: u64) {
        let ix = system_instruction::transfer(&self.context.payer.pubkey(), target, amount);
        self.process(&[ix], &[]).await;
    }
}
