//! Transaction message definitions for the wasm module.
//!
//! Messages correspond to actions such as uploading code, instantiating and
//! executing contracts. Structures implement the `AppMessage` derive to integrate
//! with the rest of the application.

use gears::derive::AppMessage;
use serde::{Deserialize, Serialize};

/// Upload new contract code.
#[derive(Clone, Debug, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgStoreCode")]
pub struct MsgStoreCode {
    #[msg(signer)]
    pub sender: String,
    pub wasm_byte_code: Vec<u8>,
}

/// Instantiate a contract.
#[derive(Clone, Debug, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgInstantiateContract")]
pub struct MsgInstantiateContract {
    #[msg(signer)]
    pub sender: String,
    pub code_id: u64,
    pub msg: Vec<u8>,
}

/// Execute a contract.
#[derive(Clone, Debug, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgExecuteContract")]
pub struct MsgExecuteContract {
    #[msg(signer)]
    pub sender: String,
    pub contract: String,
    pub msg: Vec<u8>,
}

/// Enum grouping all CosmWasm messages.
#[derive(Debug, Clone, Serialize, AppMessage)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmwasm.wasm.v1.MsgStoreCode")]
    #[msg(url(path = MsgStoreCode::TYPE_URL))]
    StoreCode(MsgStoreCode),
    #[serde(rename = "/cosmwasm.wasm.v1.MsgInstantiateContract")]
    #[msg(url(path = MsgInstantiateContract::TYPE_URL))]
    Instantiate(MsgInstantiateContract),
    #[serde(rename = "/cosmwasm.wasm.v1.MsgExecuteContract")]
    #[msg(url(path = MsgExecuteContract::TYPE_URL))]
    Execute(MsgExecuteContract),
}
