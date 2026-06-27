use crate::helpers::{funded_admin, member_with_savings, TestApp};
use kzp_mini::state::LoanStatus;
use kzp_mini::utils::pda::member_pda;
use kzp_mini::PoolError;
use solana_sdk::{pubkey::Pubkey, signature::Signer};

/// Shared fixtures for [`settle_default`](kzp_mini::settle_default) integration tests.
struct Universe {
    pool: Pubkey,
    admin: solana_sdk::signature::Keypair,
    borrower: solana_sdk::signature::Keypair,
    guarantor_a: solana_sdk::signature::Keypair,
    guarantor_b: solana_sdk::signature::Keypair,
    borrower_ata: Pubkey,
}

/// Initializes a pool with admin, borrower, and two guarantors (2B savings each).
async fn universe(app: &mut TestApp) -> Universe {
    let admin = funded_admin(app).await;
    let pool = app.setup_pool(&admin).await;
    let (borrower, borrower_ata) = member_with_savings(app, pool, 2_000_000_000).await;
    let (guarantor_a, _) = member_with_savings(app, pool, 2_000_000_000).await;
    let (guarantor_b, _) = member_with_savings(app, pool, 2_000_000_000).await;
    Universe {
        pool,
        admin,
        borrower,
        guarantor_a,
        guarantor_b,
        borrower_ata,
    }
}

