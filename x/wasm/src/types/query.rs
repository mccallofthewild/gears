//! Query request and response types for the wasm module.
//!
//! These structures will be used by gRPC/REST endpoints as well as in-process
//! queries performed by other modules. They are intentionally small for now and
//! will be expanded to cover the full set of CosmWasm queries (code info,
//! contract info, raw state etc.).
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryContractInfoRequest {
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryContractInfoResponse {
    pub code_id: u64,
}
