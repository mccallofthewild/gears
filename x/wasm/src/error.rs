//! Error definitions for the wasm module.
//!
//! This module centralises all error types that can be returned by the `Keeper`
//! or `WasmEngine`. Clear error handling is essential for diagnosing contract
//! failures and ensuring deterministic behaviour across nodes.
//!
//! Errors are modelled using `thiserror` and closely mirror the variants found
//! in `wasmd` and `cosmwasm_vm`.
use gears::application::handlers::node::{ModuleInfo, TxError};
use gears::gas::store::errors::GasStoreErrors;
use serde_json::Error as SerdeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasmError {
    /// Returned when contract code fails validation or compilation.
    #[error("invalid contract code: {0}")]
    InvalidCode(String),

    /// Wrapper around `cosmwasm_vm::VmError`.
    #[error("vm error: {0}")]
    Vm(#[from] cosmwasm_vm::VmError),

    /// Error serialising or deserialising contract data.
    #[error("serde error: {0}")]
    Serde(#[from] SerdeError),

    /// Error originating from gas metering/storage.
    #[error("gas error: {0}")]
    Gas(#[from] GasStoreErrors),

    /// Generic keeper failure.
    #[error("keeper error: {0}")]
    Keeper(String),
}

impl WasmError {
    /// Convert this error into an ABCI [`TxError`].
    ///
    /// The codes are aligned with the errors returned by `wasmd` so that
    /// external tooling can rely on stable, non‐zero identifiers.
    pub fn into<MI: ModuleInfo>(self) -> TxError {
        let code = match self {
            WasmError::InvalidCode(_) => nz::u16!(1),
            WasmError::Vm(_) => nz::u16!(2),
            WasmError::Serde(_) => nz::u16!(3),
            WasmError::Gas(_) => nz::u16!(4),
            WasmError::Keeper(_) => nz::u16!(5),
        };

        TxError::new::<MI>(self.to_string(), code)
    }
}