// Spec: nominal - admin settles default after due date from reserved liability.
//
// Given:
// - An active loan that is due for default settlement
//
// When:
// - Admin calls `settle_default`
//
// Then:
// - Loan is `Defaulted`, pool counters update, and no fresh guarantor signatures are required
with_universe!(settle_default_nominal, |app, u| {
    let amount = 1_000_000_000;
    let loan_key = app
        .activate_loan_with_term(
            &u.borrower,
            u.pool,
            31,
            amount,
            &u.guarantor_a,
            &u.guarantor_b,
            u.borrower_ata,
            0,
        )
        .await;

    let (vault, _) = kzp_mini::utils::pda::vault_pda(&u.pool);
    let vault_before = app.token_balance(&vault).await;

    let ix = app.settle_default(&u.admin, loan_key).await;
    app.process(&[ix], &[&u.admin]).await;

    let loan = app.fetch_loan(&loan_key).await;
    assert_eq!(loan.status, LoanStatus::Defaulted);
    assert_eq!(loan.outstanding, 0);

    let pool_after = app.fetch_pool(&u.pool).await;
    let (ga_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
    let (gb_member, _) = member_pda(&u.pool, &u.guarantor_b.pubkey());
    let ga_member = app.fetch_member(&ga_member).await;
    let gb_member = app.fetch_member(&gb_member).await;
    assert_eq!(pool_after.total_outstanding_loans, 0);
    assert_eq!(pool_after.total_savings, 6_000_000_000 - amount);
    assert_eq!(ga_member.locked_savings, 0);
    assert_eq!(gb_member.locked_savings, 0);
    assert_eq!(app.token_balance(&vault).await, vault_before);
    app.assert_vault_covers_liquid_savings(&u.pool).await;
});

// Spec: edge - odd outstanding amount is split deterministically and reconciles exactly.
//
// Given:
// - An active loan with an odd outstanding amount
//
// When:
// - Admin settles the default after the due date
//
// Then:
// - Guarantor A pays the rounded-up share, guarantor B pays the remainder, and totals reconcile
with_universe!(settle_default_splits_odd_outstanding_exactly, |app, u| {
    let amount = 1_000_000_001;
    let loan_key = app
        .activate_loan_with_term(
            &u.borrower,
            u.pool,
            34,
            amount,
            &u.guarantor_a,
            &u.guarantor_b,
            u.borrower_ata,
            0,
        )
        .await;

    let pool_before = app.fetch_pool(&u.pool).await;
    let (ga_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
    let (gb_member, _) = member_pda(&u.pool, &u.guarantor_b.pubkey());
    let ga_before = app.fetch_member(&ga_member).await;
    let gb_before = app.fetch_member(&gb_member).await;

    let ix = app.settle_default(&u.admin, loan_key).await;
    app.process(&[ix], &[&u.admin]).await;

    let ga_after = app.fetch_member(&ga_member).await;
    let gb_after = app.fetch_member(&gb_member).await;
    let pool_after = app.fetch_pool(&u.pool).await;

    assert_eq!(
        ga_before.savings_balance - ga_after.savings_balance,
        500_000_001
    );
    assert_eq!(
        gb_before.savings_balance - gb_after.savings_balance,
        500_000_000
    );
    assert_eq!(pool_before.total_savings - pool_after.total_savings, amount);
    assert_eq!(pool_after.total_outstanding_loans, 0);
    assert_eq!(ga_after.locked_savings, 0);
    assert_eq!(gb_after.locked_savings, 0);
});

// Spec: edge - settlement releases guarantees but clears only the matching borrower active loan.
//
// Given:
// - An active loan ready for default settlement
// - Borrower's member account has drifted to point at a different active loan
//
// When:
// - Admin settles the original defaulted loan
//
// Then:
// - Guarantor obligations are released and the non-matching borrower active loan is preserved
with_universe!(
    settle_default_preserves_nonmatching_borrower_active_loan,
    |app, u| {
        let amount = 600_000_000;
        let loan_key = app
            .activate_loan_with_term(
                &u.borrower,
                u.pool,
                35,
                amount,
                &u.guarantor_a,
                &u.guarantor_b,
                u.borrower_ata,
                0,
            )
            .await;

        let other_active_loan = Pubkey::new_unique();
        let (borrower_member, _) = member_pda(&u.pool, &u.borrower.pubkey());
        let mut borrower_state = app.fetch_member(&borrower_member).await;
        borrower_state.active_loan = Some(other_active_loan);
        app.overwrite_member(&borrower_member, &borrower_state)
            .await;

        let ix = app.settle_default(&u.admin, loan_key).await;
        app.process(&[ix], &[&u.admin]).await;

        let borrower_after = app.fetch_member(&borrower_member).await;
        let (ga_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
        let (gb_member, _) = member_pda(&u.pool, &u.guarantor_b.pubkey());
        let ga_after = app.fetch_member(&ga_member).await;
        let gb_after = app.fetch_member(&gb_member).await;
        let loan_after = app.fetch_loan(&loan_key).await;

        assert_eq!(borrower_after.active_loan, Some(other_active_loan));
        assert_eq!(ga_after.locked_savings, 0);
        assert_eq!(gb_after.locked_savings, 0);
        assert_eq!(ga_after.active_guarantee_count, 0);
        assert_eq!(gb_after.active_guarantee_count, 0);
        assert_eq!(loan_after.guarantor_a_locked_savings, 0);
        assert_eq!(loan_after.guarantor_b_locked_savings, 0);
    }
);

// Spec: defensive - settlement member accounts must still record the loan's pool.
//
// Given:
// - An active loan ready for default settlement
// - One guarantor member PDA has a drifted `pool` field
//
// When:
// - Admin calls `settle_default`
//
// Then:
// - Transaction fails before ledger state is mutated
with_universe!(
    settle_default_fails_when_guarantor_member_pool_mismatches,
    |app, u| {
        let amount = 600_000_000;
        let loan_key = app
            .activate_loan_with_term(
                &u.borrower,
                u.pool,
                36,
                amount,
                &u.guarantor_a,
                &u.guarantor_b,
                u.borrower_ata,
                0,
            )
            .await;

        let pool_before = app.fetch_pool(&u.pool).await;
        let loan_before = app.fetch_loan(&loan_key).await;
        let (ga_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
        let mut ga_state = app.fetch_member(&ga_member).await;
        ga_state.pool = Pubkey::new_unique();
        app.overwrite_member(&ga_member, &ga_state).await;

        let ix = app.settle_default(&u.admin, loan_key).await;
        app.process_expect_err(&[ix], &[&u.admin]).await;

        let pool_after = app.fetch_pool(&u.pool).await;
        let loan_after = app.fetch_loan(&loan_key).await;
        assert_eq!(
            pool_after.total_outstanding_loans,
            pool_before.total_outstanding_loans
        );
        assert_eq!(pool_after.total_savings, pool_before.total_savings);
        assert_eq!(loan_after.status, loan_before.status);
        assert_eq!(loan_after.outstanding, loan_before.outstanding);
    }
);

// Spec: edge - default before due date is rejected (`LoanNotDue`).
//
// Given:
// - An active loan whose due date is still in the future
//
// When:
// - Admin calls `settle_default`
//
// Then:
// - Transaction fails with `LoanNotDue`
with_universe!(settle_default_fails_before_due_date, |app, u| {
    let loan_key = app
        .activate_loan_with_term(
            &u.borrower,
            u.pool,
            33,
            500_000_000,
            &u.guarantor_a,
            &u.guarantor_b,
            u.borrower_ata,
            30 * 24 * 60 * 60,
        )
        .await;

    let ix = app.settle_default(&u.admin, loan_key).await;
    app.process_expect_custom_err(&[ix], &[&u.admin], PoolError::LoanNotDue)
        .await;
});

// Spec: edge - non-admin cannot settle (`NotPoolAdmin`).
//
// Given:
// - An active loan ready for settlement
//
// When:
// - Borrower (non-admin) initiates `settle_default`
//
// Then:
// - Transaction fails with `NotPoolAdmin`
with_universe!(settle_default_fails_non_admin, |app, u| {
    let loan_key = app
        .activate_loan_with_term(
            &u.borrower,
            u.pool,
            32,
            500_000_000,
            &u.guarantor_a,
            &u.guarantor_b,
            u.borrower_ata,
            0,
        )
        .await;

    let ix = app.settle_default(&u.borrower, loan_key).await;
    app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::NotPoolAdmin)
        .await;
});
