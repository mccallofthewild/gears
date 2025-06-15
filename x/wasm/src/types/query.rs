//! Query request and response types for the wasm module.
//!
//! These structures will be used by gRPC/REST endpoints as well as in-process
//! queries performed by other modules. They are intentionally small for now and
//! will be expanded to cover the full set of CosmWasm queries (code info,
//! contract info, raw state etc.).
use serde::{Deserialize, Serialize};

/// Query the metadata for a given contract address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryContractInfoRequest {
    /// Bech32 address of the contract.
    pub address: String,
}

/// Response containing the code identifier the contract was instantiated from.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryContractInfoResponse {
    /// Identifier of the uploaded WASM code.
    pub code_id: u64,
}

/// Request the raw bytes of stored code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryCodeRequest {
    /// Identifier returned by a previous `MsgStoreCode` call.
    pub code_id: u64,
}

/// Response with the original wasm bytecode.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryCodeResponse {
    pub wasm_byte_code: Vec<u8>,
}

/// Request all known code identifiers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryCodesRequest {}

/// Response listing stored code identifiers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryCodesResponse {
    pub code_ids: Vec<u64>,
}

/// Request contracts that were instantiated from a particular code id.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryContractsByCodeRequest {
    pub code_id: u64,
}

/// Response listing contract addresses.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryContractsByCodeResponse {
    pub contracts: Vec<String>,
}

/// Request a smart query to be executed by the contract.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuerySmartContractStateRequest {
    pub address: String,
    pub query_data: Vec<u8>,
}

/// Response with the json bytes returned by the contract.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuerySmartContractStateResponse {
    pub data: Vec<u8>,
}

/// Request a raw key from the contract storage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryRawContractStateRequest {
    pub address: String,
    pub key: Vec<u8>,
}

/// Response with raw value bytes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryRawContractStateResponse {
    pub data: Vec<u8>,
}
