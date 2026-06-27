//! Pool instruction commands.

use anchor_lang::{InstructionData, ToAccountMetas, system_program};
use anchor_spl::{associated_token::get_associated_token_address, token::ID as TOKEN_PROGRAM_ID};
use anyhow::{Context, Result};
use kzp_mini::{ID as PROGRAM_ID, accounts, instruction};
use solana_sdk::{pubkey::Pubkey, signature::Signer, sysvar};

use crate::accounts::fetch_pool;
use crate::commands::CommandContext;
use crate::ix::to_sdk_instruction;
use crate::pda::{member_pda, pool_pda, vault_pda};

fn parse_pubkey(label: &str, value: &str) -> Result<Pubkey> {
    value
        .parse()
        .with_context(|| format!("invalid {label} pubkey: {value}"))
}

pub(crate) fn anchor_pubkey(pk: Pubkey) -> anchor_lang::prelude::Pubkey {
    anchor_lang::prelude::Pubkey::new_from_array(pk.to_bytes())
}

pub fn initialize_pool(
    ctx: &CommandContext,
    pool_name: &str,
    entry_fee: u64,
    mint: &str,
    admin: Option<&str>,
) -> Result<()> {
    let admin_pubkey = match admin {
        Some(value) => parse_pubkey("admin", value)?,
        None => ctx.client.payer_pubkey(),
    };

    let mint = parse_pubkey("mint", mint)?;
    let (pool, _) = pool_pda(&admin_pubkey, pool_name);
    let (vault, _) = vault_pda(&pool);

    let accounts = accounts::InitializePool {
        admin: anchor_pubkey(admin_pubkey),
        pool: anchor_pubkey(pool),
        vault: anchor_pubkey(vault),
        token_mint: anchor_pubkey(mint),
        system_program: system_program::ID,
        token_program: TOKEN_PROGRAM_ID,
        rent: anchor_pubkey(sysvar::rent::ID),
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::InitializePool {
            pool_name: pool_name.to_string(),
            required_entry_fee: entry_fee,
        }
        .data(),
    });

    let sig = ctx.client.send_instructions(&[ix], &[], ctx.dry_run)?;
    println!("Pool initialized: {pool}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

pub fn join_pool(ctx: &CommandContext, pool_str: &str, entry_fee: u64) -> Result<()> {
    let pool = parse_pubkey("pool", pool_str)?;
    let member = ctx.client.payer();
    let mint = fetch_mint(ctx, pool)?;
    let member_ata =
        get_associated_token_address(&anchor_pubkey(member.pubkey()), &anchor_pubkey(mint));
    let member_ata = Pubkey::new_from_array(member_ata.to_bytes());
    let (member_account, _) = member_pda(&pool, &member.pubkey());
    let (vault, _) = vault_pda(&pool);

    let accounts = accounts::JoinPool {
        member: anchor_pubkey(member.pubkey()),
        member_account: anchor_pubkey(member_account),
        pool: anchor_pubkey(pool),
        vault: anchor_pubkey(vault),
        member_token_account: anchor_pubkey(member_ata),
        token_program: TOKEN_PROGRAM_ID,
        system_program: system_program::ID,
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::JoinPool { entry_fee }.data(),
    });

    let sig = ctx
        .client
        .send_instructions(&[ix], &[member], ctx.dry_run)?;
    println!("Joined pool {pool} as member {member_account}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

pub fn deposit_savings(ctx: &CommandContext, pool_str: &str, amount: u64) -> Result<()> {
    let pool = parse_pubkey("pool", pool_str)?;
    let member = ctx.client.payer();
    let mint = fetch_mint(ctx, pool)?;
    let member_ata = Pubkey::new_from_array(
        get_associated_token_address(&anchor_pubkey(member.pubkey()), &anchor_pubkey(mint))
            .to_bytes(),
    );
    let (member_account, _) = member_pda(&pool, &member.pubkey());
    let (vault, _) = vault_pda(&pool);

    let accounts = accounts::DepositSavings {
        member: anchor_pubkey(member.pubkey()),
        member_account: anchor_pubkey(member_account),
        pool: anchor_pubkey(pool),
        vault: anchor_pubkey(vault),
        member_token_account: anchor_pubkey(member_ata),
        token_program: TOKEN_PROGRAM_ID,
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::DepositSavings { amount }.data(),
    });

    let sig = ctx
        .client
        .send_instructions(&[ix], &[member], ctx.dry_run)?;
    println!("Deposited {amount} into pool {pool}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

pub fn exit_pool(ctx: &CommandContext, pool_str: &str) -> Result<()> {
    let pool = parse_pubkey("pool", pool_str)?;
    let member = ctx.client.payer();
    let mint = fetch_mint(ctx, pool)?;
    let member_ata = Pubkey::new_from_array(
        get_associated_token_address(&anchor_pubkey(member.pubkey()), &anchor_pubkey(mint))
            .to_bytes(),
    );
    let (member_account, _) = member_pda(&pool, &member.pubkey());
    let (vault, _) = vault_pda(&pool);

    let accounts = accounts::ExitPool {
        member: anchor_pubkey(member.pubkey()),
        member_account: anchor_pubkey(member_account),
        pool: anchor_pubkey(pool),
        vault: anchor_pubkey(vault),
        member_token_account: anchor_pubkey(member_ata),
        token_program: TOKEN_PROGRAM_ID,
    };

    let ix = to_sdk_instruction(anchor_lang::solana_program::instruction::Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts.to_account_metas(None),
        data: instruction::ExitPool {}.data(),
    });

    let sig = ctx
        .client
        .send_instructions(&[ix], &[member], ctx.dry_run)?;
    println!("Exited pool {pool}");
    if !ctx.dry_run {
        println!("Signature: {sig}");
    }
    Ok(())
}

pub(crate) fn fetch_mint(ctx: &CommandContext, pool: Pubkey) -> Result<Pubkey> {
    let pool_state = fetch_pool(&ctx.client, &pool)?;
    Ok(Pubkey::new_from_array(pool_state.token_mint.to_bytes()))
}
