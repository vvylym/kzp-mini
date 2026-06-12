use crate::helpers::{initialized_pool, member_with_savings, TestApp};
use kzp_mini::state::LoanStatus;
use kzp_mini::utils::pda::loan_pda;
use kzp_mini::PoolError;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

/// Shared fixtures for [`request_loan`](kzp_mini::request_loan) integration tests.
struct Universe {
    pool: Pubkey,
    borrower: Keypair,
    guarantor_a: Keypair,
    guarantor_b: Keypair,
    borrower_ata: Pubkey,
}

impl Universe {
    async fn request(&self, app: &mut TestApp, loan_nonce: u64, amount: u64) -> Pubkey {
        let ix = app.request_loan(
            &self.borrower,
            self.pool,
            loan_nonce,
            amount,
            self.guarantor_a.pubkey(),
            self.guarantor_b.pubkey(),
        );
        app.process(&[ix], &[&self.borrower]).await;
        loan_pda(&self.pool, &self.borrower.pubkey(), loan_nonce).0
    }
}

/// Initializes a pool with a borrower (1B savings) and two guarantors.
async fn universe(app: &mut TestApp) -> Universe {
    let (_admin, pool) = initialized_pool(app).await;

    let (borrower, borrower_ata) = member_with_savings(app, pool, 1_000_000_000).await;
    let (guarantor_a, _) = member_with_savings(app, pool, 2_000_000_000).await;
    let (guarantor_b, _) = member_with_savings(app, pool, 1_500_000_000).await;

    Universe {
        pool,
        borrower,
        guarantor_a,
        guarantor_b,
        borrower_ata,
    }
}

/// Guarantor B already backs five active loans; fresh borrower and guarantor A.
async fn universe_guarantor_at_limit(app: &mut TestApp) -> Universe {
    let (_admin, pool) = initialized_pool(app).await;

    let (guarantor_b, _) = member_with_savings(app, pool, 5_000_000_000).await;

    for i in 0..5 {
        let (borrower, borrower_ata) = member_with_savings(app, pool, 1_000_000_000).await;
        let (g2, _) = member_with_savings(app, pool, 1_000_000_000).await;
        app.activate_loan(
            &borrower,
            pool,
            i,
            500_000_000,
            &guarantor_b,
            &g2,
            borrower_ata,
        )
        .await;
    }

    let (borrower, borrower_ata) = member_with_savings(app, pool, 1_000_000_000).await;
    let (guarantor_a, _) = member_with_savings(app, pool, 1_000_000_000).await;

    Universe {
        pool,
        borrower,
        guarantor_a,
        guarantor_b,
        borrower_ata,
    }
}

// Spec: nominal - borrower creates pending loan with two guarantors.
//
// Given:
// - A pool with a borrower and two eligible guarantor members
//
// When:
// - Borrower calls `request_loan` within the savings multiplier limit
//
// Then:
// - Loan PDA is `Pending` with correct principal, guarantors, and unsigned flags
with_universe!(request_loan_nominal, |app, u| {
    let loan_key = u.request(app, 42, 2_000_000_000).await;
    let loan = app.fetch_loan(&loan_key).await;

    assert_eq!(loan.pool, u.pool);
    assert_eq!(loan.borrower, u.borrower.pubkey());
    assert_eq!(loan.principal, 2_000_000_000);
    assert_eq!(loan.outstanding, 2_000_000_000);
    assert_eq!(loan.guarantor_a, u.guarantor_a.pubkey());
    assert_eq!(loan.guarantor_b, u.guarantor_b.pubkey());
    assert!(!loan.guarantor_a_signed);
    assert!(!loan.guarantor_b_signed);
    assert_eq!(loan.status, LoanStatus::Pending);
});

// Spec: edge - borrower already has an active loan (`ExistingActiveLoan`).
//
// Given:
// - A borrower with an active disbursed loan
//
// When:
// - The same borrower calls `request_loan` again
//
// Then:
// - Transaction fails with `ExistingActiveLoan`
with_universe!(
    request_loan_fails_when_borrower_has_active_loan,
    |app, u| {
        app.activate_loan(
            &u.borrower,
            u.pool,
            1,
            500_000_000,
            &u.guarantor_a,
            &u.guarantor_b,
            u.borrower_ata,
        )
        .await;

        let ix = app.request_loan(
            &u.borrower,
            u.pool,
            2,
            500_000_000,
            u.guarantor_a.pubkey(),
            u.guarantor_b.pubkey(),
        );
        app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::ExistingActiveLoan)
            .await;
    }
);

// Spec: edge - zero loan amount (`LoanAmountMustBePositive`).
//
// Given:
// - An eligible borrower and guarantors
//
// When:
// - Borrower calls `request_loan` with amount `0`
//
// Then:
// - Transaction fails with `LoanAmountMustBePositive`
with_universe!(request_loan_fails_on_zero_amount, |app, u| {
    let ix = app.request_loan(
        &u.borrower,
        u.pool,
        42,
        0,
        u.guarantor_a.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::LoanAmountMustBePositive)
        .await;
});

