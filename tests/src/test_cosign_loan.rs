use crate::helpers::{TestApp, initialized_pool, member_with_savings};
use kzp_mini::error::PoolError;
use kzp_mini::state::LoanStatus;
use kzp_mini::utils::pda::{loan_pda, member_pda, vault_pda};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

/// Shared fixtures for [`co_sign_loan`](kzp_mini::co_sign_loan) integration tests.
struct Universe {
    pool: Pubkey,
    borrower: Keypair,
    guarantor_a: Keypair,
    guarantor_b: Keypair,
    borrower_ata: Pubkey,
}

impl Universe {
    async fn pending_loan(&self, app: &mut TestApp, loan_nonce: u64, amount: u64) -> Pubkey {
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

    async fn active_loan(&self, app: &mut TestApp, loan_nonce: u64, amount: u64) -> Pubkey {
        app.activate_loan(
            &self.borrower,
            self.pool,
            loan_nonce,
            amount,
            &self.guarantor_a,
            &self.guarantor_b,
            self.borrower_ata,
        )
        .await
    }
}

/// Initializes a pool with a borrower and two nominated guarantors (2B savings each).
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

// Spec: nominal - first guarantor signs (loan stays `Pending`), second triggers disbursement.
//
// Given:
// - A pending loan with borrower, guarantor A, and guarantor B as members
//
// When:
// - Guarantor A co-signs, then guarantor B co-signs
//
// Then:
// - After first sign: loan stays `Pending`, no guarantee obligation yet, no disbursement
// - After second sign: loan is `Active`, both guarantors carry the guarantee, principal is disbursed
with_universe!(
    co_sign_loan_nominal_partial_then_full_disbursement,
    |app, u| {
        let amount = 2_000_000_000;
        let loan_key = u.pending_loan(app, 42, amount).await;
        let bob_balance_before = app.token_balance(&u.borrower_ata).await;

        let ix_carol = app
            .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
            .await;
        app.process(&[ix_carol], &[&u.guarantor_a]).await;

        let loan = app.fetch_loan(&loan_key).await;
        assert!(loan.guarantor_a_signed);
        assert!(!loan.guarantor_b_signed);
        assert_eq!(loan.status, LoanStatus::Pending);

        let (carol_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
        let carol_member_state = app.fetch_member(&carol_member).await;
        assert_eq!(carol_member_state.active_guarantee_count, 0);
        assert_eq!(carol_member_state.pending_guarantee_count, 1);
        assert_eq!(app.token_balance(&u.borrower_ata).await, bob_balance_before);

        let ix_dave = app
            .co_sign_loan(&u.guarantor_b, loan_key, u.borrower_ata)
            .await;
        app.process(&[ix_dave], &[&u.guarantor_b]).await;

        let loan = app.fetch_loan(&loan_key).await;
        assert!(loan.guarantor_a_signed);
        assert!(loan.guarantor_b_signed);
        assert_eq!(loan.status, LoanStatus::Active);
        assert_eq!(loan.outstanding, amount);
        assert_eq!(
            app.token_balance(&u.borrower_ata).await,
            bob_balance_before + amount
        );

        let (bob_member, _) = member_pda(&u.pool, &u.borrower.pubkey());
        let bob_member_state = app.fetch_member(&bob_member).await;
        assert_eq!(bob_member_state.active_loan, Some(loan_key));
        assert!(bob_member_state.pending_loan.is_none());

        let (dave_member, _) = member_pda(&u.pool, &u.guarantor_b.pubkey());
        let carol_after = app.fetch_member(&carol_member).await;
        let dave_after = app.fetch_member(&dave_member).await;
        assert_eq!(carol_after.pending_guarantee_count, 0);
        assert_eq!(dave_after.pending_guarantee_count, 0);
        assert_eq!(carol_after.active_guarantee_count, 1);
        assert_eq!(dave_after.active_guarantee_count, 1);
        assert_eq!(carol_after.locked_savings, amount / 2);
        assert_eq!(dave_after.locked_savings, amount / 2);
        assert_eq!(loan.guarantor_a_locked_savings, amount / 2);
        assert_eq!(loan.guarantor_b_locked_savings, amount / 2);
        app.assert_vault_covers_liquid_savings(&u.pool).await;
    }
);

// Spec: edge - signer is not one of the two nominated guarantors (`NotNominatedGuarantor`).
//
// Given:
// - A pending loan with guarantors Carol and Dave
// - Eve is a pool member but not nominated on this loan
//
// When:
// - Eve calls `co_sign_loan`
//
// Then:
// - Transaction fails with `NotNominatedGuarantor`
with_universe!(co_sign_loan_fails_when_not_nominated_guarantor, |app, u| {
    let (eve, _) = member_with_savings(app, u.pool, 2_000_000_000).await;

    let loan_key = u.pending_loan(app, 7, 1_000_000_000).await;

    let ix_eve = app.co_sign_loan(&eve, loan_key, u.borrower_ata).await;
    app.process_expect_custom_err(&[ix_eve], &[&eve], PoolError::NotNominatedGuarantor)
        .await;
});

// Spec: edge - guarantor eligibility is rechecked before activation.
//
// Given:
// - Guarantor A was eligible when the borrower requested a loan
// - Guarantor B has already co-signed
// - Guarantor A becomes an active borrower before their co-sign
//
// When:
// - Guarantor A tries to provide the second co-sign
//
// Then:
// - Activation fails with `GuarantorHasActiveLoan`
with_universe!(
    co_sign_loan_fails_when_guarantor_now_has_active_loan,
    |app, u| {
        let loan_key = u.pending_loan(app, 10, 500_000_000).await;

        let ix_b = app
            .co_sign_loan(&u.guarantor_b, loan_key, u.borrower_ata)
            .await;
        app.process(&[ix_b], &[&u.guarantor_b]).await;

        let guarantor_a_ata = app.ata_for(&u.guarantor_a.pubkey());
        let (other_guarantor_a, _) = member_with_savings(app, u.pool, 2_000_000_000).await;
        let (other_guarantor_b, _) = member_with_savings(app, u.pool, 2_000_000_000).await;
        app.activate_loan(
            &u.guarantor_a,
            u.pool,
            11,
            500_000_000,
            &other_guarantor_a,
            &other_guarantor_b,
            guarantor_a_ata,
        )
        .await;

        let ix_a = app
            .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
            .await;
        app.process_expect_custom_err(
            &[ix_a],
            &[&u.guarantor_a],
            PoolError::GuarantorHasActiveLoan,
        )
        .await;
    }
);

// Spec: edge - guarantor must have enough unlocked savings for their reserved share.
//
// Given:
// - Guarantor A has less unlocked savings than their 50/50 share
// - Guarantor B already co-signed
//
// When:
// - Guarantor A tries to provide the second co-sign
//
// Then:
// - Activation fails with `GuarantorInsufficientSavings`
with_universe!(
    co_sign_loan_fails_when_guarantor_lacks_unlocked_savings_for_share,
    |app, u| {
        let (low_savings_guarantor, _) = member_with_savings(app, u.pool, 100_000_000).await;
        let loan_nonce = 12;
        let ix = app.request_loan(
            &u.borrower,
            u.pool,
            loan_nonce,
            1_000_000_000,
            low_savings_guarantor.pubkey(),
            u.guarantor_b.pubkey(),
        );
        app.process(&[ix], &[&u.borrower]).await;
        let loan_key = loan_pda(&u.pool, &u.borrower.pubkey(), loan_nonce).0;

        let ix_b = app
            .co_sign_loan(&u.guarantor_b, loan_key, u.borrower_ata)
            .await;
        app.process(&[ix_b], &[&u.guarantor_b]).await;

        let borrower_balance_before = app.token_balance(&u.borrower_ata).await;
        let ix_a = app
            .co_sign_loan(&low_savings_guarantor, loan_key, u.borrower_ata)
            .await;
        app.process_expect_custom_err(
            &[ix_a],
            &[&low_savings_guarantor],
            PoolError::GuarantorInsufficientSavings,
        )
        .await;

        let loan = app.fetch_loan(&loan_key).await;
        let (low_member, _) = member_pda(&u.pool, &low_savings_guarantor.pubkey());
        let low_member = app.fetch_member(&low_member).await;
        assert_eq!(loan.status, LoanStatus::Pending);
        assert_eq!(low_member.locked_savings, 0);
        assert_eq!(
            app.token_balance(&u.borrower_ata).await,
            borrower_balance_before
        );
    }
);

// Spec: edge - the second co-sign must include activation accounts.
//
// Given:
// - Guarantor A already provided the first co-sign
//
// When:
// - Guarantor B provides the second co-sign with only the base partial account set
//
// Then:
// - Transaction fails before activation with `MissingActivationAccounts`
with_universe!(
    co_sign_loan_fails_when_activation_accounts_missing,
    |app, u| {
        let loan_key = u.pending_loan(app, 13, 1_000_000_000).await;
        let ix_a = app
            .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
            .await;
        app.process(&[ix_a], &[&u.guarantor_a]).await;

        let ix_b = app.co_sign_loan_base_only(&u.guarantor_b, loan_key).await;
        app.process_expect_custom_err(
            &[ix_b],
            &[&u.guarantor_b],
            PoolError::MissingActivationAccounts,
        )
        .await;
    }
);

// Spec: edge - activation cannot overwrite an existing borrower active loan.
//
// Given:
// - A pending loan where guarantor B has co-signed
// - Borrower's member account has drifted to contain an active loan
//
// When:
// - Guarantor A tries to provide the second co-sign
//
// Then:
// - Activation fails with `ExistingActiveLoan`
with_universe!(
    co_sign_loan_fails_when_borrower_active_slot_changed,
    |app, u| {
        let loan_key = u.pending_loan(app, 13, 500_000_000).await;

        let ix_b = app
            .co_sign_loan(&u.guarantor_b, loan_key, u.borrower_ata)
            .await;
        app.process(&[ix_b], &[&u.guarantor_b]).await;

        let (borrower_member, _) = member_pda(&u.pool, &u.borrower.pubkey());
        let mut borrower_state = app.fetch_member(&borrower_member).await;
        borrower_state.active_loan = Some(Pubkey::new_unique());
        app.overwrite_member(&borrower_member, &borrower_state)
            .await;

        let borrower_balance_before = app.token_balance(&u.borrower_ata).await;
        let ix_a = app
            .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
            .await;
        app.process_expect_custom_err(&[ix_a], &[&u.guarantor_a], PoolError::ExistingActiveLoan)
            .await;

        let loan = app.fetch_loan(&loan_key).await;
        assert_eq!(loan.status, LoanStatus::Pending);
        assert_eq!(
            app.token_balance(&u.borrower_ata).await,
            borrower_balance_before
        );
    }
);

// Spec: edge - insufficient vault liquidity leaves pending state unchanged.
//
// Given:
// - A pending loan where guarantor A has co-signed
// - The vault SPL balance is lower than the loan principal
//
// When:
// - Guarantor B tries to provide the second co-sign
//
// Then:
// - Activation fails with `InsufficientVaultLiquidity` and no disbursement or liability lock happens
with_universe!(
    co_sign_loan_fails_on_insufficient_vault_liquidity,
    |app, u| {
        let amount = 1_000_000_000;
        let loan_key = u.pending_loan(app, 14, amount).await;

        let ix_a = app
            .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
            .await;
        app.process(&[ix_a], &[&u.guarantor_a]).await;

        let (vault, _) = vault_pda(&u.pool);
        app.overwrite_token_amount(&vault, amount - 1).await;
        let borrower_balance_before = app.token_balance(&u.borrower_ata).await;

        let ix_b = app
            .co_sign_loan(&u.guarantor_b, loan_key, u.borrower_ata)
            .await;
        app.process_expect_custom_err(
            &[ix_b],
            &[&u.guarantor_b],
            PoolError::InsufficientVaultLiquidity,
        )
        .await;

        let loan = app.fetch_loan(&loan_key).await;
        let (borrower_member, _) = member_pda(&u.pool, &u.borrower.pubkey());
        let borrower_member = app.fetch_member(&borrower_member).await;
        let (ga_member, _) = member_pda(&u.pool, &u.guarantor_a.pubkey());
        let (gb_member, _) = member_pda(&u.pool, &u.guarantor_b.pubkey());
        let ga_member = app.fetch_member(&ga_member).await;
        let gb_member = app.fetch_member(&gb_member).await;

        assert_eq!(loan.status, LoanStatus::Pending);
        assert!(loan.guarantor_a_signed);
        assert!(!loan.guarantor_b_signed);
        assert_eq!(borrower_member.pending_loan, Some(loan_key));
        assert!(borrower_member.active_loan.is_none());
        assert_eq!(ga_member.locked_savings, 0);
        assert_eq!(gb_member.locked_savings, 0);
        assert_eq!(
            app.token_balance(&u.borrower_ata).await,
            borrower_balance_before
        );
    }
);

// Spec: edge - guarantor attempts to co-sign twice (`AlreadyCoSigned`).
//
// Given:
// - A pending loan where guarantor A has already co-signed
//
// When:
// - Guarantor A calls `co_sign_loan` again
//
// Then:
// - Transaction fails with `AlreadyCoSigned`
with_universe!(co_sign_loan_fails_when_already_co_signed, |app, u| {
    let loan_key = u.pending_loan(app, 8, 1_000_000_000).await;

    let ix_carol = app
        .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
        .await;
    app.process(&[ix_carol], &[&u.guarantor_a]).await;

    let ix_dup = app
        .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
        .await;
    app.process_expect_custom_err(&[ix_dup], &[&u.guarantor_a], PoolError::AlreadyCoSigned)
        .await;
});

// Spec: edge - loan already `Active`; further co-sign rejected (`LoanNotPending`).
//
// Given:
// - A fully co-signed and disbursed active loan
//
// When:
// - A guarantor calls `co_sign_loan` again
//
// Then:
// - Transaction fails with `LoanNotPending`
with_universe!(co_sign_loan_fails_when_loan_not_pending, |app, u| {
    let loan_key = u.active_loan(app, 9, 1_000_000_000).await;

    let ix = app
        .co_sign_loan(&u.guarantor_a, loan_key, u.borrower_ata)
        .await;
    app.process_expect_custom_err(&[ix], &[&u.guarantor_a], PoolError::LoanNotPending)
        .await;
});
