//! Instruction builders mirroring the on-chain `kzp-mini` account layouts.

use anchor_lang::{InstructionData, ToAccountMetas, system_program};
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use kzp_mini::{ID as PROGRAM_ID, accounts, instruction};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use kzp_mini::utils::pda::{loan_pda, member_pda, pool_pda, vault_pda};

use super::app::TestApp;
use super::constants::DEFAULT_LOAN_TERM_SECONDS;

impl TestApp {
    pub fn initialize_pool(
        &self,
        admin: &Keypair,
        pool_name: &str,
        required_entry_fee: u64,
    ) -> Instruction {
        let (pool, _) = pool_pda(&admin.pubkey(), pool_name);
        let (vault, _) = vault_pda(&pool);
        let accounts = accounts::InitializePool {
            admin: admin.pubkey(),
            pool,
            vault,
            token_mint: self.mint.pubkey(),
            system_program: system_program::ID,
            token_program: TOKEN_PROGRAM_ID,
            rent: solana_sdk::sysvar::rent::ID,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::InitializePool {
                pool_name: pool_name.to_string(),
                required_entry_fee,
            }
            .data(),
        }
    }

    /// `vault_override` is used by negative tests that pass a wrong vault PDA.
    pub fn join_pool(
        &self,
        member: &Keypair,
        pool: Pubkey,
        member_ata: Pubkey,
        entry_fee: u64,
        vault_override: Option<Pubkey>,
    ) -> Instruction {
        let (member_account, _) = member_pda(&pool, &member.pubkey());
        let vault = vault_override.unwrap_or_else(|| vault_pda(&pool).0);
        let accounts = accounts::JoinPool {
            member: member.pubkey(),
            member_account,
            pool,
            vault,
            member_token_account: member_ata,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::JoinPool { entry_fee }.data(),
        }
    }

    pub fn deposit_savings(
        &self,
        member: &Keypair,
        pool: Pubkey,
        member_ata: Pubkey,
        amount: u64,
    ) -> Instruction {
        let (member_account, _) = member_pda(&pool, &member.pubkey());
        let (vault, _) = vault_pda(&pool);
        let accounts = accounts::DepositSavings {
            member: member.pubkey(),
            member_account,
            pool,
            vault,
            member_token_account: member_ata,
            token_program: TOKEN_PROGRAM_ID,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::DepositSavings { amount }.data(),
        }
    }

