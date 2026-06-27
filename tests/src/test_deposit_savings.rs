use crate::helpers::{TestApp, initialized_pool};
use kzp_mini::error::PoolError;
use kzp_mini::utils::pda::member_pda;
use solana_sdk::{pubkey::Pubkey, signature::Signer};

/// Shared fixtures for [`deposit_savings`](kzp_mini::deposit_savings) integration tests.
struct Universe {
    pool: Pubkey,
    member: solana_sdk::signer::keypair::Keypair,
    member_ata: Pubkey,
}

/// Initializes a pool with one joined member; `ata_balance` funds the member ATA.
async fn universe(app: &mut TestApp, ata_balance: u64) -> Universe {
    let (_admin, pool) = initialized_pool(app).await;
    let (member, member_ata) = app.create_funded_user(ata_balance).await;
    app.join_member(&member, pool, member_ata).await;
    Universe {
        pool,
        member,
        member_ata,
    }
}

// Spec: nominal - deposit increases member savings and pool `total_savings`.
//
// Given:
// - A pool member with zero savings balance
// - Sufficient tokens in the member ATA
//
// When:
// - Member calls `deposit_savings`
//
// Then:
// - Member `savings_balance` and pool `total_savings` increase by the deposit amount
with_universe!(deposit_savings_nominal, universe(500_000_000), |app, u| {
    app.deposit(&u.member, u.pool, u.member_ata, 200_000_000)
        .await;

    let (member_key, _) = member_pda(&u.pool, &u.member.pubkey());
    let member = app.fetch_member(&member_key).await;
    let pool_state = app.fetch_pool(&u.pool).await;

    assert_eq!(member.savings_balance, 200_000_000);
    assert_eq!(pool_state.total_savings, 200_000_000);
    app.assert_vault_covers_liquid_savings(&u.pool).await;
});

// Spec: nominal - repeated deposits accumulate savings balance.
//
// Given:
// - A pool member with enough ATA balance for multiple deposits
//
// When:
// - Member calls `deposit_savings` twelve times with the same amount
//
// Then:
// - Member `savings_balance` equals the sum of all deposits
with_universe!(
    deposit_savings_repeated_increases_balance,
    universe(3_000_000_000),
    |app, u| {
        for _ in 0..12 {
            app.deposit(&u.member, u.pool, u.member_ata, 200_000_000)
                .await;
        }

        let (member_key, _) = member_pda(&u.pool, &u.member.pubkey());
        let member = app.fetch_member(&member_key).await;
        assert_eq!(member.savings_balance, 2_400_000_000);
    }
);

// Spec: edge - zero deposit amount (`DepositAmountMustBePositive`).
//
// Given:
// - A pool member
//
// When:
// - Member calls `deposit_savings` with amount `0`
//
// Then:
// - Transaction fails with `DepositAmountMustBePositive`
with_universe!(
    deposit_savings_fails_on_zero_amount,
    universe(500_000_000),
    |app, u| {
        let ix = app.deposit_savings(&u.member, u.pool, u.member_ata, 0);
        app.process_expect_custom_err(&[ix], &[&u.member], PoolError::DepositAmountMustBePositive)
            .await;
    }
);

// Spec: edge - member ATA balance too low for deposit (SPL transfer failure).
//
// Given:
// - A pool member whose ATA cannot cover the requested deposit
//
// When:
// - Member calls `deposit_savings` for more than their ATA balance
//
// Then:
// - Transaction fails at the SPL transfer
with_universe!(
    deposit_savings_fails_on_insufficient_funds,
    universe(100_000_000),
    |app, u| {
        let ix = app.deposit_savings(&u.member, u.pool, u.member_ata, 1_000_000_000);
        app.process_expect_err(&[ix], &[&u.member]).await;
    }
);
