//! Genesis state management for the wasm module.
//!
//! Handles initial loading of contract code and metadata from the genesis file
//! as well as exporting the current state during chain upgrades.  The layout is
//! inspired by [`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm)
//! where code and contract info are stored under dedicated key prefixes and a
//! sequence counter tracks the next free identifier.  The interface mirrors the
//! behaviour of the Go [`wasmvm`](https://github.com/CosmWasm/wasmvm) bindings
//! which expect the engine to receive pre‐allocated code IDs.
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
use gears::baseapp::genesis::Genesis;
use gears::{
    context::{init::InitContext, query::QueryContext},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};
use serde::{Deserialize, Serialize};

use crate::{engine::WasmEngine, error::WasmError, keeper::Keeper};
use std::convert::TryInto;

/// Structure representing wasm module genesis data.
///
/// This mirrors [`GenesisState`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/types/genesis.go)
/// from `wasmd` albeit heavily simplified. Only raw code bytes and the next
/// sequence number are tracked here. A complete implementation would include
/// contract metadata, histories and pinned code checksums.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GenesisState {
    /// Raw WASM binaries to load at genesis. The order is preserved so that
    /// subsequent contract instantiations can reference the assigned `code_id`.
    pub codes: Vec<Vec<u8>>,
    /// Sequence value for generating the next `code_id` when new code is
    /// uploaded.  This mirrors the `Sequence` entries in wasmd genesis files.
    pub next_code_id: u64,
}

impl Genesis for GenesisState {}

/// Initialise module state from genesis data.
///
/// Each WASM blob is passed to [`Keeper::store_code`] which performs basic
/// validation and persists the bytes under a new code identifier. After all
/// code is loaded the sequence counter is set to the provided `next_code_id` so
/// further uploads continue from that value.
pub fn init_genesis<SK, PSK, E, DB>(
    ctx: &mut InitContext<'_, DB, SK>,
    keeper: &mut Keeper<SK, PSK, E>,
    genesis: GenesisState,
) -> Result<(), WasmError>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    E: WasmEngine,
    DB: Database,
{
    // load all provided wasm blobs via the keeper. This mirrors the
    // behaviour of `wasmd` where code is prevalidated and pinned during chain
    // initialisation.
    for wasm in genesis.codes {
        keeper.store_code(ctx, &wasm)?;
    }

    ctx.kv_store_mut(&keeper.store_key).set(
        crate::keeper::NEXT_CODE_ID_KEY,
        genesis.next_code_id.to_be_bytes().to_vec(),
    )?;

    Ok(())
}

/// Export current module state to genesis format.
///
/// The keeper's store is scanned for all code entries which are returned
/// alongside the next sequence value.
pub fn export_genesis<SK, PSK, E, DB>(
    ctx: &QueryContext<DB, SK>,
    keeper: &Keeper<SK, PSK, E>,
) -> GenesisState
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    E: WasmEngine,
    DB: Database,
{
    // replicate the scanning logic from `wasmd`'s genesis export. Each
    // stored code blob is emitted in order for later replay.
    let store = ctx.kv_store(&keeper.store_key);
    let code_store = store.prefix_store(crate::keeper::CODE_PREFIX);
    let codes: Vec<Vec<u8>> = code_store
        .into_range(..)
        .map(|(_, v)| v.into_owned())
        .collect();

    let next = store
        .get(&crate::keeper::NEXT_CODE_ID_KEY)
        .map(|v| u64::from_be_bytes(v.as_slice().try_into().unwrap_or([0; 8])))
        .unwrap_or(0);

    GenesisState {
        codes,
        next_code_id: next,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_genesis() {
        let data = r#"{
            "codes": ["aGVsbG8="],
            "next_code_id": 1
        }"#;

        let state: GenesisState = serde_json::from_str(data).expect("valid json");
        assert_eq!(state.codes.len(), 1);
        assert_eq!(state.next_code_id, 1);
    }
}
