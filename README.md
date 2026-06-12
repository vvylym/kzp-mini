# KZP Minimal

On-chain workplace mutual-aid fund (Kasa Zapomogowa Pracownicza) on Solana. Members pay a one-time entry fee, deposit savings, request co-signed loans (two guarantors), repay, and exit when obligations are clear.

> **Superteam Poland submission** — this repository is an entry for [Build everyday real-world systems as on-chain Rust programs](https://superteam.fun/earn/listing/build-everyday-real-world-systems-as-on-chain-rust-programs) on [Superteam Earn](https://superteam.fun/earn).

**Repository:** [github.com/vvylym/kzp-mini](https://github.com/vvylym/kzp-mini)  
**Program ID (devnet):** `GsjUnBFvYtcxNwCrydPUjQTngGTqdx5v7APnWahnqwkx`

## Devnet demo

The program is deployed on devnet. A full pool lifecycle (initialize → join → deposit → loan → co-sign → repay → exit) was executed with the `kzp` CLI.

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
| Co-sign + disburse (guarantor B) | [explorer](https://explorer.solana.com/tx/5d9rpicU6iWgEbQDYb6oVDemv4Vd6fNnqCF9Q3hEDKx1kiMp8XR7Zx2Edn59eDTerMgabcaMR2H7UK8f58q5s3Ni?cluster=devnet) |
| Repay loan | [explorer](https://explorer.solana.com/tx/3dHLmTDjhWkPsp3ruWUGqkLea5aazYgHn7siPUSHwycemdeiiwE7LJxP2o9ieRdQisCTdzsWSuGxCfKPPen6Hizp?cluster=devnet) |
| Exit pool | [explorer](https://explorer.solana.com/tx/4yyvHNjit8M4gEwpWzXBJUAPeyEAVUXsAz4eZsrcLNEHU8CsX5rN8kWQcC8qXYh4TJkpwkuAomhmCr68UYmkG9AV?cluster=devnet) |

**Demo accounts:** pool `4nZf8sPfRvKUucN43FirPQ3T3JvvkCo4LFukGYCRL7tg` · loan `toqwDKpVPAdHPkLSJWyh5yLdDTf7szVEKPRtVNek4bL` · mint `9CsRiEPkagzfTLTDhsBhuvfHNXupBugtHzy1wPefTi4V`

Reproduce locally (requires a funded devnet wallet with ~3 SOL for first-time deploy):

```bash
bash scripts/devnet-demo.sh
```

## Installation

### Prerequisites

| Tool | Version / notes |
|------|-----------------|
| [Rust](https://rustup.rs/) | 1.89 (`rust-toolchain.toml`) — include `rustfmt`, `clippy`, and `llvm-tools-preview` |
| [Solana CLI](https://docs.anza.xyz/cli/install) | 3.x |
| [Anchor](https://www.anchor-lang.com/docs/installation) | 1.0.2 (`avm install 1.0.2`) |
| [pnpm](https://pnpm.io/) | Anchor workspace metadata |
| [cargo-nextest](https://nexte.st/) | `cargo install cargo-nextest --locked` |

```bash
# Rust components used by CI (format, lint, tests)
rustup component add rustfmt clippy

# Test runner (same stack as scripts/ci.sh and GitHub Actions)
cargo install cargo-nextest --locked
```

Program and demo keypairs live under `keys/` and are **not** in git — see [keys/README.md](keys/README.md).

Solana 4.x is not supported by the current dependency set.

### Clone and build

```bash
# Clone the repository
git clone https://github.com/vvylym/kzp-mini.git
cd kzp-mini

# Build the on-chain program
NO_DNA=1 anchor build --ignore-keys

# Build the CLI (optional; or use cargo run -p kzp-cli --release --)
cargo build -p kzp-cli --release
export PATH="$PWD/target/release:$PATH"
```

Deploy to a cluster when you need your own program instance (devnet is the default for the CLI):

```bash
bash scripts/deploy.sh devnet    # or localnet / mainnet
```

## Usage

The `kzp` CLI talks to the on-chain program over RPC. **Every subcommand works on any cluster** — pass `--cluster localnet|devnet|mainnet` or `--rpc-url <URL>` to override the default.

**Default cluster: devnet** (`https://api.devnet.solana.com`). Wallet: `~/.config/solana/id.json` unless `--wallet` is set.

### Devnet setup

After installation you still need a funded wallet, the program deployed at the ID above, and an SPL mint with tokens in member ATAs:

```bash
# Create a keypair if you do not have ~/.config/solana/id.json
solana-keygen new

# Point Solana CLI at devnet (optional; kzp already defaults to devnet RPC)
solana config set --url devnet

# Fund the wallet with devnet SOL for rent and fees
solana airdrop 2

# Deploy the program (skip if this program ID is already live on devnet)
NO_DNA=1 anchor build --ignore-keys
bash scripts/deploy.sh devnet

# Create a devnet SPL mint and mint tokens to your ATA (save MINT for pool init)
spl-token create-token
spl-token create-account <MINT>
spl-token mint <MINT> 1000000000
```

Verify connectivity with `kzp config` (prints RPC URL, cluster, wallet, payer, and program ID).

### Commands

Replace `<MINT>`, `<POOL>`, `<LOAN>`, and pubkeys with values from your run. Amounts are in **base units** (smallest SPL denomination).

```bash
# Confirm RPC URL, cluster, wallet, payer, and program ID
kzp config

# Admin: create pool + vault for an SPL mint (prints pool PDA)
kzp pool initialize --name "Fabryka Lodz KZP" --entry-fee 50 --mint <MINT>

# Member: pay entry fee and open a member PDA (wallet must hold ≥ entry_fee tokens)
kzp pool join --pool <POOL> --entry-fee 50

# Member: move tokens from your ATA into the pool vault; credits savings ledger
kzp pool deposit --pool <POOL> --amount 1000000

# Borrower: open a pending loan (needs two guarantor pubkeys who are already members)
kzp loan request --pool <POOL> --nonce 1 --amount 500000 --guarantor-a <PUBKEY_A> --guarantor-b <PUBKEY_B>

# Guarantor: co-sign pending loan (run twice, once per guarantor wallet; disburses on second sign)
kzp loan cosign --loan <LOAN>

# Borrower: repay principal to the vault (partial or full)
kzp loan repay --loan <LOAN> --amount 500000

# Borrower: cancel a loan still in Pending status
kzp loan cancel --loan <LOAN>

# Guarantor: revoke your partial co-sign while the loan is Pending
kzp loan withdraw-cosign --loan <LOAN>

# Admin (payer wallet) + both guarantors: settle default; wallet flags are keypair JSON paths
kzp loan settle-default --loan <LOAN> --pool <POOL> --guarantor-a-wallet ~/.config/solana/guarantor_a.json --guarantor-b-wallet ~/.config/solana/guarantor_b.json

# Member: withdraw savings and close member PDA (no active loan or guarantees)
kzp pool exit --pool <POOL>

# Simulate any command without sending (prints logs on failure)
kzp --dry-run pool join --pool <POOL> --entry-fee 50
```

**Other clusters:** append `--cluster localnet` (start `solana-test-validator` first) or `--cluster mainnet` (real SOL and tokens). **Other wallet:** `--wallet /path/to/keypair.json`.

## Architecture

### Domain

A **workplace mutual-aid pool** (KZP): coworkers save together and lend to each other with **social guarantee** — every loan requires **two guarantors** who accept partial liability if the borrower defaults.

The program enforces membership, savings, loan limits (3× savings), co-sign before disbursement, repayment, cancellation, admin default settlement, and exit only when obligations are clear. **Who may join** and **when a default is justified** stay off-chain; on-chain code enforces **mechanics and fund safety**.

### Traditional KZP: how the friction works

In a typical employer-run or committee-run KZP:

| Concern | Traditional handling |
|---------|---------------------|
| **Membership** | HR or a clerk maintains a roster; entry fees collected via payroll or cash |
| **Savings** | Spreadsheet or ledger entry per member; physical cash or bank account balance |
| **Loans** | Committee approves borrower + two guarantors; trust and workplace norms |
| **Co-sign / guarantee** | Paper signatures or verbal agreement; no atomic link between “I guarantee” and money moving |
| **Repayment** | Manual recording; disputes settled in meetings |
| **Default** | Committee decides who pays; guarantors pressured socially; settlement may be partial or delayed |
| **Exit** | Member must be “clear” on loans and guarantees — verified manually |

Friction appears as **coordination cost** (meetings, chasing signatures), **opaque state** (who guaranteed what, is the cash box solvent?), and **dispute resolution** outside any shared rule engine. Nothing prevents disbursing more than the pool holds unless someone notices. Guarantors can be counted as obligated before funds actually move, or stuck informally after changing their mind.

### On Solana: how this program maps it

| Traditional concept | On-chain implementation |
|---------------------|-------------------------|
| Pool / cash box | **Pool** PDA + **vault** PDA holding SPL tokens |
| Member record | **Member** PDA per wallet (`savings_balance`, obligations) |
| Loan application | **Loan** PDA (`Pending` → `Active` → `Repaid` / `Defaulted`) |
| Two guarantors | `guarantor_a` / `guarantor_b`; both must co-sign to disburse |
| Partial approval | `guarantor_*_signed` flags + `pending_guarantees` before disbursement |
| Disbursement | `co_sign_loan` CPI from vault to borrower ATA when both signed and vault ≥ principal |
| Repayment | `repay_loan` CPI to vault; updates `outstanding` and pool counters |
| Default | Admin-initiated `settle_default`; 50/50 ledger debit + guarantor SPL to vault (guarantors sign) |
| Leave the pool | `exit_pool` pays `savings_balance` from vault and closes Member PDA |

**PDAs:** Pool `["pool", admin, pool_name]` · Vault `["vault", pool]` · Member `["member", pool, owner]` · Loan `["loan", pool, borrower, loan_nonce_le]`. Helpers: `kzp_mini::utils::pda`.

**Instructions (10):** `initialize_pool` · `join_pool` · `deposit_savings` · `request_loan` · `co_sign_loan` · `repay_loan` · `cancel_loan` · `withdraw_cosign` · `settle_default` · `exit_pool`. Details: [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md).

### Code layout

```
programs/kzp-mini/src/
├── lib.rs              #[program] dispatch → handlers::*::handle
├── state.rs            Pool, Member, Loan, LoanStatus
├── instructions/       Anchor account constraints only
├── handlers/           CPIs + state updates; one `handle` per instruction
├── operations/         Pure rules/math (unit-tested)
└── utils/              PDA seeds and helpers
```

Handlers are split from `instructions/` so business rules in `operations/` can be tested without full Anchor contexts. The `instructions/withdraw_co_sign.rs` module name differs from the `withdraw_cosign` instruction name only at the file level.

### Accounting: ledger vs vault

1. **SPL vault** — actual token balance in the vault PDA.
2. **Ledger** — `pool.total_savings` and each `member.savings_balance`.

Entry fees go to the vault but **not** to member savings, leaving a small **liquidity buffer** for disburse → repay → exit cycles. `pool.total_outstanding_loans` is updated on disburse, repay, and default.

**Fail-closed checks:** `co_sign_loan` requires `vault.amount >= principal`; `exit_pool` requires `vault.amount >= savings_balance`. If ledger and vault diverge, exit is blocked rather than over-paying.

### Loan and guarantee lifecycle

```
Pending → Active → Repaid
                 ↘ Defaulted
Pending → cancel_loan (account closed)
```

**`pending_guarantees`** vs **`active_guarantees`:** partial co-sign only adds pending obligations; active guarantees are set after both co-signs and successful disbursement. That avoids locking guarantors for exit before money moves, and avoids treating them as fully liable while the loan is still pending.

| Actor | Escape hatch while Pending |
|-------|----------------------------|
| Borrower | `cancel_loan` |
| Guarantor | `withdraw_cosign` |

### Tradeoffs & constraints

| Topic | Choice | Implication |
|-------|--------|-------------|
| **Policy vs code** | Eligibility and default justification off-chain | Admin can call `settle_default` without on-chain proof of default; guarantors must still sign SPL transfers |
| **Single admin** | One `pool.admin` | Simple ops; no multisig in this release |
| **Default split** | 50/50 on outstanding (`div_ceil` for odd amounts) | Predictable guarantor liability; first guarantor pays the extra token on odd sums |
| **Loan cap** | 3× savings, 5 guarantees/member, one active loan/borrower | Limits exposure; may block legitimate edge cases |
| **Ledger vs vault** | Both tracked; exit/disburse fail-closed | Safer than spreadsheet drift; requires monitoring if counters diverge from reality |
| **Pending loans** | Only borrower can `cancel_loan` | Abandoned pending loans keep the Loan PDA and pay rent until cancelled |
| **Account upgrades** | No migration instruction | `Member` layout changes need a **fresh pool deploy**; devnet PDAs are not preserved across schema changes |
| **Runtime limits** | BPF stack, account size | e.g. `Box<>` on `settle_default` accounts; `Member` guarantee vectors capped at 5 |
| **Toolchain** | Rust 1.89, Anchor 1.0.2, Solana 3.x | Solana 4.x not supported by current dependencies |
| **Testing** | `operations/` unit tests + 46 `solana-program-test` integration tests | Specs in [docs/USE_CASES.md](docs/USE_CASES.md); conventions in [tests/README.md](tests/README.md) |

Full threat model and mitigations: [docs/SECURITY.md](docs/SECURITY.md). Account field reference: [docs/STATE.md](docs/STATE.md).

## Documentation

| Document | Contents |
|----------|----------|
| [docs/README.md](docs/README.md) | Doc index and quick reference |
| [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md) | All instructions and accounts |
| [docs/STATE.md](docs/STATE.md) | Pool, Member, Loan layouts |
| [docs/USE_CASES.md](docs/USE_CASES.md) | BDD acceptance criteria |
| [docs/ERRORS.md](docs/ERRORS.md) | Error codes |
| [docs/SECURITY.md](docs/SECURITY.md) | Threat model and mitigations |
| [tests/README.md](tests/README.md) | Integration tests |

## Contributing

Run the full local CI before opening a pull request:

```bash
# Format, clippy, nextest, anchor build, CLI smoke test
bash scripts/ci.sh
```

Individual steps:

```bash
# Format check
cargo fmt --all -- --check

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build program first (integration tests need target/deploy/kzp_mini.so)
mkdir -p target/deploy && cp keys/program.json target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor build --ignore-keys

# Tests (nextest)
cargo nextest run --workspace --all-targets --all-features

# Anchor test alias (runs nextest via Anchor.toml)
anchor test
```

Integration tests use `solana-program-test` and must run with a single thread when invoked directly (`cargo nextest` handles workspace scheduling; see [tests/README.md](tests/README.md)). Business rules in `programs/kzp-mini/src/operations/` should stay unit-tested; acceptance criteria live in [docs/USE_CASES.md](docs/USE_CASES.md).

## License

MIT
