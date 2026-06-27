use crate::helpers::{initialized_pool, member_with_savings, TestApp, LAMPORTS};
use anchor_spl::token::{spl_token, ID as TOKEN_PROGRAM_ID};
use kzp_mini::state::LoanStatus;
use kzp_mini::utils::pda::{member_pda, vault_pda};
use kzp_mini::PoolError;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

/// Shared fixtures for [`repay_loan`](kzp_mini::repay_loan) integration tests.
struct Universe {
    pool: Pubkey,
    borrower: Keypair,
    guarantor_a: Keypair,
    guarantor_b: Keypair,
    borrower_ata: Pubkey,
    loan: Pubkey,
    amount: u64,
}

/// Initializes a pool and activates a loan for `amount` principal.
async fn universe(app: &mut TestApp, amount: u64) -> Universe {
    let (_admin, pool) = initialized_pool(app).await;

    let (borrower, borrower_ata) = member_with_savings(app, pool, 2_000_000_000).await;
    let (guarantor_a, _) = member_with_savings(app, pool, 2_000_000_000).await;
    let (guarantor_b, _) = member_with_savings(app, pool, 2_000_000_000).await;

    let loan = app
        .activate_loan(
            &borrower,
            pool,
            1,
            amount,
            &guarantor_a,
            &guarantor_b,
            borrower_ata,
        )
        .await;

    Universe {
        pool,
        borrower,
        guarantor_a,
        guarantor_b,
        borrower_ata,
        loan,
        amount,
    }
}

// Spec: nominal - partial repayment reduces outstanding and vault receives tokens.
//
// Given:
// - An active disbursed loan
// - Borrower ATA holds enough tokens to repay partially
//
// When:
// - Borrower calls `repay_loan` for less than the outstanding balance
//
// Then:
// - Loan `outstanding` decreases, status stays `Active`, vault balance increases
with_universe!(
    repay_loan_partial_nominal,
    universe(2_000_000_000),
    |app, u| {
        let vault_before = app.token_balance(&vault_pda(&u.pool).0).await;
        let repay = 500_000_000;
        let ix = app
            .repay_loan(&u.borrower, u.loan, u.borrower_ata, repay)
            .await;
        app.process(&[ix], &[&u.borrower]).await;

        let loan = app.fetch_loan(&u.loan).await;
        assert_eq!(loan.outstanding, u.amount - repay);
        assert_eq!(loan.status, LoanStatus::Active);
        assert_eq!(
            app.token_balance(&vault_pda(&u.pool).0).await,
            vault_before + repay
        );
        app.assert_vault_covers_liquid_savings(&u.pool).await;
    }
);

