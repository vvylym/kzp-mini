use crate::helpers::{initialized_pool, member_with_savings, TestApp};
use kzp_mini::utils::pda::{loan_pda, member_pda};
use kzp_mini::PoolError;
use solana_sdk::{pubkey::Pubkey, signature::Signer};

/// Shared fixtures for [`cancel_loan`](kzp_mini::cancel_loan) integration tests.
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

// Spec: nominal - borrower cancels a pending loan; guarantor pending refs cleared.
//
// Given:
// - A pending loan where guarantor A has co-signed
//
// When:
// - Borrower calls `cancel_loan`
//
// Then:
// - Loan account is closed and guarantor pending guarantee refs are cleared
with_universe!(cancel_loan_nominal, |app, u| {
    let loan_nonce = 11;
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

    let ix_cancel = app.cancel_loan(&u.borrower, loan_key).await;
    app.process(&[ix_cancel], &[&u.borrower]).await;

    assert!(!app.account_exists(&loan_key).await);

    let (ga_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
    let ga = app.fetch_member(&ga_member).await;
    assert!(!ga.pending_guarantees.contains(&loan_key));
});

// Spec: edge - cannot cancel an active loan (`LoanNotPending`).
//
// Given:
// - A fully disbursed active loan
//
// When:
// - Borrower calls `cancel_loan`
//
// Then:
// - Transaction fails with `LoanNotPending`
with_universe!(cancel_loan_fails_when_active, |app, u| {
    let loan_key = app
        .activate_loan(
            &u.borrower,
            u.pool,
            12,
            500_000_000,
            &u.guarantor_a,
            &u.guarantor_b,
            u.borrower_ata,
        )
        .await;

    let ix = app.cancel_loan(&u.borrower, loan_key).await;
    app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::LoanNotPending)
        .await;
});
