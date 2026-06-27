use crate::helpers::{ENTRY_FEE, TestApp, initialized_pool};
use kzp_mini::error::PoolError;
use kzp_mini::utils::pda::{member_pda, vault_pda};
use solana_sdk::{pubkey::Pubkey, signature::Signer};

/// Shared fixtures for [`join_pool`](kzp_mini::join_pool) integration tests.
struct Universe {
    pool: Pubkey,
}

/// Initializes a pool with no members yet.
async fn universe(app: &mut TestApp) -> Universe {
    let (_admin, pool) = initialized_pool(app).await;
    Universe { pool }
}

// Spec: nominal - member pays entry fee only; savings remain zero until `deposit_savings`.
//
// Given:
// - An initialized pool
// - A funded user with tokens in their ATA
//
// When:
// - User calls `join_pool` paying the required entry fee
//
// Then:
// - Member PDA is created with zero savings and no loan or guarantees
// - Pool `total_members` is 1 and vault balance increases by the entry fee
with_universe!(join_pool_nominal_entry_fee_only, |app, u| {
    let (bob, bob_ata) = app.create_funded_user(500_000_000).await;
    let vault_before = app.token_balance(&vault_pda(&u.pool).0).await;

    app.join_member(&bob, u.pool, bob_ata).await;

    let (member_key, _) = member_pda(&u.pool, &bob.pubkey());
    let member = app.fetch_member(&member_key).await;
    let pool_state = app.fetch_pool(&u.pool).await;

    assert_eq!(member.pool, u.pool);
    assert_eq!(member.member_id, 0);
    assert_eq!(member.owner, bob.pubkey());
    assert_eq!(member.entry_fee_paid, ENTRY_FEE);
    assert_eq!(member.savings_balance, 0);
    assert_eq!(member.locked_savings, 0);
    assert!(member.active_loan.is_none());
    assert!(member.pending_loan.is_none());
    assert_eq!(member.active_guarantee_count, 0);
    assert_eq!(member.pending_guarantee_count, 0);
    assert_eq!(pool_state.total_members, 1);
    assert_eq!(pool_state.total_savings, 0);
    assert_eq!(
        app.token_balance(&vault_pda(&u.pool).0).await,
        vault_before + ENTRY_FEE
    );
});

// Spec: nominal - join then deposit (entry fee + separate savings deposit).
//
// Given:
// - An initialized pool
// - A funded user with enough tokens for entry fee and savings
//
// When:
// - User joins the pool then calls `deposit_savings`
//
// Then:
// - Member savings and pool `total_savings` reflect the deposit
with_universe!(
    join_pool_nominal_with_initial_savings_via_deposit,
    |app, u| {
        let (bob, bob_ata) = app.create_funded_user(500_000_000 + ENTRY_FEE).await;
        app.join_with_savings(&bob, u.pool, bob_ata, 500_000_000)
            .await;

        let (member_key, _) = member_pda(&u.pool, &bob.pubkey());
        let member = app.fetch_member(&member_key).await;
        let pool_state = app.fetch_pool(&u.pool).await;

        assert_eq!(member.savings_balance, 500_000_000);
        assert_eq!(pool_state.total_members, 1);
        assert_eq!(pool_state.total_savings, 500_000_000);
    }
);

// Spec: edge - second `join_pool` for same member fails (Anchor `init` on member PDA).
//
// Given:
// - A user who is already a pool member
//
// When:
// - The same user calls `join_pool` again
//
// Then:
// - Transaction fails (member account already initialized)
with_universe!(join_pool_fails_when_already_member, |app, u| {
    let (bob, bob_ata) = app.create_funded_user(500_000_000).await;
    app.join_member(&bob, u.pool, bob_ata).await;

    let ix = app.join_pool(&bob, u.pool, bob_ata, ENTRY_FEE, None);
    app.process_expect_err(&[ix], &[&bob]).await;
});

// Spec: edge - entry fee does not match pool requirement (`InvalidEntryFeeAmount`).
//
// Given:
// - An initialized pool with a non-zero required entry fee
// - A funded non-member
//
// When:
// - User calls `join_pool` with entry fee `0`
//
// Then:
// - Transaction fails with `InvalidEntryFeeAmount`
with_universe!(join_pool_fails_on_wrong_entry_fee, |app, u| {
    let (bob, bob_ata) = app.create_funded_user(500_000_000).await;
    let ix = app.join_pool(&bob, u.pool, bob_ata, 0, None);
    app.process_expect_custom_err(&[ix], &[&bob], PoolError::InvalidEntryFeeAmount)
        .await;
});

// Spec: edge - member ATA cannot cover entry fee (SPL transfer failure).
//
// Given:
// - An initialized pool
// - A user whose ATA holds fewer tokens than the entry fee
//
// When:
// - User calls `join_pool`
//
// Then:
// - Transaction fails at the SPL transfer
with_universe!(join_pool_fails_on_insufficient_token_balance, |app, u| {
    let (bob, bob_ata) = app.create_funded_user(10).await;
    let ix = app.join_pool(&bob, u.pool, bob_ata, ENTRY_FEE, None);
    app.process_expect_err(&[ix], &[&bob]).await;
});

// Spec: edge - vault account does not belong to this pool (PDA seeds constraint).
//
// Given:
// - An initialized pool and a second pool with its own vault
// - A funded non-member of the first pool
//
// When:
// - User calls `join_pool` on pool A but passes pool B's vault
//
// Then:
// - Transaction fails at the vault PDA constraint
with_universe!(join_pool_fails_on_invalid_vault, |app, u| {
    let (bob, bob_ata) = app.create_funded_user(500_000_000).await;
    let (_other_admin, other_pool) = initialized_pool(app).await;
    let wrong_vault = vault_pda(&other_pool).0;

    let ix = app.join_pool(&bob, u.pool, bob_ata, ENTRY_FEE, Some(wrong_vault));
    app.process_expect_err(&[ix], &[&bob]).await;
});
