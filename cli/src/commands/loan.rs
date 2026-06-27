//! Loan instruction commands.

use anchor_lang::{InstructionData, ToAccountMetas, system_program};
use anchor_spl::{associated_token::get_associated_token_address, token::ID as TOKEN_PROGRAM_ID};
use anyhow::{Context, Result};
use kzp_mini::{ID as PROGRAM_ID, accounts, instruction};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signature::Signer};

use crate::accounts::fetch_loan;
use crate::commands::CommandContext;
use crate::commands::pool::{anchor_pubkey, fetch_mint};
use crate::ix::to_sdk_instruction;
use crate::pda::{loan_pda, member_pda, vault_pda};

fn parse_pubkey(label: &str, value: &str) -> Result<Pubkey> {
    value
        .parse()
        .with_context(|| format!("invalid {label} pubkey: {value}"))
}

pub fn request_loan(
    ctx: &CommandContext,
    pool_str: &str,
    nonce: u64,
    amount: u64,
    guarantor_a: &str,
    guarantor_b: &str,
    term_seconds: i64,
) -> Result<()> {
    let pool = parse_pubkey("pool", pool_str)?;
    let borrower = ctx.client.payer();
    let guarantor_a = parse_pubkey("guarantor_a", guarantor_a)?;
    let guarantor_b = parse_pubkey("guarantor_b", guarantor_b)?;

    let (member_account, _) = member_pda(&pool, &borrower.pubkey());
    let (loan, _) = loan_pda(&pool, &borrower.pubkey(), nonce);
    let (guarantor_a_member, _) = member_pda(&pool, &guarantor_a);
    let (guarantor_b_member, _) = member_pda(&pool, &guarantor_b);

    let accounts = accounts::RequestLoan {
        borrower: anchor_pubkey(borrower.pubkey()),
        member_account: anchor_pubkey(member_account),
        pool: anchor_pubkey(pool),
        loan: anchor_pubkey(loan),
        guarantor_a_member: anchor_pubkey(guarantor_a_member),
        guarantor_a: anchor_pubkey(guarantor_a),
        guarantor_b_member: anchor_pubkey(guarantor_b_member),
        guarantor_b: anchor_pubkey(guarantor_b),
        system_program: system_program::ID,
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::RequestLoan {
            loan_nonce: nonce,
            amount,
            loan_term_seconds: term_seconds,
        }
        .data(),
    });

    let sig = ctx
        .client
        .send_instructions(&[ix], &[borrower], ctx.dry_run)?;
    println!("Loan requested: {loan}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

/// Co-signs a loan; pool and borrower are resolved from the loan account on-chain.
pub fn co_sign_loan(
    ctx: &CommandContext,
    loan_str: &str,
    borrower_ata: Option<&str>,
) -> Result<()> {
    let loan = parse_pubkey("loan", loan_str)?;
    let loan_state = fetch_loan(&ctx.client, &loan)?;
    let pool = Pubkey::new_from_array(loan_state.pool.to_bytes());
    let guarantor = ctx.client.payer();
    let signer = guarantor.pubkey();

    let (guarantor_member, _) = member_pda(&pool, &signer);

    let accounts = accounts::CoSignLoan {
        guarantor: anchor_pubkey(signer),
        loan: anchor_pubkey(loan),
        guarantor_member: anchor_pubkey(guarantor_member),
    };

    let mut ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::CoSignLoan {}.data(),
    });
    let guarantor_a = Pubkey::new_from_array(loan_state.guarantor_a.to_bytes());
    let guarantor_b = Pubkey::new_from_array(loan_state.guarantor_b.to_bytes());
    let completes_activation = (signer == guarantor_a && loan_state.guarantor_b_signed)
        || (signer == guarantor_b && loan_state.guarantor_a_signed);
    if completes_activation {
        let borrower = Pubkey::new_from_array(loan_state.borrower.to_bytes());
        let mint = fetch_mint(ctx, pool)?;
        let borrower_token_account = match borrower_ata {
            Some(value) => parse_pubkey("borrower_ata", value)?,
            None => Pubkey::new_from_array(
                get_associated_token_address(&anchor_pubkey(borrower), &anchor_pubkey(mint))
                    .to_bytes(),
            ),
        };
        let other_guarantor = if signer == guarantor_a {
            guarantor_b
        } else {
            guarantor_a
        };
        let (other_guarantor_member, _) = member_pda(&pool, &other_guarantor);
        let (borrower_member, _) = member_pda(&pool, &borrower);
        let (vault, _) = vault_pda(&pool);
        ix.accounts.extend([
            AccountMeta::new(pool, false),
            AccountMeta::new(other_guarantor_member, false),
            AccountMeta::new(borrower_member, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(borrower_token_account, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ]);
    }

    let sig = ctx
        .client
        .send_instructions(&[ix], &[guarantor], ctx.dry_run)?;
    println!("Co-signed loan {loan} (pool {pool})");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

/// Repays a loan; pool and guarantors are resolved from the loan account on-chain.
pub fn repay_loan(ctx: &CommandContext, loan_str: &str, amount: u64) -> Result<()> {
    let loan = parse_pubkey("loan", loan_str)?;
    let loan_state = fetch_loan(&ctx.client, &loan)?;
    let pool = Pubkey::new_from_array(loan_state.pool.to_bytes());
    let guarantor_a = Pubkey::new_from_array(loan_state.guarantor_a.to_bytes());
    let guarantor_b = Pubkey::new_from_array(loan_state.guarantor_b.to_bytes());
    let borrower = ctx.client.payer();

    let mint = fetch_mint(ctx, pool)?;
    let borrower_ata = Pubkey::new_from_array(
        get_associated_token_address(&anchor_pubkey(borrower.pubkey()), &anchor_pubkey(mint))
            .to_bytes(),
    );
    let (vault, _) = vault_pda(&pool);

    let accounts = accounts::RepayLoan {
        borrower: anchor_pubkey(borrower.pubkey()),
        loan: anchor_pubkey(loan),
        pool: anchor_pubkey(pool),
        vault: anchor_pubkey(vault),
        borrower_token_account: anchor_pubkey(borrower_ata),
        token_program: TOKEN_PROGRAM_ID,
    };

    let mut ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::RepayLoan { amount }.data(),
    });
    if amount == loan_state.outstanding {
        let (borrower_member, _) = member_pda(&pool, &borrower.pubkey());
        let (guarantor_a_member, _) = member_pda(&pool, &guarantor_a);
        let (guarantor_b_member, _) = member_pda(&pool, &guarantor_b);
        ix.accounts.extend([
            AccountMeta::new(borrower_member, false),
            AccountMeta::new(guarantor_a_member, false),
            AccountMeta::new(guarantor_b_member, false),
        ]);
    }

    let sig = ctx
        .client
        .send_instructions(&[ix], &[borrower], ctx.dry_run)?;
    println!("Repaid {amount} on loan {loan} (pool {pool})");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

/// Cancels a pending loan (borrower only).
pub fn cancel_loan(ctx: &CommandContext, loan_str: &str) -> Result<()> {
    let loan = parse_pubkey("loan", loan_str)?;
    let loan_state = fetch_loan(&ctx.client, &loan)?;
    let pool = Pubkey::new_from_array(loan_state.pool.to_bytes());
    let borrower = ctx.client.payer();
    let guarantor_a = Pubkey::new_from_array(loan_state.guarantor_a.to_bytes());
    let guarantor_b = Pubkey::new_from_array(loan_state.guarantor_b.to_bytes());
    let (borrower_member, _) = member_pda(&pool, &borrower.pubkey());
    let (guarantor_a_member, _) = member_pda(&pool, &guarantor_a);
    let (guarantor_b_member, _) = member_pda(&pool, &guarantor_b);

    let accounts = accounts::CancelLoan {
        borrower: anchor_pubkey(borrower.pubkey()),
        loan: anchor_pubkey(loan),
        borrower_member: anchor_pubkey(borrower_member),
        guarantor_a_member: anchor_pubkey(guarantor_a_member),
        guarantor_b_member: anchor_pubkey(guarantor_b_member),
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::CancelLoan {}.data(),
    });

    let sig = ctx
        .client
        .send_instructions(&[ix], &[borrower], ctx.dry_run)?;
    println!("Cancelled pending loan {loan}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

/// Withdraws a partial co-sign on a pending loan.
pub fn withdraw_cosign(ctx: &CommandContext, loan_str: &str) -> Result<()> {
    let loan = parse_pubkey("loan", loan_str)?;
    let loan_state = fetch_loan(&ctx.client, &loan)?;
    let pool = Pubkey::new_from_array(loan_state.pool.to_bytes());
    let guarantor = ctx.client.payer();
    let guarantor_a = Pubkey::new_from_array(loan_state.guarantor_a.to_bytes());
    let guarantor_b = Pubkey::new_from_array(loan_state.guarantor_b.to_bytes());
    let (guarantor_a_member, _) = member_pda(&pool, &guarantor_a);
    let (guarantor_b_member, _) = member_pda(&pool, &guarantor_b);

    let accounts = accounts::WithdrawCosign {
        guarantor: anchor_pubkey(guarantor.pubkey()),
        loan: anchor_pubkey(loan),
        guarantor_a_member: anchor_pubkey(guarantor_a_member),
        guarantor_b_member: anchor_pubkey(guarantor_b_member),
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::WithdrawCosign {}.data(),
    });

    let sig = ctx
        .client
        .send_instructions(&[ix], &[guarantor], ctx.dry_run)?;
    println!("Withdrew co-sign on loan {loan}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

/// Permissionlessly settles an active loan as defaulted from reserved guarantor savings.
pub fn settle_default(ctx: &CommandContext, loan_str: &str, pool_str: &str) -> Result<()> {
    let loan = parse_pubkey("loan", loan_str)?;
    let pool = parse_pubkey("pool", pool_str)?;
    let crank = ctx.client.payer();
    let loan_state = fetch_loan(&ctx.client, &loan)?;
    let borrower = Pubkey::new_from_array(loan_state.borrower.to_bytes());
    let guarantor_a = Pubkey::new_from_array(loan_state.guarantor_a.to_bytes());
    let guarantor_b = Pubkey::new_from_array(loan_state.guarantor_b.to_bytes());
    let (borrower_member, _) = member_pda(&pool, &borrower);
    let (guarantor_a_member, _) = member_pda(&pool, &guarantor_a);
    let (guarantor_b_member, _) = member_pda(&pool, &guarantor_b);

    let accounts = accounts::SettleDefault {
        crank: anchor_pubkey(crank.pubkey()),
        pool: anchor_pubkey(pool),
        loan: anchor_pubkey(loan),
        borrower: anchor_pubkey(borrower),
        borrower_member: anchor_pubkey(borrower_member),
        guarantor_a_member: anchor_pubkey(guarantor_a_member),
        guarantor_b_member: anchor_pubkey(guarantor_b_member),
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::SettleDefault {}.data(),
    });

    let sig = ctx.client.send_instructions(&[ix], &[crank], ctx.dry_run)?;
    println!("Settled default on loan {loan}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}
