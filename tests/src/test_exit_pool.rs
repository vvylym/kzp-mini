use crate::helpers::{initialized_pool, member_with_savings, TestApp};
use kzp_mini::utils::pda::member_pda;
use kzp_mini::PoolError;
use solana_sdk::{pubkey::Pubkey, signature::Signer};

/// Shared fixtures for [`exit_pool`](kzp_mini::exit_pool) integration tests.
struct Universe {
    pool: Pubkey,
}

/// Initializes an empty pool (no members beyond those created per test).
async fn universe(app: &mut TestApp) -> Universe {
    let (_admin, pool) = initialized_pool(app).await;
    Universe { pool }
}

// Spec: nominal - member withdraws savings and member account is closed.
//
// Given:
// - A pool member with 2B savings and no loan or guarantee obligations
//
// When:
// - Member calls `exit_pool`
//
// Then:
// - Savings are returned to the member ATA, member PDA is closed, pool counters drop to zero
with_universe!(exit_pool_nominal, |app, u| {
    let (carol, carol_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let balance_before = app.token_balance(&carol_ata).await;
    let (member_key, _) = member_pda(&u.pool, &carol.pubkey());

    let ix = app.exit_pool(&carol, u.pool, carol_ata);
    app.process(&[ix], &[&carol]).await;

    let pool_state = app.fetch_pool(&u.pool).await;
    assert_eq!(pool_state.total_members, 0);
    assert_eq!(pool_state.total_savings, 0);
    assert!(!app.account_exists(&member_key).await);
    assert_eq!(
        app.token_balance(&carol_ata).await,
        balance_before + 2_000_000_000
    );
});

// Spec: edge - member with outstanding loan cannot exit (`OutstandingLoanExists`).
//
// Given:
// - Carol is the borrower on an active disbursed loan
//
// When:
// - Carol calls `exit_pool`
//
// Then:
// - Transaction fails with `OutstandingLoanExists`
with_universe!(exit_pool_fails_with_outstanding_loan, |app, u| {
    let (carol, carol_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (dave, _) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (eve, _) = member_with_savings(app, u.pool, 2_000_000_000).await;

    app.activate_loan(&carol, u.pool, 1, 1_000_000_000, &dave, &eve, carol_ata)
        .await;

    let ix = app.exit_pool(&carol, u.pool, carol_ata);
    app.process_expect_custom_err(&[ix], &[&carol], PoolError::OutstandingLoanExists)
        .await;
});

// Spec: edge - member with pending borrower loan cannot exit (`OutstandingLoanExists`).
//
// Given:
// - Carol has requested a loan that is still pending
//
// When:
// - Carol calls `exit_pool`
//
// Then:
// - Transaction fails with `OutstandingLoanExists`
with_universe!(exit_pool_fails_with_pending_borrower_loan, |app, u| {
    let (carol, carol_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (dave, _) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (eve, _) = member_with_savings(app, u.pool, 2_000_000_000).await;

    let ix_req = app.request_loan(
        &carol,
        u.pool,
        7,
        1_000_000_000,
        dave.pubkey(),
        eve.pubkey(),
    );
    app.process(&[ix_req], &[&carol]).await;

    let ix = app.exit_pool(&carol, u.pool, carol_ata);
    app.process_expect_custom_err(&[ix], &[&carol], PoolError::OutstandingLoanExists)
        .await;
});

// Spec: edge - member guaranteeing another loan cannot exit (`ActiveGuaranteesExist`).
//
// Given:
// - Carol is a guarantor on Bob's active loan
// - Carol has no loan of her own
//
// When:
// - Carol calls `exit_pool`
//
// Then:
// - Transaction fails with `ActiveGuaranteesExist`
with_universe!(exit_pool_fails_with_active_guarantees, |app, u| {
    let (bob, bob_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (carol, carol_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (dave, _) = member_with_savings(app, u.pool, 2_000_000_000).await;

    app.activate_loan(&bob, u.pool, 1, 1_000_000_000, &carol, &dave, bob_ata)
        .await;

    let ix = app.exit_pool(&carol, u.pool, carol_ata);
    app.process_expect_custom_err(&[ix], &[&carol], PoolError::ActiveGuaranteesExist)
        .await;
});

// Spec: edge - member with pending co-sign obligation cannot exit (`PendingGuaranteesExist`).
//
// Given:
// - Carol has co-signed Bob's pending loan but disbursement has not occurred
//
// When:
// - Carol calls `exit_pool`
//
// Then:
// - Transaction fails with `PendingGuaranteesExist`
with_universe!(exit_pool_fails_with_pending_guarantees, |app, u| {
    let (bob, bob_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (carol, carol_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (dave, _) = member_with_savings(app, u.pool, 2_000_000_000).await;

    let ix_req = app.request_loan(
        &bob,
        u.pool,
        3,
        1_000_000_000,
        carol.pubkey(),
        dave.pubkey(),
    );
    app.process(&[ix_req], &[&bob]).await;
    let (loan, _) = kzp_mini::utils::pda::loan_pda(&u.pool, &bob.pubkey(), 3);
    let ix_cosign = app.co_sign_loan(&carol, loan, bob_ata).await;
    app.process(&[ix_cosign], &[&carol]).await;

    let ix = app.exit_pool(&carol, u.pool, carol_ata);
    app.process_expect_custom_err(&[ix], &[&carol], PoolError::PendingGuaranteesExist)
        .await;
});

// Spec: edge - member with both own loan and guarantees blocked (`OutstandingLoanExists` first).
//
// Given:
// - Carol guarantees Frank's loan and also borrows on her own active loan
//
// When:
// - Carol calls `exit_pool`
//
// Then:
// - Transaction fails with `OutstandingLoanExists` (checked before guarantees)
with_universe!(exit_pool_fails_with_both_loan_and_guarantees, |app, u| {
    let (carol, carol_ata) = member_with_savings(app, u.pool, 5_000_000_000).await;
    let (dave, _) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (eve, _) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let (frank, frank_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;

    app.activate_loan(&frank, u.pool, 2, 500_000_000, &carol, &dave, frank_ata)
        .await;
    app.activate_loan(&carol, u.pool, 1, 1_000_000_000, &eve, &dave, carol_ata)
        .await;

    let ix = app.exit_pool(&carol, u.pool, carol_ata);
    app.process_expect_custom_err(&[ix], &[&carol], PoolError::OutstandingLoanExists)
        .await;
});
