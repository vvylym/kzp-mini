//! On-chain account fetching for the CLI.

use anchor_lang::AccountDeserialize;
use anyhow::{Context, Result, bail};
use kzp_mini::ID as PROGRAM_ID;
use kzp_mini::state::{Loan, Pool};
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::client::KzpClient;

fn program_id() -> Pubkey {
    Pubkey::new_from_array(PROGRAM_ID.to_bytes())
}

pub(crate) fn ensure_program_owner(address: &Pubkey, account: &Account, label: &str) -> Result<()> {
    let expected = program_id();
    if account.owner != expected {
        bail!(
            "{label} account {address} is owned by {}, expected {}",
            account.owner,
            expected
        );
    }
    Ok(())
}

fn fetch_program_account(client: &KzpClient, address: &Pubkey, label: &str) -> Result<Account> {
    let account = client
        .rpc
        .get_account(address)
        .with_context(|| format!("failed to fetch {label} account"))?;
    ensure_program_owner(address, &account, label)?;
    Ok(account)
}

fn deserialize_program_account<T: AccountDeserialize>(
    client: &KzpClient,
    address: &Pubkey,
    label: &str,
) -> Result<T> {
    let account = fetch_program_account(client, address, label)?;
    T::try_deserialize(&mut account.data.as_slice())
        .with_context(|| format!("failed to deserialize {label}"))
}

pub fn fetch_pool(client: &KzpClient, pool: &Pubkey) -> Result<Pool> {
    deserialize_program_account(client, pool, "pool")
}

pub fn fetch_loan(client: &KzpClient, loan: &Pubkey) -> Result<Loan> {
    deserialize_program_account(client, loan, "loan")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_check_rejects_non_program_account() {
        let address = Pubkey::new_unique();
        let account = Account {
            lamports: 0,
            data: Vec::new(),
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };

        let err = ensure_program_owner(&address, &account, "loan").unwrap_err();
        assert!(err.to_string().contains("expected"));
    }
}
