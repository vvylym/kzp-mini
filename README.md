# KZP Mini

KZP Mini is a Solana implementation of a workplace mutual-aid fund (*Kasa Zapomogowa Pracownicza*).

Members:

- join a shared pool
- deposit savings
- request loans with two guarantors
- repay obligations
- withdraw savings and leave

The protocol enforces financial mechanics on-chain while allowing membership policy and governance to remain off-chain. Built in Rust using [Anchor](https://www.anchor-lang.com/).

> **Superteam Poland submission** - entry for [Build everyday real-world systems as on-chain Rust programs](https://superteam.fun/earn/listing/build-everyday-real-world-systems-as-on-chain-rust-programs) on [Superteam Earn](https://superteam.fun/earn).

**Repository:** [github.com/vvylym/kzp-mini](https://github.com/vvylym/kzp-mini)

## At a glance

| Property | Value |
|----------|-------|
| Blockchain | Solana (devnet deployed) |
| Language | Rust stable for host builds |
| Framework | Anchor crates 1.1.2 |
| Solana crates | `solana-client` 4.1.0 · `solana-program-test` 4.1.0 · `solana-sdk` 4.0.1 |
| Loan model | Two guarantors, 3× savings cap |
| Savings model | Shared SPL vault + ledger |
| Client | `kzp` CLI |
| License | MIT |

---

## Why KZP?

Traditional workplace savings associations rely on spreadsheets, manual bookkeeping, paper signatures, and committee decisions. That creates opaque balances, delayed settlement, manual verification, and dispute risk.

KZP Mini replaces **administrative coordination** with **on-chain enforcement**.

| Concern | Traditional handling | Friction |
|---------|---------------------|----------|
| Membership | HR roster, payroll deductions | Manual roster upkeep |
| Savings | Spreadsheet per member | Opaque balances |
| Loans | Committee + verbal guarantees | Signatures disconnected from money |
| Default | Social pressure, delayed settlement | No shared rule engine |
| Exit | Manual “all clear” check | Error-prone |

| Traditional concept | On-chain implementation |
|---------------------|-------------------------|
| Pool / cash box | **Pool** + **Vault** PDAs |
| Member record | **Member** PDA |
| Loan | **Loan** PDA (`Pending` → `Active` → closed on repay/default) |
| Two guarantors | Both must co-sign before disbursement |
| Default | Permissionless `settle_default` after due date; 50/50 reserved guarantor liability |

Deeper domain and mapping: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Key features

### Savings

- One-time entry fee on join
- Member deposits to shared vault
- Per-member savings ledger

### Lending

- Loan requests with two nominated guarantors
- Partial then full co-sign
- Automatic disbursement when both guarantors approve

### Risk controls

- Loan cap at 3× member savings
- One unresolved loan per borrower
- Up to five guarantees per member
- Exit blocked while borrowing or guaranteeing

### Administration

- Pool initialization and SPL mint binding
- Permissionless due-date default settlement from reserved guarantor savings liability

---

## Live devnet demo

**Program ID:** `GsjUnBFvYtcxNwCrydPUjQTngGTqdx5v7APnWahnqwkx`

The same program ID is preserved across releases. The original bounty submission remains available as [`release/0.1.0`](https://github.com/vvylym/kzp-mini/tree/release/0.1.0) and [`v0.1.0`](https://github.com/vvylym/kzp-mini/releases/tag/v0.1.0). The current hardened revision is [`release/0.2.0`](https://github.com/vvylym/kzp-mini/tree/release/0.2.0) / [`v0.2.0`](https://github.com/vvylym/kzp-mini/releases/tag/v0.2.0).

### 0.2.0 verified lifecycle

- [x] Upgrade program in place
- [x] Initialize pool
- [x] Join members (admin + two guarantors)
- [x] Deposit borrower savings and guarantor backing
- [x] Request loan
- [x] Co-sign (guarantor A, then B)
- [x] Disburse principal
- [x] Repay loan and close terminal loan account
- [x] Exit pool

| Step | Transaction |
|------|-------------|
| Program upgrade | [explorer](https://explorer.solana.com/tx/34Cgj2Fc9aE3Jh8br8aMBCvxoeSS5NrFi2gutKPVjRS6kAQvpmoTASJCKnkeAvT6rxdcLgx2LgakYUUs9QbbkYpo?cluster=devnet) |
| Initialize pool | [explorer](https://explorer.solana.com/tx/5kVVwFn3biEUX4h22pmTi3znSmcKqDJUNemws2YRB2iZiq3bC2p8YpVcreJ9ikYbsq54zFvdrBMJeN3zAXt3ZHNA?cluster=devnet) |
| Admin join | [explorer](https://explorer.solana.com/tx/5JSVyJrQEkQa8ideCMmPSwBS7XveqWYaEKSf1bw2co8CYKYL8AtUp99xxpVUHpb9ZsznPdyAP8iEtXTRJMAAzg9e?cluster=devnet) |
| Guarantor A join | [explorer](https://explorer.solana.com/tx/24JpScXb4kPdPVCeZExD5ZtEhi8xoomUq2J9xmSrVNWgFf15PKNKXMEcLxuZW62Z1Mo4vgmGy9b8pu5kWD6R4wkV?cluster=devnet) |
| Guarantor B join | [explorer](https://explorer.solana.com/tx/41tNYYbQFrNmg4biVcRHoyvEfcUsgAMc1b7ziczrrR1cTWXSd81J8jfDnbtD4ifaJoaKwbacLk5Sfdw6zFT3XVca?cluster=devnet) |
| Admin deposit savings | [explorer](https://explorer.solana.com/tx/4kTdSiZFVnM6VfyGtoLwohAhX789XEutkXi8DnJpQL3sw7ECk2dh6yZpDqZW1UJx2VE4a4BLNyJFnmzX8UPDqsb7?cluster=devnet) |
| Guarantor A deposit savings | [explorer](https://explorer.solana.com/tx/63mfzdGYG8Q8J5hMX6AC2hVjs6qPMcdH2mKt3KCbjVCJoQbfVFzuaXgWAHEBcjqnpm5edW3SJnoQyCJrLqzPJyTN?cluster=devnet) |
| Guarantor B deposit savings | [explorer](https://explorer.solana.com/tx/3ZX3oPbQj3dU2H67TcF74TWC2ogjcNE86kL6s36mA4tdEDQYfn9KxAEK26MhE4NYxdeTQVpx6k6mvvynhfyWbd2V?cluster=devnet) |
| Request loan | [explorer](https://explorer.solana.com/tx/2yeGsfQ3i53SBbqtTfFtJMZnvDvore7uXMh5Uzv7yRjivrxJVWn39yK2USDqmfYjPwF32vgmcJzZXbPRhUnZFtbV?cluster=devnet) |
| Co-sign (guarantor A) | [explorer](https://explorer.solana.com/tx/9MgM6uizxiyLb7kii5KCV6dPSJ51rLKoBLsE8VLMtBs1xAh5C4XsGijNxNutSFbKLXtqgSbdhEXLB8vic8hRkK9?cluster=devnet) |
| Co-sign + disburse (B) | [explorer](https://explorer.solana.com/tx/GoQSDNipyjagWWNEc9GEjuzkMHVJdazJ5c9jdYxibNYZ6U6ob1MYPZZBety9SD4DgRoLCCRk265YnSgbAZt6UY6?cluster=devnet) |
| Repay loan | [explorer](https://explorer.solana.com/tx/3ScT8xMCXYrVpQEa5pZ55dbViuHiSLTpNM62DMq8s3TBZUXVypJQ9yarWGZFayPxAoe8zHX8paeN512vhB2BQHVU?cluster=devnet) |
| Exit pool | [explorer](https://explorer.solana.com/tx/346qUXqW2Rja64FFvv61sG1e8RjcRB4CZyXctT2MF5UNbosFXRaxWcoX35sHdATSDoMyVoayLDAFZZmd8oNW4wvC?cluster=devnet) |

**0.2.0 demo accounts:** pool `5CFCSvhNdCdGYCAZYcP7taA87vAmKsSLtiHPP8TBWeng` · loan `2LRcvkpTdkNQn5gVpJPEfgj4imkj8bTm5JeEHGvHKZPP` · mint `8GKf5BwMfFEZmkoGYuH4vNwGy1qgdxpSesq6jGp55J7a`

```bash
bash scripts/devnet-demo.sh   # full scripted lifecycle
```

---

## Quick start

### Install

Rust stable, Solana CLI 4.x, Anchor CLI 1.1.2, Anchor crates 1.1.2, Solana crates 4.x, cargo-nextest. Full toolchain notes: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).

### Build

```bash
git clone https://github.com/vvylym/kzp-mini.git && cd kzp-mini
mkdir -p target/deploy && cp keys/program.json target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor build --ignore-keys
cargo build -p kzp-cli --release
```

Keypairs are local-only - see [keys/README.md](keys/README.md).

### Deploy

```bash
bash scripts/deploy.sh devnet
```

### Verify

```bash
cargo run -p kzp-cli --release -- config
```

---

## CLI usage

Commands grouped by role. Full reference: [docs/CLI.md](docs/CLI.md).

| Role | Commands |
|------|----------|
| **Pool setup / cranks** | `pool initialize` · permissionless `loan settle-default` |
| **Members** | `pool join` · `pool deposit` · `pool exit` |
| **Borrowers** | `loan request` · `loan repay` · `loan cancel` |
| **Guarantors** | `loan cosign` · `loan withdraw-cosign` |

```bash
kzp pool initialize --name "My Pool" --entry-fee 50 --mint <MINT>
kzp pool join --pool <POOL> --entry-fee 50
kzp pool deposit --pool <POOL> --amount 1000000
kzp loan request --pool <POOL> --nonce 1 --amount 500000 \
  --guarantor-a <PUBKEY_A> --guarantor-b <PUBKEY_B>
kzp loan cosign --loan <LOAN>
kzp loan repay --loan <LOAN> --amount 500000
kzp pool exit --pool <POOL>
```

Global flags: `--cluster`, `--rpc-url`, `--wallet`, `--dry-run`.

---

## System architecture

```
                    Pool PDA
                        │
                        ▼
                   Vault PDA
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
   Member PDA      Member PDA      Member PDA
        │
        ▼
    Loan PDA
```

| Account | Purpose |
|---------|---------|
| **Pool** | Admin, mint, counters |
| **Vault** | SPL token custody |
| **Member** | Savings, loan/guarantee obligations |
| **Loan** | Principal, guarantors, status |

**Instructions (10):** `initialize_pool` · `join_pool` · `deposit_savings` · `request_loan` · `co_sign_loan` · `repay_loan` · `cancel_loan` · `withdraw_cosign` · `settle_default` · `exit_pool`

Details: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md) · [docs/STATE.md](docs/STATE.md)

---

## Financial model

Two parallel views:

1. **SPL vault** - actual token balance
2. **Ledger** - `pool.total_savings` and `member.savings_balance`

Entry fees fund the vault but not member savings (liquidity buffer). Disburse and exit are **fail-closed** if vault balance is insufficient.

Full model: [docs/ACCOUNTING.md](docs/ACCOUNTING.md).

---

## Loan lifecycle

```
Pending → Active → full repay (loan account closed)
                 ↘ due default (loan account closed)
Pending → cancel_loan
```

| State | Guarantor tracking | Escape hatches |
|-------|-------------------|----------------|
| Pending | `pending_guarantee_count` | Borrower: `cancel_loan` · Guarantor: `withdraw_cosign` |
| Active | `active_guarantee_count` + loan-local backing | Repay or permissionless `settle_default` after due date |
| Closed | Counts and backing cleared | Guarantors may exit when obligations are cleared |

---

## Security & trust model

### Enforced on-chain

- Membership and savings state
- Loan limits and guarantor rules
- Co-sign before disbursement
- Vault liquidity checks
- Arithmetic safety (`checked_add` / `checked_sub`)

### Off-chain / trusted

- Who may join the pool (workplace policy)
- Whether a due loan should be cranked as defaulted (off-chain policy)
- SPL mint legitimacy at pool creation

Full threat model: [docs/SECURITY.md](docs/SECURITY.md).

---

## Project structure

```
programs/kzp-mini/src/
├── instructions/   Anchor account constraints + one handle per instruction
├── operations/     Pure business rules (unit-tested)
├── state.rs        Pool, Member, Loan
└── utils/          PDA helpers

cli/                kzp command-line client
tests/              solana-program-test integration suite
docs/               Architecture, CLI, accounting, security
scripts/            ci.sh, deploy.sh, devnet-demo.sh
keys/               Local keypairs (gitignored)
```

**Philosophy:** `instructions` = account validation and orchestration · `operations` = testable rules without Anchor contexts.

---

## Testing

| Layer | Coverage |
|-------|----------|
| Unit tests | `programs/kzp-mini/src/operations/` |
| Integration | 62 `solana-program-test` scenarios |
| Specs | BDD criteria in [docs/USE_CASES.md](docs/USE_CASES.md) |

```bash
bash scripts/ci.sh
```

Conventions: [tests/README.md](tests/README.md).

---

## Documentation

| Document | Audience | Purpose |
|----------|----------|---------|
| [README.md](README.md) | Everyone | Overview and navigation |
| [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md) | Developers | Install, deploy, devnet setup |
| [docs/CLI.md](docs/CLI.md) | Operators | Full CLI by role |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Reviewers | Accounts, instructions, mapping |
| [docs/ACCOUNTING.md](docs/ACCOUNTING.md) | Auditors | Ledger vs vault, limits |
| [docs/SECURITY.md](docs/SECURITY.md) | Auditors | Threat model |
| [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md) | Integrators | Instruction account metas |
| [docs/STATE.md](docs/STATE.md) | Integrators | Account layouts |
| [docs/USE_CASES.md](docs/USE_CASES.md) | Contributors | Acceptance criteria |
| [docs/ERRORS.md](docs/ERRORS.md) | Integrators | Error codes |

Index: [docs/README.md](docs/README.md).

---

## Limitations & tradeoffs

| Topic | Choice | Implication |
|-------|--------|-------------|
| Policy vs code | Eligibility/default policy off-chain | Any signer can `settle_default` after `due_ts`; off-chain policy decides when to crank |
| Single admin | One `pool.admin` | No multisig in this release |
| Default split | 50/50 (`div_ceil` on odd amounts) | First guarantor pays extra token on odd sums |
| Loan cap | 3× savings, 5 guarantees | May block edge cases |
| Ledger vs vault | Both tracked; fail-closed exit/disburse | Safer; monitor for drift |
| Pending loans | Only borrower can cancel | Abandoned PDAs pay rent until cancelled |
| Upgrades | No migration instruction | Schema changes need fresh pool deploy |
| Toolchain | Stable host Rust, Anchor crates 1.1.2, Solana crates 4.x | SBF builds still use Solana's bundled compiler; do not set a workspace `rust-version` above that compiler's support |

---

## Roadmap

### Planned

- Multisig pool administration
- On-chain governance voting for defaults
- Accrued yield on member savings (pool-level interest distribution)
- Dynamic guarantee models (e.g. variable split)
- Account migration support

---

## Contributing

```bash
bash scripts/ci.sh
```

Before opening a PR: `cargo fmt`, `cargo clippy`, `cargo nextest`, `anchor build`. See [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md) for toolchain details.

---

## License

MIT
