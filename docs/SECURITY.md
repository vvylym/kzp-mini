# Security

Design context: [Architecture](../README.md#architecture). Instruction reference: [INSTRUCTIONS.md](./INSTRUCTIONS.md).

## Threat model

KZP Minimal is an on-chain mutual-aid pool: members deposit SPL tokens, borrow against savings with two guarantors, repay, and exit when obligations are clear.

**Trust assumptions**

- Pool admin chooses a legitimate SPL mint at initialization.
- Members understand guarantor obligations before co-signing.
- Clients pass correct associated token accounts (validated by mint + owner constraints).
- Admin default settlement is trusted for bad-debt resolution (no on-chain oracle). A single admin key is used for this release.

## Controls in place

| Area | Mitigation |
|------|------------|
| **Access control** | Signer checks on all mutating instructions; borrower/guarantor PDAs derived from seeds + bumps |
| **Token CPIs** | Vault PDA signs outbound transfers; user signs inbound deposits/repayments; mint/owner constraints on ATAs |
| **Arithmetic** | `checked_add` / `checked_sub` on savings and pool counters |
| **Loan limits** | 3× savings cap, one active loan per borrower, five guarantees per guarantor (active + pending) |
| **Co-sign** | Pending obligations in `pending_guarantees`; active only at disbursement when both sign |
| **Vault liquidity (disburse)** | `InsufficientVaultLiquidity` at disbursement if `vault.amount < principal` |
| **Vault liquidity (exit)** | `InsufficientVaultLiquidity` if `vault.amount < member.savings_balance` (fail-closed when ledger exceeds physical vault) |
| **Pending cleanup** | `cancel_loan` (borrower) and `withdraw_cosign` (guarantor) while `Pending` |
| **Exit** | Blocked for `active_loan`, `active_guarantees`, or `pending_guarantees` |
| **Default** | Admin-only `settle_default`; 50/50 from guarantor savings ledger **and** SPL transfer from guarantor ATAs to vault |
| **PDAs** | Pool, vault, member, loan accounts use canonical seeds; vault bump stored on loan at request |

## Co-sign lifecycle

| Step | Loan | Guarantor `pending_guarantees` | Guarantor `active_guarantees` | Can exit? |
|------|------|-------------------------------|------------------------------|-----------|
| Requested | Pending | `[]` | `[]` | Yes |
| One co-sign | Pending | `[loan]` | `[]` | No (pending) |
| Both co-sign / disburse | Active | `[]` | `[loan]` | No (active) |
| Repaid | Repaid | `[]` | `[]` | Yes |
| Cancelled (borrower) | closed | cleared | `[]` | Yes |
| Co-sign withdrawn | Pending | cleared | `[]` | Yes |

Guarantors can `withdraw_cosign` while pending. Borrower can `cancel_loan` anytime while pending.

---

## Mitigated

Issues identified during audit and subsequently handled on-chain.

| # | Former gap | Fix | Instruction / check |
|---|------------|-----|---------------------|
| 1 | Disbursement could drain an under-funded vault | Vault balance checked before transfer | `co_sign_loan` → `InsufficientVaultLiquidity` |
| 2 | Pending loans could stall forever | Borrower-initiated cancel | `cancel_loan` |
| 3 | Partial co-sign locked guarantors in `active_guarantees` or allowed exit griefing | Separate `pending_guarantees`; exit blocked while pending | `co_sign_loan`, `withdraw_cosign`, `exit_pool` |
| 4 | `total_outstanding_loans` was stale | Counter updated on disburse, repay, default | `co_sign_loan`, `repay_loan`, `settle_default` |
| 5 | No on-chain default path | Admin settlement with 50/50 guarantor split | `settle_default` |
| 6 | Vault vs ledger drift on exit | Exit blocked when vault cannot cover payout | `exit_pool` → `InsufficientVaultLiquidity` |
| 7 | Default did not recapitalize vault | Guarantors transfer default shares to vault via SPL CPI | `settle_default` (guarantors co-sign token transfer) |

Integration tests in `tests/src/test_*.rs` cover these paths.

---

## Accepted limitations

By-design tradeoffs or operational constraints. Changing them requires a product decision.

| Topic | What happens | Why it stays |
|-------|--------------|--------------|
| **Admin default discretion** | Admin can call `settle_default` on any active loan with no on-chain proof of default. Guarantors must co-sign the transaction for SPL transfers. | Single admin for this release; policy is off-chain. |
| **Stalled pending loans** | If a guarantor `withdraw_cosign`s or never signs, loan stays `Pending` until borrower `cancel_loan`s. Guarantors cannot exit while pending. | Workflow friction only; no fund theft. |
| **Abandoned pending loans** | If borrower never cancels, loan PDA remains open and accrues rent. Only borrower can close it. | Solana account model; no third-party close without new instruction. |
| **Member layout change** | Redeploy with resized `Member` breaks existing member PDAs. | Fresh pool deploy required on upgrade. |
| **Underwater pool exits** | When `vault.amount < savings_balance`, exit fails. In normal operation entry fees in the vault provide a small buffer; mismatch mainly appears after ledger/vault drift (e.g. partial default without token transfer). | Fail-closed by design. |

**Off-chain mitigation:** operational policy for when admin calls `settle_default`; monitor vault vs aggregate savings.

---

## Optional hardening (future)

| Priority | Hardening | Closes / reduces | Notes |
|----------|-----------|------------------|-------|
| **Medium** | `close_stale_loan` (admin or borrower after timeout) | Abandoned pending loans / rent | Requires `Clock` sysvar + max pending duration |
| **Low** | On-chain default evidence (repayment deadline, attestation account) | Admin default discretion | Governance scope |
| **Low** | Invariant check: `total_savings == Σ member.savings_balance` | Counter drift | Better as off-chain indexer |
| **N/A** | Member migration instruction | Redeploy pain | Only if preserving accounts across upgrades |

---

## Reporting

See `security_txt` in `programs/kzp-mini/src/lib.rs` for contact and policy URLs.
