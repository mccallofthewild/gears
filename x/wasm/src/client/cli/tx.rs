//! CLI transaction subcommands for the wasm module.
//!
//! These commands allow broadcasting the three basic CosmWasm messages
//! supported by this crate. Behaviour mirrors the upstream
//! [`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/client/cli/tx.go)
//! commands but is simplified to match the available message types in
//! `x/wasm/src/message.rs`.
//!
//! - `store-code` uploads raw wasm bytecode.
//! - `instantiate` creates a new contract instance from an uploaded code id.
//! - `execute` calls an existing contract with a JSON message.
//!
//! Messages produced by these helpers are forwarded to the standard
//! transaction handler provided by `gears`.

use anyhow::Result;
use clap::{Args, Subcommand};
use gears::types::{address::AccAddress, tx::Message as _};

use crate::message::{
    Message as WasmMessage, MsgExecuteContract, MsgInstantiateContract, MsgStoreCode,
};

/// Entry point for wasm transaction commands.
#[derive(Args, Debug, Clone)]
pub struct WasmTxCli {
    #[command(subcommand)]
    pub command: WasmCommands,
}

/// Supported wasm transaction subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum WasmCommands {
    /// Upload wasm bytecode from the given file path.
    StoreCode { file: std::path::PathBuf },
    /// Instantiate a contract from previously uploaded code. `msg` is hex or UTF-8 encoded JSON.
    Instantiate { code_id: u64, msg: String },
    /// Execute a contract. `msg` is hex or UTF-8 encoded JSON.
    Execute { contract: AccAddress, msg: String },
}

/// Convert CLI arguments into a [`WasmMessage`] ready for signing and broadcasting.
pub fn run_wasm_tx_command(args: WasmTxCli, from_address: AccAddress) -> Result<WasmMessage> {
    let sender = from_address.to_string();

    match args.command {
        WasmCommands::StoreCode { file } => {
            let wasm = std::fs::read(file)?;
            Ok(WasmMessage::StoreCode(MsgStoreCode {
                sender,
                wasm_byte_code: wasm,
            }))
        }
        WasmCommands::Instantiate { code_id, msg } => {
            let bytes = hex::decode(&msg).unwrap_or_else(|_| msg.into_bytes());
            Ok(WasmMessage::Instantiate(MsgInstantiateContract {
                sender,
                code_id,
                msg: bytes,
            }))
        }
        WasmCommands::Execute { contract, msg } => {
            let bytes = hex::decode(&msg).unwrap_or_else(|_| msg.into_bytes());
            Ok(WasmMessage::Execute(MsgExecuteContract {
                sender,
                contract: contract.to_string(),
                msg: bytes,
            }))
        }
    }
}
