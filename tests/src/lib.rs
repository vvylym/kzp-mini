//! Integration tests for `kzp-mini` using `solana-program-test`.
//!
//! See `tests/README.md` for layout and how to run.

#[cfg(test)]
#[macro_use]
mod helpers;
#[cfg(test)]
mod test_cancel_loan;
#[cfg(test)]
mod test_cosign_loan;
#[cfg(test)]
mod test_deposit_savings;
#[cfg(test)]
mod test_exit_pool;
#[cfg(test)]
mod test_initialize_pool;
#[cfg(test)]
mod test_join_pool;
#[cfg(test)]
mod test_repay_loan;
#[cfg(test)]
mod test_request_loan;
#[cfg(test)]
mod test_settle_default;
#[cfg(test)]
mod test_withdraw_cosign;
