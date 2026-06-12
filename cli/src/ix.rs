//! Converts Anchor instructions to `solana-sdk` instructions.

use anchor_lang::solana_program::instruction::Instruction as ProgramInstruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

/// Bridges Anchor-generated instructions to the SDK type used by the RPC client.
pub fn to_sdk_instruction(ix: ProgramInstruction) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(ix.program_id.to_bytes()),
        accounts: ix
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: Pubkey::new_from_array(meta.pubkey.to_bytes()),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: ix.data,
    }
}
