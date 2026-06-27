//! On-chain account deserialization for assertions.

use anchor_lang::{AccountDeserialize, AccountSerialize};
use anchor_spl::token::spl_token::state::Account as TokenAccountState;
use kzp_mini::state::{Loan, Member, Pool};
use solana_sdk::{account::AccountSharedData, program_pack::Pack, pubkey::Pubkey};

use super::app::TestApp;

impl TestApp {
    /// Fetches and deserializes a [`Pool`] account.
    pub async fn fetch_pool(&mut self, pool: &Pubkey) -> Pool {
        let account = self
            .context
            .banks_client
            .get_account(*pool)
            .await
            .unwrap()
            .unwrap();
        Pool::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    /// Fetches and deserializes a [`Member`] account.
    pub async fn fetch_member(&mut self, member: &Pubkey) -> Member {
        let account = self
            .context
            .banks_client
            .get_account(*member)
            .await
            .unwrap()
            .unwrap();
        Member::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    /// Fetches and deserializes a [`Loan`] account.
    pub async fn fetch_loan(&mut self, loan: &Pubkey) -> Loan {
        let account = self
            .context
            .banks_client
            .get_account(*loan)
            .await
            .unwrap()
            .unwrap();
        Loan::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    /// Returns whether an account exists at `key`.
    pub async fn account_exists(&mut self, key: &Pubkey) -> bool {
        self.context
            .banks_client
            .get_account(*key)
            .await
            .unwrap()
            .is_some()
    }

    /// Overwrites a member PDA with synthetic state for defensive invariant tests.
    pub async fn overwrite_member(&mut self, member_key: &Pubkey, member: &Member) {
        let mut account = self
            .context
            .banks_client
            .get_account(*member_key)
            .await
            .unwrap()
            .unwrap();
        let mut data = Vec::with_capacity(account.data.len());
        member.try_serialize(&mut data).unwrap();
        assert!(data.len() <= account.data.len());
        account.data[..data.len()].copy_from_slice(&data);
        self.context
            .set_account(member_key, &AccountSharedData::from(account));
    }

    /// Overwrites an SPL token account amount for defensive liquidity tests.
    pub async fn overwrite_token_amount(&mut self, token_account: &Pubkey, amount: u64) {
        let mut account = self
            .context
            .banks_client
            .get_account(*token_account)
            .await
            .unwrap()
            .unwrap();
        let mut token = TokenAccountState::unpack(&account.data).unwrap();
        token.amount = amount;
        TokenAccountState::pack(token, &mut account.data).unwrap();
        self.context
            .set_account(token_account, &AccountSharedData::from(account));
    }
}
