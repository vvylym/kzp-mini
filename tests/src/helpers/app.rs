//! In-process test application: BPF program + SPL mint for `kzp-mini` integration tests.

use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::spl_token;
use anchor_spl::token::spl_token::state::Mint as MintState;
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use kzp_mini::ID as PROGRAM_ID;
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer},
};

/// Local test app wrapping `solana-program-test` with `kzp-mini` and a shared SPL mint.
pub struct TestApp {
    pub banks: BanksClient,
    /// Pays rent for program-derived accounts and ATA creation.
    pub payer: Keypair,
    /// SPL mint used as the pool token for every test.
    pub mint: Keypair,
    pub mint_authority: Keypair,
    pub(super) blockhash: solana_sdk::hash::Hash,
}

impl TestApp {
    /// Deploys `kzp_mini.so` and initializes a 6-decimal test mint.
    pub async fn new() -> Self {
        let bpf_out = format!("{}/../target/deploy", env!("CARGO_MANIFEST_DIR"));
        std::env::set_var("BPF_OUT_DIR", &bpf_out);

        let mut program_test = ProgramTest::default();
        program_test.prefer_bpf(true);
        program_test.add_program("kzp_mini", PROGRAM_ID, None);

        let (banks, payer, blockhash) = program_test.start().await;
        let mint = Keypair::new();
        let mint_authority = Keypair::new();

        let mut app = Self {
            banks,
            payer,
            mint,
            mint_authority,
            blockhash,
        };
        app.init_mint().await;
        app
    }

    async fn init_mint(&mut self) {
        let rent = self.banks.get_rent().await.expect("rent");
        let lamports = rent.minimum_balance(MintState::LEN);

        let ixs = [
            system_instruction::create_account(
                &self.payer.pubkey(),
                &self.mint.pubkey(),
                lamports,
                MintState::LEN as u64,
                &TOKEN_PROGRAM_ID,
            ),
            spl_token::instruction::initialize_mint2(
                &TOKEN_PROGRAM_ID,
                &self.mint.pubkey(),
                &self.mint_authority.pubkey(),
                None,
                6,
            )
            .unwrap(),
        ];
        let mint_signer = Keypair::try_from(self.mint.to_bytes().as_ref()).unwrap();
        self.process(&ixs, &[&mint_signer]).await;
    }
}
