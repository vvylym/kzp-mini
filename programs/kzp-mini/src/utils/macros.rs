//! Logging and validation macros inspired by Kamino vault patterns.

/// Logs a message and returns a program error when the invariant is false.
#[macro_export]
macro_rules! require_msg {
    ($invariant:expr, $error:expr, $($arg:tt)+) => {
        if !($invariant) {
            msg!($($arg)+);
            return Err($error.into());
        }
    };
}

pub use require_msg;
