//! Transaction message definitions for the wasm module.
//!
//! Messages correspond to user actions such as uploading code, instantiating and
//! executing contracts. Structures are shaped to be compatible with
//! `cosmwasm_std` and will ultimately derive the `Tx` procedural macros defined
//! elsewhere in this repository.
//!
//! This file only declares the Rust structs without implementing the protobuf
//! conversions or CLI wiring yet.
use serde::{Deserialize, Serialize};

/// Upload new contract code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgStoreCode {
    pub sender: String,
    pub wasm_byte_code: Vec<u8>,
}

/// Instantiate a contract.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgInstantiateContract {
    pub sender: String,
    pub code_id: u64,
    pub msg: Vec<u8>,
}

/// Execute a contract.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgExecuteContract {
    pub sender: String,
    pub contract: String,
    pub msg: Vec<u8>,
}
