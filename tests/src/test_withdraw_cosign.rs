use crate::helpers::{TestApp, initialized_pool, member_with_savings};
use kzp_mini::error::PoolError;
use kzp_mini::utils::pda::{loan_pda, member_pda};
use solana_sdk::{pubkey::Pubkey, signature::Signer};

/// Shared fixtures for [`withdraw_cosign`](kzp_mini::withdraw_cosign) integration tests.
struct Universe {
    pool: Pubkey,
    borrower: solana_sdk::signature::Keypair,
    guarantor_a: solana_sdk::signature::Keypair,
    guarantor_b: solana_sdk::signature::Keypair,
    borrower_ata: Pubkey,
}

/// Initializes a pool with a borrower and two guarantors (2B savings each).
async fn universe(app: &mut TestApp) -> Universe {
    let (_admin, pool) = initialized_pool(app).await;
    let (borrower, borrower_ata) = member_with_savings(app, pool, 2_000_000_000).await;
    let (guarantor_a, _) = member_with_savings(app, pool, 2_000_000_000).await;
    let (guarantor_b, _) = member_with_savings(app, pool, 2_000_000_000).await;
    Universe {
        pool,
        borrower,
        guarantor_a,
        guarantor_b,
        borrower_ata,
    }
}

// Spec: nominal - guarantor withdraws partial co-sign; pending ref cleared.
//
// Given:
// - A pending loan where guarantor A has co-signed
//
// When:
// - Guarantor A calls `withdraw_cosign`
//
// Then:
// - Co-sign flag is cleared and pending guarantee ref is removed
with_universe!(withdraw_cosign_nominal, |app, u| {
    let loan_nonce = 21;
    let ix_req = app.request_loan(
        &u.borrower,
        u.pool,
        loan_nonce,
        1_000_000_000,
        u.guarantor_a.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process(&[ix_req], &[&u.borrower]).await;
    let (loan_key, _) = loan_pda(&u.pool, &u.borrower.pubkey(), loan_nonce);

    let ix_cosign = app
        .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
        .await;
    app.process(&[ix_cosign], &[&u.guarantor_a]).await;

    let ix_withdraw = app.withdraw_cosign(&u.guarantor_a, loan_key).await;
    app.process(&[ix_withdraw], &[&u.guarantor_a]).await;

    let loan = app.fetch_loan(&loan_key).await;
    assert!(!loan.guarantor_a_signed);

    let (ga_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
    let ga = app.fetch_member(&ga_member).await;
    assert_eq!(ga.pending_guarantee_count, 0);
});

// Spec: edge - cannot withdraw when not co-signed (`NotCoSigned`).
//
// Given:
// - A pending loan where guarantor A has not co-signed
//
// When:
// - Guarantor A calls `withdraw_cosign`
//
// Then:
// - Transaction fails with `NotCoSigned`
with_universe!(withdraw_cosign_fails_when_not_signed, |app, u| {
    let loan_nonce = 22;
    let ix_req = app.request_loan(
        &u.borrower,
        u.pool,
        loan_nonce,
        1_000_000_000,
        u.guarantor_a.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process(&[ix_req], &[&u.borrower]).await;
    let (loan_key, _) = loan_pda(&u.pool, &u.borrower.pubkey(), loan_nonce);

    let ix = app.withdraw_cosign(&u.guarantor_a, loan_key).await;
    app.process_expect_custom_err(&[ix], &[&u.guarantor_a], PoolError::NotCoSigned)
        .await;
});

// Spec: edge - cannot withdraw co-sign once loan is active (`LoanNotPending`).
//
// Given:
// - A loan that both guarantors co-signed and activated
//
// When:
// - Guarantor A calls `withdraw_cosign`
//
// Then:
// - Transaction fails with `LoanNotPending`
with_universe!(withdraw_cosign_fails_when_loan_active, |app, u| {
    let loan_key = app
        .activate_loan(
            &u.borrower,
            u.pool,
            23,
            1_000_000_000,
            &u.guarantor_a,
            &u.guarantor_b,
            u.borrower_ata,
        )
        .await;

    let ix = app.withdraw_cosign(&u.guarantor_a, loan_key).await;
    app.process_expect_custom_err(&[ix], &[&u.guarantor_a], PoolError::LoanNotPending)
        .await;
});

// Spec: edge - non-nominated member cannot withdraw a co-sign (`NotNominatedGuarantor`).
//
// Given:
// - A pending loan with guarantors A and B
// - Eve is a pool member but not nominated on the loan
//
// When:
// - Eve calls `withdraw_cosign`
//
// Then:
// - Transaction fails with `NotNominatedGuarantor`
with_universe!(withdraw_cosign_fails_when_not_nominated, |app, u| {
    let (eve, _) = member_with_savings(app, u.pool, 2_000_000_000).await;
    let loan_nonce = 24;
    let ix_req = app.request_loan(
        &u.borrower,
        u.pool,
        loan_nonce,
        1_000_000_000,
        u.guarantor_a.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process(&[ix_req], &[&u.borrower]).await;
    let (loan_key, _) = loan_pda(&u.pool, &u.borrower.pubkey(), loan_nonce);

    let ix = app.withdraw_cosign(&eve, loan_key).await;
    app.process_expect_custom_err(&[ix], &[&eve], PoolError::NotNominatedGuarantor)
        .await;
});
