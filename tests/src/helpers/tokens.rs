//! SPL token helpers: ATAs, minting, and balance reads.

use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use anchor_spl::token::spl_token;
use anchor_spl::token::spl_token::state::Account as TokenAccountState;
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use super::app::TestApp;
use super::constants::LAMPORTS;
impl TestApp {
    /// Returns the ATA address for `owner` under this harness mint.
    pub fn ata_for(&self, owner: &Pubkey) -> Pubkey {
        get_associated_token_address(owner, &self.mint.pubkey())
    }

    /// Creates an idempotent ATA for `owner` and optionally mints `fund_amount` into it.
    pub async fn ensure_ata(&mut self, owner: &Keypair, fund_amount: u64) -> Pubkey {
        let ata = get_associated_token_address(&owner.pubkey(), &self.mint.pubkey());
        let create_ix = create_associated_token_account_idempotent(
            &self.payer.pubkey(),
            &owner.pubkey(),
            &self.mint.pubkey(),
            &TOKEN_PROGRAM_ID,
        );
        self.process(&[create_ix], &[]).await;
        if fund_amount > 0 {
            self.mint_to(&ata, fund_amount).await;
        }
        ata
    }

    /// Mints test tokens from the harness mint into `dest`.
    pub async fn mint_to(&mut self, dest: &Pubkey, amount: u64) {
        let ix = spl_token::instruction::mint_to(
            &TOKEN_PROGRAM_ID,
            &self.mint.pubkey(),
            dest,
            &self.mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap();
        let mint_authority = Keypair::try_from(self.mint_authority.to_bytes().as_ref()).unwrap();
        self.process(&[ix], &[&mint_authority]).await;
    }

    /// Reads the SPL token balance of an ATA.
    pub async fn token_balance(&mut self, ata: &Pubkey) -> u64 {
        let account = self.banks.get_account(*ata).await.unwrap().unwrap();
        TokenAccountState::unpack(&account.data).unwrap().amount
    }

    /// New wallet with `LAMPORTS` SOL and an ATA holding `token_amount`.
    pub async fn create_funded_user(&mut self, token_amount: u64) -> (Keypair, Pubkey) {
        let user = Keypair::new();
        self.airdrop_sol(&user.pubkey(), LAMPORTS).await;
        let ata = self.ensure_ata(&user, token_amount).await;
        (user, ata)
    }
}
