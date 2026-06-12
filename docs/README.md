# Documentation index

**Program ID (devnet):** `GsjUnBFvYtcxNwCrydPUjQTngGTqdx5v7APnWahnqwkx`

Start with the [root README](../README.md) for overview and navigation.

## By audience

| You are… | Start here |
|----------|------------|
| Hackathon judge | [README § Live devnet demo](../README.md#live-devnet-demo) |
| Solana developer | [GETTING_STARTED.md](./GETTING_STARTED.md) → [CLI.md](./CLI.md) |
| Security reviewer | [SECURITY.md](./SECURITY.md) → [ACCOUNTING.md](./ACCOUNTING.md) |
| Contributor | [ARCHITECTURE.md](./ARCHITECTURE.md) → [USE_CASES.md](./USE_CASES.md) |

## Reference

| Document | Purpose |
|----------|---------|
| [GETTING_STARTED.md](./GETTING_STARTED.md) | Install, build, deploy, devnet wallet |
| [CLI.md](./CLI.md) | Full `kzp` command reference by role |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Accounts, instructions, traditional mapping |
| [ACCOUNTING.md](./ACCOUNTING.md) | Ledger vs vault, limits, default split |
| [INSTRUCTIONS.md](./INSTRUCTIONS.md) | Instruction account metas |
| [STATE.md](./STATE.md) | On-chain account layouts |
| [SECURITY.md](./SECURITY.md) | Threat model and mitigations |
| [USE_CASES.md](./USE_CASES.md) | BDD acceptance criteria |
| [ERRORS.md](./ERRORS.md) | `PoolError` codes |
| [../tests/README.md](../tests/README.md) | Integration test conventions |
| [../keys/README.md](../keys/README.md) | Local keypair layout |

## Quick reference

### Instructions

`initialize_pool` · `join_pool` · `deposit_savings` · `request_loan` · `co_sign_loan` · `repay_loan` · `cancel_loan` · `withdraw_cosign` · `settle_default` · `exit_pool`

### PDAs

| Account | Seeds |
|---------|-------|
| Pool | `["pool", admin, pool_name]` |
| Vault | `["vault", pool]` |
| Member | `["member", pool, owner]` |
| Loan | `["loan", pool, borrower, loan_nonce_le]` |
