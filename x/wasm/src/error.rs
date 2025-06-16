//! Unified error type for the CosmWasm module.
//!
//! This mirrors wasmd's error handling by grouping common failure modes into a
//! single enum that can be converted into ABCI response codes. The variants are
//! intentionally coarse grained so that clients can handle them without needing
//! to inspect VM specific details.

use cosmwasm_vm::VmError;
use thiserror::Error;

use tendermint_informal::abci::Code;

/// Errors returned by the CosmWasm engine and keeper.
#[derive(Debug, Error)]
pub enum WasmError {
    /// Failure during wasm bytecode compilation.
    #[error("compilation failed for code {code_id}: {source}")]
    CompileErr { source: VmError, code_id: u64 },
    /// Any runtime issue when executing or querying a contract.
    #[error("runtime error: {source}")]
    RuntimeErr { source: VmError },
    /// Lookup failures for contracts or code.
    #[error("{kind} not found")]
    NotFound { kind: &'static str },
    /// Sender lacks the required permissions for the attempted action.
    #[error("unauthorized: {action}")]
    Unauthorized { action: &'static str },
    /// Malformed message or query payload.
    #[error("invalid request: {reason}")]
    InvalidRequest { reason: String },
    /// Unexpected internal issue. Should be rare in production.
    #[error("internal error: {reason}")]
    Internal { reason: String },
}

impl WasmError {
    /// Return the ABCI code matching this error.
    ///
    /// Mapping follows wasmd conventions: `NotFound` -> 5, `Unauthorized` -> 4,
    /// `InvalidRequest` -> 3 and all other variants map to code 1.
    pub fn abci_code(&self) -> Code {
        match self {
            WasmError::NotFound { .. } => Code::from(5u32),
            WasmError::Unauthorized { .. } => Code::from(4u32),
            WasmError::InvalidRequest { .. } => Code::from(3u32),
            WasmError::Internal { .. }
            | WasmError::CompileErr { .. }
            | WasmError::RuntimeErr { .. } => Code::from(1u32),
        }
    }
}

impl From<VmError> for WasmError {
    fn from(err: VmError) -> Self {
        match err {
            VmError::CompileErr { .. } | VmError::StaticValidationErr { .. } => {
                WasmError::CompileErr {
                    source: err,
                    code_id: 0,
                }
            }
            _ => WasmError::RuntimeErr { source: err },
        }
    }
}

impl From<anyhow::Error> for WasmError {
    fn from(err: anyhow::Error) -> Self {
        WasmError::Internal {
            reason: err.to_string(),
        }
    }
}

impl From<std::io::Error> for WasmError {
    fn from(err: std::io::Error) -> Self {
        WasmError::Internal {
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for WasmError {
    fn from(err: serde_json::Error) -> Self {
        WasmError::InvalidRequest {
            reason: err.to_string(),
        }
    }
}
