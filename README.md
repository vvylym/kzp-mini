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
| Language | Rust 1.89 |
| Framework | Anchor 1.0.2 |
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
| Loan | **Loan** PDA (`Pending` → `Active` → `Repaid` / `Defaulted`) |
| Two guarantors | Both must co-sign before disbursement |
| Default | Admin `settle_default`; 50/50 guarantor liability |

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
- One active loan per borrower
- Up to five guarantees per member
- Exit blocked while borrowing or guaranteeing

### Administration

- Pool initialization and SPL mint binding
- Default settlement with guarantor SPL recovery

---

## Live devnet demo

**Program ID:** `GsjUnBFvYtcxNwCrydPUjQTngGTqdx5v7APnWahnqwkx`

### Verified lifecycle

- [x] Deploy program
- [x] Initialize pool
- [x] Join members (admin + two guarantors)
- [x] Deposit savings
- [x] Request loan
- [x] Co-sign (guarantor A, then B)
- [x] Disburse principal
- [x] Repay loan
- [x] Exit pool

| Step | Transaction |
|------|-------------|
| Program deploy | [explorer](https://explorer.solana.com/tx/2yxGQoDawUuxBkDHZ9ujbEDfDHxEwUa7y984oPLZvUyoYBQyzmcdvYSsdeCCFGhANXxLoKdSCukJ53Y9dVonPrrF?cluster=devnet) |
| Initialize pool | [explorer](https://explorer.solana.com/tx/5PZoFBeMyAcghwxe4BsZ8JTb46Vc1THNCkH8KnPtyENuUfp3qncXUmoGf1NNhATzYxQ3BRJaoAENdkb645vUUfEr?cluster=devnet) |
| Admin join | [explorer](https://explorer.solana.com/tx/4rLcE1oqSymJrWNXPf9GHdBJieun7s6GvHcJ8xf1zUaUXibGcMihnSq7VDqrUQ2EBrq27TFA7sMSnSu5CmhnTkes?cluster=devnet) |
| Guarantor A join | [explorer](https://explorer.solana.com/tx/37kNGnfavU2CjZafgrgRuQbNBR62SmrgxqcfBBkpa83tasi93QMR26yJi9go2rVBdTn41xcGpFrSLtmEBwiaL1kd?cluster=devnet) |
| Guarantor B join | [explorer](https://explorer.solana.com/tx/4e28PX7kBYvLq8GqT6MtdZb46VF9g6J11kJUKnQWW2vdPp9suW3bScLtv5AnhzrAADUoCZe2tSXDYhtkFWyMMAAU?cluster=devnet) |
| Deposit savings | [explorer](https://explorer.solana.com/tx/nEVmd3sxt3GPDCMp5LKMzSbEdijHLrBrfivDq298NkxbPMKm9sDtSU6oddrZe2Jjjeq5wA9PxQKKJcC2XDKymeT?cluster=devnet) |
| Request loan | [explorer](https://explorer.solana.com/tx/2U4Xj4Z96Q9R2fuwnJgj2YGGi6rAWq8CZ99PAEniRwC9ms3witedtVH4XamszXaMYaTnF4rzYnS4fYotQRkcaSpc?cluster=devnet) |
| Co-sign (guarantor A) | [explorer](https://explorer.solana.com/tx/487GaUXDizbkQx189kQ9JerCxBKhJDiR82WxsVnNBV6h35yJKBmyPcEqa2KLmgPTwNp4dYTqDEbjHte3sxxwXYBs?cluster=devnet) |
| Co-sign + disburse (B) | [explorer](https://explorer.solana.com/tx/5d9rpicU6iWgEbQDYb6oVDemv4Vd6fNnqCF9Q3hEDKx1kiMp8XR7Zx2Edn59eDTerMgabcaMR2H7UK8f58q5s3Ni?cluster=devnet) |
| Repay loan | [explorer](https://explorer.solana.com/tx/3dHLmTDjhWkPsp3ruWUGqkLea5aazYgHn7siPUSHwycemdeiiwE7LJxP2o9ieRdQisCTdzsWSuGxCfKPPen6Hizp?cluster=devnet) |
| Exit pool | [explorer](https://explorer.solana.com/tx/4yyvHNjit8M4gEwpWzXBJUAPeyEAVUXsAz4eZsrcLNEHU8CsX5rN8kWQcC8qXYh4TJkpwkuAomhmCr68UYmkG9AV?cluster=devnet) |

**Demo accounts:** pool `4nZf8sPfRvKUucN43FirPQ3T3JvvkCo4LFukGYCRL7tg` · loan `toqwDKpVPAdHPkLSJWyh5yLdDTf7szVEKPRtVNek4bL` · mint `9CsRiEPkagzfTLTDhsBhuvfHNXupBugtHzy1wPefTi4V`

```bash
bash scripts/devnet-demo.sh   # full scripted lifecycle
```

---

## Quick start

### Install

Rust 1.89, Solana CLI 3.x, Anchor 1.0.2, cargo-nextest. Full toolchain notes: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).

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
| **Pool administrator** | `pool initialize` · `loan settle-default` |
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
Pending → Active → Repaid
                 ↘ Defaulted
Pending → cancel_loan
```

| State | Guarantor tracking | Escape hatches |
|-------|-------------------|----------------|
| Pending | `pending_guarantees` | Borrower: `cancel_loan` · Guarantor: `withdraw_cosign` |
| Active | `active_guarantees` | Repay or admin `settle_default` |
| Repaid / Defaulted | Cleared | Guarantors may exit when list empty |

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
- Whether a default is justified (admin judgment)
- SPL mint legitimacy at pool creation

Full threat model: [docs/SECURITY.md](docs/SECURITY.md).

---

## Project structure

```
programs/kzp-mini/src/
├── instructions/   Anchor account constraints
├── handlers/       CPIs + state updates
├── operations/     Pure business rules (unit-tested)
├── state.rs        Pool, Member, Loan
└── utils/          PDA helpers

cli/                kzp command-line client
tests/              solana-program-test integration suite
docs/               Architecture, CLI, accounting, security
scripts/            ci.sh, deploy.sh, devnet-demo.sh
keys/               Local keypairs (gitignored)
```

**Philosophy:** `instructions` = validation · `handlers` = orchestration · `operations` = testable rules without Anchor contexts.

---

## Testing

| Layer | Coverage |
|-------|----------|
| Unit tests | `programs/kzp-mini/src/operations/` |
| Integration | 46 `solana-program-test` scenarios |
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
| Policy vs code | Eligibility off-chain | Admin can `settle_default` without on-chain proof |
| Single admin | One `pool.admin` | No multisig in this release |
| Default split | 50/50 (`div_ceil` on odd amounts) | First guarantor pays extra token on odd sums |
| Loan cap | 3× savings, 5 guarantees | May block edge cases |
| Ledger vs vault | Both tracked; fail-closed exit/disburse | Safer; monitor for drift |
| Pending loans | Only borrower can cancel | Abandoned PDAs pay rent until cancelled |
| Upgrades | No migration instruction | Schema changes need fresh pool deploy |
| Toolchain | Rust 1.89, Anchor 1.0.2, Solana 3.x | Solana 4.x unsupported |

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
