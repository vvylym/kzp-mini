# Hardening Lifecycle Tracker

This tracker records each trustless-enforcement case from documentation through
test, implementation, verification, and commit. It is the source of truth for
incremental work on `hardening/trustless-enforcement`.

Status values:

- `planned` - documented, not implemented yet
- `red` - test added and verified failing against current behavior
- `green` - focused test passes after implementation
- `blocked` - case needs a prior implementation slice

| Case | Use case | Target behavior | Test file | Initial | Final | Commit |
|------|----------|-----------------|-----------|---------|-------|--------|
| H-001 | UC-1 | Pool names longer than the seed-safe maximum are rejected before account creation. | `programs/kzp-mini/src/operations/pool_ops.rs` | pass: `cargo test -p kzp-mini pool_name_validation` | green | `fix(pool): validate pool name length` |
| H-002 | UC-1 | Pool names exactly at the maximum accepted length initialize successfully. | `tests/src/test_initialize_pool.rs` | pass: `cargo test -p kzp-tests initialize_pool_accepts_max_length_name` | green | `fix(pool): validate pool name length` |
| H-003 | UC-2 | Member count increments use checked arithmetic. | `programs/kzp-mini/src/handlers/join_pool.rs` unit/integration coverage | code review | green | `fix(pool): validate pool name length` |
| H-004 | UC-3 | A borrower with a pending loan cannot request another loan with a different nonce. | `tests/src/test_request_loan.rs` | pass: `cargo test -p kzp-tests request_loan` | green | `fix(loans): reserve pending borrower capacity` |
| H-005 | UC-3 | Canceling a pending loan clears the borrower reservation and permits a new request. | `tests/src/test_request_loan.rs` / `tests/src/test_cancel_loan.rs` | pass: `cargo test -p kzp-tests cancel_loan` | green | `fix(loans): reserve pending borrower capacity` |
| H-006 | UC-4 | Activation cannot overwrite an existing borrower active loan. | `tests/src/test_cosign_loan.rs` | planned | planned | pending |
| H-007 | UC-4 | Guarantor eligibility is rechecked immediately before activation. | `tests/src/test_cosign_loan.rs` | pass: `cargo test -p kzp-tests co_sign_loan_fails_when_guarantor_now_has_active_loan` | green | `fix(loans): reserve pending borrower capacity` |
| H-008 | UC-4 | Guarantor savings liability is reserved before principal leaves the vault. | `tests/src/test_cosign_loan.rs` | pass: `cargo test -p kzp-tests co_sign_loan` | green | `fix(guarantees): reserve default liability` |
| H-009 | UC-4 | Insufficient vault liquidity on the second co-sign leaves loan and member state pending. | `tests/src/test_cosign_loan.rs` | planned | planned | pending |
| H-010 | UC-5 | Non-borrowers cannot cancel a pending loan. | `tests/src/test_cancel_loan.rs` | pass: `cargo test -p kzp-tests cancel_loan` | green | `fix(loans): reserve pending borrower capacity` |
| H-011 | UC-5 | Borrower can cancel an unsigned pending loan. | `tests/src/test_cancel_loan.rs` | pass: `cargo test -p kzp-tests cancel_loan` | green | `fix(loans): reserve pending borrower capacity` |
| H-012 | UC-6 | Co-sign withdrawal is rejected once the loan is active. | `tests/src/test_withdraw_cosign.rs` | planned | planned | pending |
| H-013 | UC-6 | Non-nominated members cannot withdraw a co-sign. | `tests/src/test_withdraw_cosign.rs` | planned | planned | pending |
| H-014 | UC-8 | Default settlement is rejected before the loan due date. | `tests/src/test_settle_default.rs` | planned | planned | pending |
| H-015 | UC-8 | Default settlement after due date uses reserved liability without guarantor signatures. | `tests/src/test_settle_default.rs` | planned | planned | pending |
| H-016 | UC-8 | Odd outstanding amounts split deterministically and reconcile exactly. | `tests/src/test_settle_default.rs` | planned | planned | pending |
| H-017 | UC-8 | Default settlement releases guarantee obligations and clears only the matching active loan. | `tests/src/test_settle_default.rs` | planned | planned | pending |
| H-018 | UC-9 | A member cannot exit while guarantor liability is reserved. | `tests/src/test_exit_pool.rs` | pass: `cargo test -p kzp-tests exit_pool_fails_with_active_guarantees` | green | `fix(guarantees): reserve default liability` |
| H-019 | UC-9 | Under-funded vault exit fails without mutating member or pool totals. | `tests/src/test_exit_pool.rs` | planned | planned | pending |
| H-020 | CLI | CLI refuses to deserialize pool or loan accounts not owned by this program. | CLI unit/integration coverage or manual command verification | planned | planned | pending |
| H-021 | UC-9 | A member with a pending borrower loan cannot exit the pool. | `tests/src/test_exit_pool.rs` | pass: `cargo test -p kzp-tests exit_pool_fails_with_pending_borrower_loan` | green | `fix(loans): reserve pending borrower capacity` |

## Incremental Commit Policy

Each green slice should be committed with a conventional commit message after
focused verification passes. Prefer one commit per coherent behavior, including
the related docs, tests, and implementation.

Do not stage `.idea/` files. Use explicit path staging for each commit.