// Spec: nominal - full repayment clears borrower loan and guarantor obligations.
//
// Given:
// - An active loan with outstanding balance after a prior partial repayment
//
// When:
// - Borrower repays the remaining outstanding amount
//
// Then:
// - Loan status is `Repaid`, borrower `active_loan` is cleared, guarantors drop the guarantee
with_universe!(
    repay_loan_full_nominal,
    universe(2_000_000_000),
    |app, u| {
        let partial = 1_500_000_000;
        let ix_partial = app
            .repay_loan(&u.borrower, u.loan, u.borrower_ata, partial)
            .await;
        app.process(&[ix_partial], &[&u.borrower]).await;

        let final_repay = 500_000_000;
        let ix = app
            .repay_loan(&u.borrower, u.loan, u.borrower_ata, final_repay)
            .await;
        app.process(&[ix], &[&u.borrower]).await;

        let loan = app.fetch_loan(&u.loan).await;
        let (bob_member, _) = member_pda(&u.pool, &u.borrower.pubkey());
        let bob_member = app.fetch_member(&bob_member).await;
        let (carol_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
        let (dave_member, _) = member_pda(&u.pool, &u.guarantor_b.pubkey());
        let carol_member = app.fetch_member(&carol_member).await;
        let dave_member = app.fetch_member(&dave_member).await;

        assert_eq!(loan.outstanding, 0);
        assert_eq!(loan.status, LoanStatus::Repaid);
        assert!(bob_member.active_loan.is_none());
        assert_eq!(carol_member.active_guarantee_count, 0);
        assert_eq!(dave_member.active_guarantee_count, 0);
        assert_eq!(carol_member.locked_savings, 0);
        assert_eq!(dave_member.locked_savings, 0);
        assert_eq!(loan.guarantor_a_locked_savings, 0);
        assert_eq!(loan.guarantor_b_locked_savings, 0);
        app.assert_vault_covers_liquid_savings(&u.pool).await;
    }
);

// Spec: edge - repayment above outstanding (`RepaymentExceedsOutstanding`).
//
// Given:
// - An active loan with 2B outstanding
//
// When:
// - Borrower attempts to repay 3B
//
// Then:
// - Transaction fails with `RepaymentExceedsOutstanding`
with_universe!(
    repay_loan_fails_when_exceeds_outstanding,
    universe(2_000_000_000),
    |app, u| {
        let ix = app
            .repay_loan(&u.borrower, u.loan, u.borrower_ata, 3_000_000_000)
            .await;
        app.process_expect_custom_err(
            &[ix],
            &[&u.borrower],
            PoolError::RepaymentExceedsOutstanding,
        )
        .await;
    }
);

// Spec: edge - signer is not the loan borrower (`NotLoanBorrower`).
//
// Given:
// - An active loan owned by Bob
// - Eve is a pool member but not the borrower
//
// When:
// - Eve calls `repay_loan` on Bob's loan
//
// Then:
// - Transaction fails with `NotLoanBorrower`
with_universe!(
    repay_loan_fails_when_not_borrower,
    universe(2_000_000_000),
    |app, u| {
        let (eve, eve_ata) = member_with_savings(app, u.pool, 1_000_000_000).await;

        let ix = app.repay_loan(&eve, u.loan, eve_ata, 500_000_000).await;
        app.process_expect_custom_err(&[ix], &[&eve], PoolError::NotLoanBorrower)
            .await;
    }
);

// Spec: edge - loan already repaid; further repay rejected (`LoanNotActive`).
//
// Given:
// - A loan that has been fully repaid
//
// When:
// - Borrower calls `repay_loan` again
//
// Then:
// - Transaction fails with `LoanNotActive`
with_universe!(
    repay_loan_fails_when_already_repaid,
    universe(1_000_000_000),
    |app, u| {
        let ix_full = app
            .repay_loan(&u.borrower, u.loan, u.borrower_ata, u.amount)
            .await;
        app.process(&[ix_full], &[&u.borrower]).await;

        let ix = app
            .repay_loan(&u.borrower, u.loan, u.borrower_ata, 100_000_000)
            .await;
        app.process_expect_custom_err(&[ix], &[&u.borrower], PoolError::LoanNotActive)
            .await;
    }
);

// Spec: edge - zero repayment amount (`RepaymentAmountMustBePositive`).
//
// Given:
// - An active loan
//
// When:
// - Borrower calls `repay_loan` with amount `0`
//
// Then:
// - Transaction fails with `RepaymentAmountMustBePositive`
with_universe!(
    repay_loan_fails_on_zero_amount,
    universe(2_000_000_000),
    |app, u| {
        let ix = app.repay_loan(&u.borrower, u.loan, u.borrower_ata, 0).await;
        app.process_expect_custom_err(
            &[ix],
            &[&u.borrower],
            PoolError::RepaymentAmountMustBePositive,
        )
        .await;
    }
);

// Spec: edge - borrower ATA has insufficient SPL balance (token program error).
//
// Given:
// - An active loan with 2B outstanding
// - Borrower ATA holds less than the outstanding amount
//
// When:
// - Borrower calls `repay_loan` for the full outstanding amount
//
// Then:
// - Transaction fails at the SPL transfer
with_universe!(
    repay_loan_fails_on_insufficient_token_balance,
    universe(2_000_000_000),
    |app, u| {
        let sink = Keypair::new();
        app.airdrop_sol(&sink.pubkey(), LAMPORTS).await;
        let sink_ata = app.ensure_ata(&sink, 0).await;

        let balance = app.token_balance(&u.borrower_ata).await;
        let keep = 100_000_000;
        let transfer_ix = spl_token::instruction::transfer(
            &TOKEN_PROGRAM_ID,
            &u.borrower_ata,
            &sink_ata,
            &u.borrower.pubkey(),
            &[],
            balance.saturating_sub(keep),
        )
        .unwrap();
        app.process(&[transfer_ix], &[&u.borrower]).await;

        let ix = app
            .repay_loan(&u.borrower, u.loan, u.borrower_ata, u.amount)
            .await;
        app.process_expect_err(&[ix], &[&u.borrower]).await;
    }
);
