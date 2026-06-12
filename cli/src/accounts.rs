//! On-chain account fetching for the CLI.

use anchor_lang::AccountDeserialize;
use anyhow::{Context, Result};
use kzp_mini::state::Loan;
use solana_sdk::pubkey::Pubkey;

use crate::client::KzpClient;

pub fn fetch_loan(client: &KzpClient, loan: &Pubkey) -> Result<Loan> {
    let account = client
        .rpc
        .get_account(loan)
        .context("failed to fetch loan account")?;
    Loan::try_deserialize(&mut account.data.as_slice()).context("failed to deserialize loan")
}
