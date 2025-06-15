//! Core keeper for the wasm module.
//!
//! The `Keeper` owns persistent storage keys for contract code and instances and
//! exposes high level methods used by the rest of the application. It is
//! conceptually similar to the keeper defined in `wasmd/x/wasm` but adapted to
//! work with Gears traits and the `WasmEngine` abstraction.
//!
//! Major duties include:
//! - Persisting uploaded contract code and assigning stable identifiers.
//! - Instantiating contracts with configured permissions and tracking their
//!   metadata.
//! - Executing and querying contracts via an associated `WasmEngine` instance.
//! - Managing contract admin updates, code migration and pinning.
//!
//! Constraints & Security:
//! - All state mutations must go through the provided context traits to ensure
//!   atomicity and gas accounting.
//! - Access control should mirror the Cosmos SDK module, preventing unauthorised
//!   calls.
//! - The keeper must be free of `unsafe` code as per repository policy.
use crate::{engine::WasmEngine, error::WasmError};

/// Trait describing keeper behaviour. Concrete implementations will be provided
/// once the full storage layout is defined.
pub trait Keeper {
    /// Store new WASM code and return an id.
    fn store_code(&mut self, wasm: &[u8]) -> Result<u64, WasmError>;

    /// Instantiate a contract.
    fn instantiate(&mut self, code_id: u64, msg: &[u8]) -> Result<Vec<u8>, WasmError>;

    /// Execute a contract call.
    fn execute(&mut self, addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, WasmError>;
}

/// Basic keeper implementation parametrised by a `WasmEngine`.
pub struct WasmKeeper<E> {
    pub engine: E,
}

impl<E: WasmEngine> Keeper for WasmKeeper<E> {
    fn store_code(&mut self, wasm: &[u8]) -> Result<u64, WasmError> {
        self.engine.store_code(wasm).map_err(Into::into)
    }

    fn instantiate(&mut self, code_id: u64, msg: &[u8]) -> Result<Vec<u8>, WasmError> {
        self.engine.instantiate(code_id, msg).map_err(Into::into)
    }

    fn execute(&mut self, addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, WasmError> {
        self.engine.execute(addr, msg).map_err(Into::into)
    }
}
