//! Integration-test support for `kzp-mini`.
//!
//! # Layout
//!
//! | Module | Responsibility |
//! |--------|----------------|
//! | [`app`] | [`TestApp`]: BPF loader, mint bootstrap |
//! | [`transactions`] | Submit txs, expect errors, SOL airdrops |
//! | [`tokens`] | ATAs, minting, balance reads |
//! | [`accounts`] | Deserialize pool / member / loan state |
//! | [`instructions`] | Build program instructions (same names as on-chain) |
//! | [`fixtures`] | Multi-step flows (`setup_pool`, `activate_loan`, …) |
//! | [`constants`] | Shared amounts and error-code mapping |
//!
//! PDA helpers: `kzp_mini::utils::pda`. Test setup macro: [`with_universe`].

mod accounts;
mod app;
mod constants;
mod errors;
mod fixtures;
mod instructions;
#[macro_use]
mod macros;
mod tokens;
mod transactions;

pub use app::TestApp;
pub use constants::{ENTRY_FEE, LAMPORTS, POOL_NAME};
pub use fixtures::{funded_admin, initialized_pool, member_with_savings};