// Spec: edge - amount exceeds savings × max multiplier (`LoanExceedsMaxMultiplier`).
//
// Given:
// - A borrower with 1B savings (max loan 2B at 2× multiplier)
//
// When:
// - Borrower requests 4B
//
// Then:
// - Transaction fails with `LoanExceedsMaxMultiplier`
with_universe!(request_loan_fails_when_exceeds_max_multiplier, |app, u| {
    let ix = app.request_loan(
        &u.borrower,
        u.pool,
        42,
        4_000_000_000,
        u.guarantor_a.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::LoanExceedsMaxMultiplier)
        .await;
});

// Spec: edge - borrower cannot guarantee own loan (`SelfGuaranteeNotAllowed`).
//
// Given:
// - An eligible borrower
//
// When:
// - Borrower nominates themselves as guarantor A
//
// Then:
// - Transaction fails with `SelfGuaranteeNotAllowed`
with_universe!(request_loan_fails_on_self_guarantee, |app, u| {
    let ix = app.request_loan(
        &u.borrower,
        u.pool,
        42,
        1_000_000_000,
        u.borrower.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::SelfGuaranteeNotAllowed)
        .await;
});

// Spec: edge - same pubkey for both guarantors (`DuplicateGuarantors`).
//
// Given:
// - An eligible borrower
//
// When:
// - Borrower nominates the same member for both guarantor slots
//
// Then:
// - Transaction fails with `DuplicateGuarantors`
with_universe!(request_loan_fails_on_duplicate_guarantors, |app, u| {
    let ix = app.request_loan(
        &u.borrower,
        u.pool,
        42,
        1_000_000_000,
        u.guarantor_a.pubkey(),
        u.guarantor_a.pubkey(),
    );
    app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::DuplicateGuarantors)
        .await;
});

// Spec: edge - guarantor is not a pool member (member PDA missing).
//
// Given:
// - A borrower and one valid guarantor member
// - A wallet that never joined the pool
//
// When:
// - Borrower nominates the stranger as guarantor A
//
// Then:
// - Transaction fails (guarantor member account not found)
with_universe!(request_loan_fails_when_guarantor_not_member, |app, u| {
    let stranger = Keypair::new();

    let ix = app.request_loan(
        &u.borrower,
        u.pool,
        42,
        1_000_000_000,
        stranger.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process_expect_err(&[ix], &[&u.borrower]).await;
});

// Spec: edge - guarantor already has an active loan (`GuarantorHasActiveLoan`).
//
// Given:
// - Guarantor A is the borrower on another active loan
//
// When:
// - A different member requests a loan with guarantor A
//
// Then:
// - Transaction fails with `GuarantorHasActiveLoan`
with_universe!(
    request_loan_fails_when_guarantor_has_active_loan,
    |app, u| {
        let carol_ata = app.ata_for(&u.guarantor_a.pubkey());

        app.activate_loan(
            &u.guarantor_a,
            u.pool,
            10,
            500_000_000,
            &u.borrower,
            &u.guarantor_b,
            carol_ata,
        )
        .await;

        let ix = app.request_loan(
            &u.borrower,
            u.pool,
            42,
            1_000_000_000,
            u.guarantor_a.pubkey(),
            u.guarantor_b.pubkey(),
        );
        app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::GuarantorHasActiveLoan)
            .await;
    }
);

// Spec: edge - guarantor already backing max concurrent loans (`GuarantorLimitReached`).
//
// Given:
// - Guarantor B already guarantees five active loans
// - Fresh borrower Bob and guarantor Carol
//
// When:
// - Bob requests a loan with Dave and Carol as guarantors
//
// Then:
// - Transaction fails with `GuarantorLimitReached`
with_universe!(
    request_loan_fails_when_guarantor_limit_reached,
    universe_guarantor_at_limit,
    |app, u| {
        let ix = app.request_loan(
            &u.borrower,
            u.pool,
            99,
            500_000_000,
            u.guarantor_b.pubkey(),
            u.guarantor_a.pubkey(),
        );
        app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::GuarantorLimitReached)
            .await;
    }
);

// Spec: edge - duplicate loan nonce for same borrower (Anchor `init` on loan PDA).
//
// Given:
// - A pending loan already exists for borrower nonce 42
//
// When:
// - Borrower calls `request_loan` again with the same nonce
//
// Then:
// - Transaction fails (loan PDA already initialized)
with_universe!(request_loan_fails_when_loan_nonce_already_used, |app, u| {
    u.request(app, 42, 1_000_000_000).await;

    let ix_dup = app.request_loan(
        &u.borrower,
        u.pool,
        42,
        500_000_000,
        u.guarantor_a.pubkey(),
        u.guarantor_b.pubkey(),
    );
    app.process_expect_err(&[ix_dup], &[&u.borrower]).await;
});
