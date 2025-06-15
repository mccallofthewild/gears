//! Module parameter definitions.
//!
//! Parameters control gas costs and permissions for wasm execution. They are
//! loaded from the application parameter store via the standard `ParamsKeeper`
//! mechanism. The structure closely follows the schema used by `wasmd` so that
//! existing genesis files and governance proposals remain compatible.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmParams {
    /// Maximum allowed size for contract byte code in bytes.
    pub max_wasm_size: u64,
}

/// Placeholder trait for accessing module params from a keeper.
pub trait WasmParamsKeeper {
    fn params(&self) -> WasmParams;
}
