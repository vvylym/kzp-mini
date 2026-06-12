use crate::helpers::{funded_admin, TestApp, ENTRY_FEE, POOL_NAME};
use kzp_mini::utils::pda::{pool_pda, vault_pda};
use kzp_mini::PoolError;
use solana_sdk::signature::{Keypair, Signer};

/// Shared fixtures for [`initialize_pool`](kzp_mini::initialize_pool) integration tests.
struct Universe {
    admin: Keypair,
}

/// Provides a funded admin wallet ready to create a pool.
async fn universe(app: &mut TestApp) -> Universe {
    Universe {
        admin: funded_admin(app).await,
    }
}

// Spec: nominal - admin creates pool PDA, vault, and initial counters are zero.
//
// Given:
// - A deployed program and SPL mint
// - A funded admin wallet
//
// When:
// - Admin calls `initialize_pool` with a valid name and entry fee
//
// Then:
// - Pool and vault PDAs exist with zero members, savings, and loans
// - Pool metadata matches admin, mint, vault, and entry fee
with_universe!(initialize_pool_nominal, |app, u| {
    let (pool, _) = pool_pda(&u.admin.pubkey(), POOL_NAME);
    let (vault, _) = vault_pda(&pool);

    let ix = app.initialize_pool(&u.admin, POOL_NAME, ENTRY_FEE);
    app.process(&[ix], &[&u.admin]).await;

    let pool_state = app.fetch_pool(&pool).await;
    assert_eq!(pool_state.admin, u.admin.pubkey());
    assert_eq!(pool_state.token_mint, app.mint.pubkey());
    assert_eq!(pool_state.vault, vault);
    assert_eq!(pool_state.required_entry_fee, ENTRY_FEE);
    assert_eq!(pool_state.total_members, 0);
    assert_eq!(pool_state.total_savings, 0);
    assert_eq!(pool_state.total_outstanding_loans, 0);
    assert_eq!(app.token_balance(&vault).await, 0);
});

// Spec: edge - duplicate `initialize_pool` for same PDA fails (Anchor `init` constraint).
//
// Given:
// - A pool already initialized for the admin and pool name
//
// When:
// - Admin calls `initialize_pool` again with the same name
//
// Then:
// - Transaction fails (member PDA already exists)
with_universe!(initialize_pool_fails_when_pool_already_exists, |app, u| {
    let ix = app.initialize_pool(&u.admin, POOL_NAME, ENTRY_FEE);
    app.process(&[ix], &[&u.admin]).await;

    let ix_dup = app.initialize_pool(&u.admin, POOL_NAME, ENTRY_FEE);
    app.process_expect_err(&[ix_dup], &[&u.admin]).await;
});

// Spec: edge - pool name shorter than minimum (`PoolNameTooShort`).
//
// Given:
// - A funded admin wallet
//
// When:
// - Admin calls `initialize_pool` with an empty pool name
//
// Then:
// - Transaction fails with `PoolNameTooShort`
with_universe!(initialize_pool_fails_on_empty_name, |app, u| {
    let ix = app.initialize_pool(&u.admin, "", ENTRY_FEE);
    app.process_expect_custom_err(&[ix], &[&u.admin], PoolError::PoolNameTooShort)
        .await;
});

// Spec: edge - token mint account is not a valid mint (Anchor mint constraint).
//
// Given:
// - A funded admin wallet
//
// When:
// - Admin calls `initialize_pool` but passes a non-mint account as `token_mint`
//
// Then:
// - Transaction fails at the mint validation constraint
with_universe!(initialize_pool_fails_on_invalid_mint, |app, u| {
    let fake_mint = Keypair::new().pubkey();
    let mut ix = app.initialize_pool(&u.admin, POOL_NAME, ENTRY_FEE);
    for meta in ix.accounts.iter_mut() {
        if meta.pubkey == app.mint.pubkey() {
            meta.pubkey = fake_mint;
        }
    }
    app.process_expect_err(&[ix], &[&u.admin]).await;
});
