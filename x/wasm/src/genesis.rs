//! Genesis state management for the wasm module.
//!
//! Handles initial loading of contract code and metadata from the genesis file
//! as well as exporting the current state during chain upgrades. The structure
//! mirrors the approach used by `wasmd` where code and contract info are stored
//! in dedicated sub-stores keyed by identifiers.
//!
//! Responsibilities:
//! - Define `GenesisState` structures serialisable via `serde`/protobuf.
//! - Provide `init_genesis` and `export_genesis` functions consumed by the
//!   application during startup and export.
//! - Validate any embedded contract code for safety before storing.
//!
//! Security considerations:
//! - Genesis contracts may contain malicious code. Validation should include
//!   checksums and compilation before execution is allowed.
//! - Panic-free error handling is required to avoid halting the node on corrupt
//!   genesis files.
use serde::{Deserialize, Serialize};

/// Structure representing wasm module genesis data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisState {
    /// Placeholder for stored contract code and metadata.
    pub codes: Vec<Vec<u8>>, // TODO: proper structures
}

/// Initialise module state from genesis data.
pub fn init_genesis<S>(_state: &GenesisState, _keeper: &mut S) {
    // TODO: load codes into keeper
}

/// Export current module state to genesis format.
pub fn export_genesis<S>(_keeper: &S) -> GenesisState {
    // TODO: read codes from keeper
    GenesisState { codes: Vec::new() }
}
