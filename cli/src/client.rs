//! RPC client wrapper for building and sending transactions.

use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

use crate::config::AppConfig;

/// Thin RPC wrapper used by all CLI commands.
pub struct KzpClient {
    pub config: AppConfig,
    pub rpc: RpcClient,
}

impl KzpClient {
    pub fn new(config: AppConfig) -> Self {
        let rpc =
            RpcClient::new_with_commitment(config.rpc_url.clone(), CommitmentConfig::confirmed());
        Self { config, rpc }
    }

    pub fn payer(&self) -> &Keypair {
        &self.config.keypair
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.config.keypair.pubkey()
    }

    pub fn send_instructions(
        &self,
        instructions: &[Instruction],
        extra_signers: &[&Keypair],
        dry_run: bool,
    ) -> Result<Signature> {
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .context("failed to fetch blockhash")?;

        let mut signers: Vec<&Keypair> = vec![&self.config.keypair];
        signers.extend_from_slice(extra_signers);

        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.payer_pubkey()),
            &signers,
            blockhash,
        );

        if dry_run {
            let simulation = self
                .rpc
                .simulate_transaction(&tx)
                .context("simulation request failed")?;
            if let Some(err) = simulation.value.err {
                anyhow::bail!("simulation failed: {err:?}");
            }
            println!("Simulation OK (dry-run, not sent)");
            if let Some(logs) = simulation.value.logs {
                for line in logs {
                    println!("  {line}");
                }
            }
            return Ok(Signature::default());
        }

        let signature = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .context("transaction failed")?;
        Ok(signature)
    }
}
