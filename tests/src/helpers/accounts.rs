//! On-chain account deserialization for assertions.

use anchor_lang::AccountDeserialize;
use kzp_mini::state::{Loan, Member, Pool};
use solana_sdk::pubkey::Pubkey;

use super::app::TestApp;

impl TestApp {
    /// Fetches and deserializes a [`Pool`] account.
    pub async fn fetch_pool(&mut self, pool: &Pubkey) -> Pool {
        let account = self.banks.get_account(*pool).await.unwrap().unwrap();
        Pool::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    /// Fetches and deserializes a [`Member`] account.
    pub async fn fetch_member(&mut self, member: &Pubkey) -> Member {
        let account = self.banks.get_account(*member).await.unwrap().unwrap();
        Member::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    /// Fetches and deserializes a [`Loan`] account.
    pub async fn fetch_loan(&mut self, loan: &Pubkey) -> Loan {
        let account = self.banks.get_account(*loan).await.unwrap().unwrap();
        Loan::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    /// Returns whether an account exists at `key`.
    pub async fn account_exists(&mut self, key: &Pubkey) -> bool {
        self.banks.get_account(*key).await.unwrap().is_some()
    }
}
