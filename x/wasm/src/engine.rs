//! CosmWasm execution engine and trait definitions.
//!
//! This module defines the [`WasmEngine`] trait which abstracts over the
//! underlying execution backend for smart contracts. The default implementation
//! uses `cosmwasm_vm` and mirrors the high level API provided by the Go
//! [`wasmvm`](https://github.com/CosmWasm/wasmvm) `VM` struct. It exposes
//! operations for storing code, instantiating contracts, executing calls,
//! migrating, querying and handling IBC events.
//!
//! Responsibilities:
//! - Manage a `cosmwasm_vm::Cache` of compiled WASM binaries.
//! - Provide functions corresponding to the `wasmvm` API such as `store_code`,
//!   `instantiate`, `execute`, `query` and the IBC callbacks.
//! - Bridge the VM's `Backend` trait with concrete storage, API and querier
//!   implementations defined elsewhere in this crate.
//!
//! Constraints & Security:
//! - All execution must be deterministic and respect the gas limits supplied via
//!   Gears' gas meter integration. No unsafe code is allowed.
//! - Persistent caches must be pinned/unpinned carefully to avoid leaking memory
//!   or compiling untrusted code more than once.
//! - Inputs should be validated to avoid injection of malformed WASM modules.
//!
//! This file only defines the trait and a skeletal struct. Implementation will
//! follow the design laid out in `COSMWASM_ADR.md` and `COSMWASM_PRD.md`.
use cosmwasm_vm::VmError;

/// High level interface used by the `Keeper` to execute contracts.
pub trait WasmEngine {
    /// Stores new contract code and returns an identifier.
    fn store_code(&mut self, wasm: &[u8]) -> Result<u64, VmError>;

    /// Instantiates a contract from previously stored code.
    fn instantiate(&mut self, code_id: u64, msg: &[u8]) -> Result<Vec<u8>, VmError>;

    /// Executes a contract call.
    fn execute(&mut self, contract_addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, VmError>;

    /// Runs a read-only query against a contract.
    fn query(&self, contract_addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, VmError>;
}

/// Placeholder engine using `cosmwasm_vm` directly.
pub struct CosmwasmEngine {
    // TODO: cache and backend fields
}

impl WasmEngine for CosmwasmEngine {
    fn store_code(&mut self, _wasm: &[u8]) -> Result<u64, VmError> {
        unimplemented!()
    }

    fn instantiate(&mut self, _code_id: u64, _msg: &[u8]) -> Result<Vec<u8>, VmError> {
        unimplemented!()
    }

    fn execute(&mut self, _contract_addr: &[u8], _msg: &[u8]) -> Result<Vec<u8>, VmError> {
        unimplemented!()
    }

    fn query(&self, _contract_addr: &[u8], _msg: &[u8]) -> Result<Vec<u8>, VmError> {
        unimplemented!()
    }
}
