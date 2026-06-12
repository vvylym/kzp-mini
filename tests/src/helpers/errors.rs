//! Assertion helpers for `solana-program-test` transaction failures.

use solana_program_test::BanksClientError;
use solana_sdk::transaction::TransactionError;

/// Asserts that a banks client error is a custom program error with `expected_code`.
pub(crate) fn assert_custom_error(err: BanksClientError, expected_code: u32) {
    let BanksClientError::TransactionError(tx_err) = err else {
        panic!("expected transaction error, got {err:?}");
    };

    let TransactionError::InstructionError(
        _,
        solana_sdk::instruction::InstructionError::Custom(code),
    ) = tx_err
    else {
        panic!("expected custom instruction error, got {tx_err:?}");
    };

    assert_eq!(code, expected_code);
}
