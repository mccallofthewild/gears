//! ABCI handler for the CosmWasm module.
//!
//! This component wires the `Keeper` into the Gears application runtime. It
//! implements `ABCIHandler` so that transactions and queries targeting the wasm
//! module are properly routed. The handler delegates message processing to the
//! `Keeper` and manages conversion between protobuf-defined types and the
//! internal representations expected by `cosmwasm_vm`.
//!
//! Key responsibilities:
//! - Dispatch transaction messages (`MsgStoreCode`, `MsgInstantiate`, ...).
//! - Forward queries to the keeper while enforcing read-only access.
//! - Register genesis and block lifecycle hooks if required.
//!
//! Constraints & Security:
//! - Must ensure only valid messages are executed and that gas accounting is
//!   enforced via the provided `TxContext`.
//! - Avoid panics; all failures should be propagated as `TxError` or
//!   `QueryError`.
//! - Queries must not mutate state.
//!
//! Implementation of the message handlers is left for future commits.
use crate::keeper::Keeper;
use gears::application::handlers::node::{ABCIHandler, TxError};
use gears::baseapp::errors::QueryError;
use gears::baseapp::QueryRequest;
use gears::context::query::QueryContext;
use gears::context::tx::TxContext;

/// Placeholder struct implementing `ABCIHandler`.
pub struct WasmABCIHandler<K> {
    pub keeper: K,
}

impl<K> ABCIHandler for WasmABCIHandler<K>
where
    K: Keeper,
{
    fn handle_tx(
        &self,
        _ctx: &mut TxContext,
        _tx: &gears::types::tx::raw::TxWithRaw,
    ) -> Result<(), TxError> {
        // TODO: dispatch messages to keeper
        Ok(())
    }

    fn handle_query(
        &self,
        _ctx: &QueryContext,
        _req: &QueryRequest,
    ) -> Result<Vec<u8>, QueryError> {
        // TODO: dispatch queries to keeper
        Ok(Vec::new())
    }
}
