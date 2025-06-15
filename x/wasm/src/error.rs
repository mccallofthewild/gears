//! Error definitions for the wasm module.
//!
//! This module centralises all error types that can be returned by the `Keeper`
//! or `WasmEngine`. Clear error handling is essential for diagnosing contract
//! failures and ensuring deterministic behaviour across nodes.
//!
//! Errors are modelled using `thiserror` and closely mirror the variants found
//! in `wasmd` and `cosmwasm_vm`.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasmError {
    /// Returned when contract code fails validation or compilation.
    #[error("invalid contract code: {0}")]
    InvalidCode(String),

    /// Wrapper around `cosmwasm_vm::VmError`.
    #[error("vm error: {0}")]
    Vm(#[from] cosmwasm_vm::VmError),

    /// Generic keeper failure.
    #[error("keeper error: {0}")]
    Keeper(String),
}