    pub fn request_loan(
        &self,
        borrower: &Keypair,
        pool: Pubkey,
        loan_nonce: u64,
        amount: u64,
        guarantor_a: Pubkey,
        guarantor_b: Pubkey,
    ) -> Instruction {
        self.request_loan_with_term(
            borrower,
            pool,
            loan_nonce,
            amount,
            guarantor_a,
            guarantor_b,
            DEFAULT_LOAN_TERM_SECONDS,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_loan_with_term(
        &self,
        borrower: &Keypair,
        pool: Pubkey,
        loan_nonce: u64,
        amount: u64,
        guarantor_a: Pubkey,
        guarantor_b: Pubkey,
        loan_term_seconds: i64,
    ) -> Instruction {
        let (member_account, _) = member_pda(&pool, &borrower.pubkey());
        let (loan, _) = loan_pda(&pool, &borrower.pubkey(), loan_nonce);
        let (guarantor_a_member, _) = member_pda(&pool, &guarantor_a);
        let (guarantor_b_member, _) = member_pda(&pool, &guarantor_b);
        let accounts = accounts::RequestLoan {
            borrower: borrower.pubkey(),
            member_account,
            pool,
            loan,
            guarantor_a_member,
            guarantor_a,
            guarantor_b_member,
            guarantor_b,
            system_program: system_program::ID,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::RequestLoan {
                loan_nonce,
                amount,
                loan_term_seconds,
            }
            .data(),
        }
    }

    /// Resolves pool and member PDAs from the on-chain [`Loan`] (no pool account in ix).
    pub async fn co_sign_loan(
        &mut self,
        guarantor: &Keypair,
        loan: Pubkey,
        borrower_ata: Pubkey,
    ) -> Instruction {
        let loan_state = self.fetch_loan(&loan).await;
        let pool = loan_state.pool;
        let signer = guarantor.pubkey();
        let (guarantor_member, _) = member_pda(&pool, &signer);
        let completes_activation = (signer == loan_state.guarantor_a
            && loan_state.guarantor_b_signed)
            || (signer == loan_state.guarantor_b && loan_state.guarantor_a_signed);
        let accounts = accounts::CoSignLoan {
            guarantor: signer,
            loan,
            guarantor_member,
        };
        let mut metas = accounts.to_account_metas(None);
        if completes_activation {
            let other_guarantor = if signer == loan_state.guarantor_a {
                loan_state.guarantor_b
            } else {
                loan_state.guarantor_a
            };
            let (other_guarantor_member, _) = member_pda(&pool, &other_guarantor);
            let (borrower_member, _) = member_pda(&pool, &loan_state.borrower);
            let (vault, _) = vault_pda(&pool);
            metas.extend([
                AccountMeta::new(pool, false),
                AccountMeta::new(other_guarantor_member, false),
                AccountMeta::new(borrower_member, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(borrower_ata, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ]);
        }
        Instruction {
            program_id: PROGRAM_ID,
            accounts: metas,
            data: instruction::CoSignLoan {}.data(),
        }
    }

    pub async fn co_sign_loan_base_only(
        &mut self,
        guarantor: &Keypair,
        loan: Pubkey,
    ) -> Instruction {
        let loan_state = self.fetch_loan(&loan).await;
        let (guarantor_member, _) = member_pda(&loan_state.pool, &guarantor.pubkey());
        let accounts = accounts::CoSignLoan {
            guarantor: guarantor.pubkey(),
            loan,
            guarantor_member,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::CoSignLoan {}.data(),
        }
    }

    /// Resolves pool, vault, and guarantor member PDAs from the on-chain [`Loan`].
    pub async fn repay_loan(
        &mut self,
        borrower: &Keypair,
        loan: Pubkey,
        borrower_ata: Pubkey,
        amount: u64,
    ) -> Instruction {
        let loan_state = self.fetch_loan(&loan).await;
        let pool = loan_state.pool;
        let (vault, _) = vault_pda(&pool);
        let accounts = accounts::RepayLoan {
            borrower: borrower.pubkey(),
            loan,
            pool,
            vault,
            borrower_token_account: borrower_ata,
            token_program: TOKEN_PROGRAM_ID,
        };
        let mut metas = accounts.to_account_metas(None);
        if amount == loan_state.outstanding {
            let (borrower_member, _) = member_pda(&pool, &borrower.pubkey());
            let (guarantor_a_member, _) = member_pda(&pool, &loan_state.guarantor_a);
            let (guarantor_b_member, _) = member_pda(&pool, &loan_state.guarantor_b);
            metas.extend([
                AccountMeta::new(borrower_member, false),
                AccountMeta::new(guarantor_a_member, false),
                AccountMeta::new(guarantor_b_member, false),
            ]);
        }
        Instruction {
            program_id: PROGRAM_ID,
            accounts: metas,
            data: instruction::RepayLoan { amount }.data(),
        }
    }

    pub async fn repay_loan_base_only(
        &mut self,
        borrower: &Keypair,
        loan: Pubkey,
        borrower_ata: Pubkey,
        amount: u64,
    ) -> Instruction {
        let loan_state = self.fetch_loan(&loan).await;
        let pool = loan_state.pool;
        let (vault, _) = vault_pda(&pool);
        let accounts = accounts::RepayLoan {
            borrower: borrower.pubkey(),
            loan,
            pool,
            vault,
            borrower_token_account: borrower_ata,
            token_program: TOKEN_PROGRAM_ID,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::RepayLoan { amount }.data(),
        }
    }

    pub fn exit_pool(&self, member: &Keypair, pool: Pubkey, member_ata: Pubkey) -> Instruction {
        let (member_account, _) = member_pda(&pool, &member.pubkey());
        let (vault, _) = vault_pda(&pool);
        let accounts = accounts::ExitPool {
            member: member.pubkey(),
            member_account,
            pool,
            vault,
            member_token_account: member_ata,
            token_program: TOKEN_PROGRAM_ID,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::ExitPool {}.data(),
        }
    }

    pub async fn cancel_loan(&mut self, borrower: &Keypair, loan: Pubkey) -> Instruction {
        let loan_state = self.fetch_loan(&loan).await;
        let pool = loan_state.pool;
        let (borrower_member, _) = member_pda(&pool, &borrower.pubkey());
        let (guarantor_a_member, _) = member_pda(&pool, &loan_state.guarantor_a);
        let (guarantor_b_member, _) = member_pda(&pool, &loan_state.guarantor_b);
        let accounts = accounts::CancelLoan {
            borrower: borrower.pubkey(),
            loan,
            borrower_member,
            guarantor_a_member,
            guarantor_b_member,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::CancelLoan {}.data(),
        }
    }

    pub async fn withdraw_cosign(&mut self, guarantor: &Keypair, loan: Pubkey) -> Instruction {
        let loan_state = self.fetch_loan(&loan).await;
        let pool = loan_state.pool;
        let (guarantor_a_member, _) = member_pda(&pool, &loan_state.guarantor_a);
        let (guarantor_b_member, _) = member_pda(&pool, &loan_state.guarantor_b);
        let accounts = accounts::WithdrawCosign {
            guarantor: guarantor.pubkey(),
            loan,
            guarantor_a_member,
            guarantor_b_member,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::WithdrawCosign {}.data(),
        }
    }

    pub async fn settle_default(&mut self, crank: &Keypair, loan: Pubkey) -> Instruction {
        let loan_state = self.fetch_loan(&loan).await;
        let pool = loan_state.pool;
        let (borrower_member, _) = member_pda(&pool, &loan_state.borrower);
        let (guarantor_a_member, _) = member_pda(&pool, &loan_state.guarantor_a);
        let (guarantor_b_member, _) = member_pda(&pool, &loan_state.guarantor_b);
        let accounts = accounts::SettleDefault {
            crank: crank.pubkey(),
            pool,
            loan,
            borrower: loan_state.borrower,
            borrower_member,
            guarantor_a_member,
            guarantor_b_member,
        };
        Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction::SettleDefault {}.data(),
        }
    }
}
