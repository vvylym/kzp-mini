# Integration Tests

Rust integration tests for `kzp-mini` using `solana-program-test`.

## Layout

Each `test_*.rs` file defines a documented `Universe` struct and `universe()` (or variant) that builds the shared fixture for that instruction. Every test uses regular `//` comments immediately above `with_universe!`:

- **Spec:** nominal or edge case label
- **Given / When / Then:** BDD scenario

Shared setup helpers live in `helpers/fixtures.rs` (`funded_admin`, `initialized_pool`, `member_with_savings`).

Specification and use cases: [../docs/USE_CASES.md](../docs/USE_CASES.md).

| File | Coverage |
|------|----------|
| `helpers/` | See `helpers/mod.rs` - split by responsibility (`app`, `transactions`, `tokens`, `accounts`, `instructions`, `fixtures`) |
| `test_initialize_pool.rs` | Pool + vault creation, name bounds, validation |
| `test_join_pool.rs` | Entry fee, duplicate member, invalid vault |
| `test_deposit_savings.rs` | Savings deposits, zero amount |
| `test_request_loan.rs` | Loan limits, borrower reservation, guarantor rules, nonce reuse |
| `test_cosign_loan.rs` | Partial/full disbursement, activation rechecks, co-sign errors |
| `test_repay_loan.rs` | Partial/full repay, borrower checks |
| `test_cancel_loan.rs` | Borrower cancels pending loan and clears reservations |
| `test_withdraw_cosign.rs` | Guarantor revokes partial co-sign |
| `test_settle_default.rs` | Admin default settlement, 50/50 guarantor split |
| `test_exit_pool.rs` | Exit nominal, pending/active loan and guarantee blocks |

## `with_universe!`

This macro declares the test function, applies `#[tokio::test]`, boots [`TestApp`](src/helpers/app.rs), and calls the module's `universe` setup. No `#[tokio::test]` or `async fn` wrapper needed.

```rust
// Spec: nominal - member exits the pool.
//
// Given:
// - A pool member with savings and no obligations
//
// When:
// - Member calls `exit_pool`
//
// Then:
// - Savings are returned and the member PDA is closed
with_universe!(exit_pool_nominal, |app, u| {
    let (carol, carol_ata) = member_with_savings(app, u.pool, 2_000_000_000).await;
    // ...
});

// Custom universe setup (function or args):
with_universe!(deposit_savings_nominal, universe(500_000_000), |app, u| { ... });
with_universe!(request_loan_fails_when_guarantor_limit_reached, universe_guarantor_at_limit, |app, u| { ... });
```

## Run

```bash
NO_DNA=1 anchor build --ignore-keys
cargo llvm-cov nextest --workspace --all-targets --all-features
# or: anchor test   # runs `cargo nextest --workspace` per Anchor.toml
```

BPF artifact path: `target/deploy/kzp_mini.so` (set via `BPF_OUT_DIR` in `TestApp`).

Shared dependency versions live in the root [`Cargo.toml`](../Cargo.toml) `[workspace.dependencies]` (`anchor-lang`, `anchor-spl`, `solana-sdk`, …).

## Error assertions

Custom program errors use Anchor codes: `6000 + PoolError as u32` (see `helpers/constants.rs`).
