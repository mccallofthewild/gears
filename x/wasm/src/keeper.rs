//! Core keeper for the wasm module.
//!
//! The `Keeper` owns persistent storage keys for contract code and instances and
//! exposes high level methods used by the rest of the application. It mirrors
//! concepts from [`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/keeper/keeper.go)
//! but is written entirely in Rust and delegates execution to a [`WasmEngine`].
//!
//! Major duties include:
//! - Persisting uploaded contract code and assigning stable identifiers.
//! - Instantiating contracts with configured permissions and tracking their
//!   metadata.
//! - Executing and querying contracts via an associated [`WasmEngine`] instance.
//! - Managing contract admin updates, code migration and pinning.
//!
//! Constraints & Security:
//! - All state mutations must go through the provided context traits to ensure
//!   atomicity and gas accounting.
//! - Access control should mirror the Cosmos SDK module, preventing unauthorised
//!   calls.
//! - The keeper must be free of `unsafe` code as per repository policy.

use crate::{engine::WasmEngine, error::WasmError};
use kv_store::StoreKey;

/// Trait describing keeper behaviour. Concrete implementations will be provided
/// once the full storage layout is defined.
pub trait Keeper {
    /// Store new WASM code and return an id.
    ///
    /// Corresponds to [`Keeper.StoreCode`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/keeper/keeper.go)
    /// and [`VM.StoreCode`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go).
    /// The engine is expected to validate and cache the module, returning a stable
    /// identifier for later instantiation. Beware that the default `CosmwasmEngine`
    /// truncates the 32 byte module checksum down to a `u64` which can collide.
    fn store_code(&mut self, wasm: &[u8]) -> Result<u64, WasmError>;

    /// Instantiate a contract.
    ///
    /// Mirrors [`Keeper.Instantiate`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/keeper/keeper.go)
    /// and [`VM.Instantiate`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go).
    /// The caller passes a `code_id` previously returned from `store_code` and a
    /// serialized instantiate message. In a real implementation this should also
    /// record metadata (creator, admin) and enforce instantiation permissions.
    fn instantiate(&mut self, code_id: u64, msg: &[u8]) -> Result<Vec<u8>, WasmError>;

    /// Execute a contract call.
    ///
    /// Equivalent to [`Keeper.Execute`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/keeper/keeper.go)
    /// and [`VM.Execute`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go).
    /// It should load the contract instance, provide a writeable storage context
    /// and pass the message to the contract's `execute` entry point.
    fn execute(&mut self, addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, WasmError>;

    /// Run a read-only query against a contract.
    ///
    /// Follows [`Keeper.Query`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/keeper/keeper.go)
    /// and [`VM.Query`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go).
    /// Queries must not mutate state. Engines should prepare a read-only instance
    /// and return the raw binary response from the contract.
    fn query(&self, addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, WasmError>;
}

/// Basic keeper implementation parametrised by a `WasmEngine`.
#[derive(Debug, Clone)]
pub struct WasmKeeper<SK: StoreKey, E> {
    /// Persistent key identifying this module's KV store.
    pub store_key: SK,
    /// Execution engine used to run contracts. In a complete module this would be
    /// paired with references to stores and module parameters.
    pub engine: E,
}

impl<SK: StoreKey, E> WasmKeeper<SK, E> {
    /// Create a new keeper with the given store key and execution engine.
    pub fn new(store_key: SK, engine: E) -> Self {
        Self { store_key, engine }
    }
}

impl<SK: StoreKey, E: WasmEngine> Keeper for WasmKeeper<SK, E> {
    /// Store code by delegating to the engine.
    ///
    /// **Footguns:** the default engine maps a 32 byte checksum to a `u64` id
    /// using truncation (see [`CosmwasmEngine::store_code`]). Production chains
    /// should implement a proper mapping to avoid collisions.
    fn store_code(&mut self, wasm: &[u8]) -> Result<u64, WasmError> {
        self.engine.store_code(wasm).map_err(Into::into)
    }

    /// Instantiate a contract instance via the engine.
    ///
    /// **Note:** real implementations must persist the new contract address,
    /// code id and admin in state. This skeleton merely forwards the call.
    fn instantiate(&mut self, code_id: u64, msg: &[u8]) -> Result<Vec<u8>, WasmError> {
        self.engine.instantiate(code_id, msg).map_err(Into::into)
    }

    /// Execute a contract entry point.
    ///
    /// **Missing behaviour:** gas metering, event recording and access control.
    /// The upstream keeper checks caller permissions and deducts gas before
    /// invoking the VM.
    fn execute(&mut self, addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, WasmError> {
        self.engine.execute(addr, msg).map_err(Into::into)
    }

    /// Query a contract in read-only mode.
    ///
    /// This simply forwards to the engine. Real implementations should validate
    /// the contract address and attach a read-only storage backend. Errors from
    /// the VM are converted into [`WasmError`] for the caller.
    fn query(&self, addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, WasmError> {
        self.engine.query(addr, msg).map_err(Into::into)
    }
}
