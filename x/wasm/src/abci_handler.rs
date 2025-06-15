//! ABCI handler for the CosmWasm module.
//!
//! This wires the [`Keeper`] into the Gears application runtime following the
//! same high level flow as `wasmd`. It dispatches Cosmos SDK style messages to
//! the keeper and exposes a minimal query interface.

use crate::{
    genesis::{init_genesis, GenesisState},
    keeper::Keeper,
    message::{Message, MsgExecuteContract, MsgInstantiateContract, MsgStoreCode},
};
use address::AccAddress;
use gears::{
    application::handlers::node::{ABCIHandler, ModuleInfo, TxError},
    baseapp::{errors::QueryError, NullQueryRequest, NullQueryResponse, QueryRequest},
    context::{init::InitContext, query::QueryContext, tx::TxContext},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};
use std::{marker::PhantomData, sync::Mutex};

/// Handler wrapping a [`Keeper`] inside a `Mutex` so mutable access can be
/// shared across the ABCI lifecycle hooks.
#[derive(Debug)]
pub struct WasmABCIHandler<SK, PSK, E, MI>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
{
    keeper: Mutex<Keeper<SK, PSK, E>>,
    _marker: PhantomData<MI>,
}

impl<SK, PSK, E, MI> WasmABCIHandler<SK, PSK, E, MI>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
{
    pub fn new(keeper: Keeper<SK, PSK, E>) -> Self {
        Self {
            keeper: Mutex::new(keeper),
            _marker: PhantomData,
        }
    }
}

impl<SK, PSK, E, MI> ABCIHandler for WasmABCIHandler<SK, PSK, E, MI>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    E: crate::engine::WasmEngine + Send + 'static,
    MI: ModuleInfo,
{
    type Message = Message;

    type Genesis = GenesisState;

    type StoreKey = SK;

    type QReq = NullQueryRequest;

    type QRes = NullQueryResponse;

    fn typed_query<DB: Database>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        unreachable!("typed queries not implemented for wasm module")
    }

    fn run_ante_checks<DB: Database>(
        &self,
        _ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        _tx: &gears::types::tx::raw::TxWithRaw<Self::Message>,
        _is_check: bool,
    ) -> Result<(), TxError> {
        Ok(())
    }

    fn msg<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        msg: &Self::Message,
    ) -> Result<(), TxError> {
        let mut keeper = self.keeper.lock().expect("poisoned mutex");
        let result = match msg {
            Message::StoreCode(MsgStoreCode { wasm_byte_code, .. }) => {
                keeper.store_code(ctx, wasm_byte_code).map(|_| ())
            }
            Message::Instantiate(MsgInstantiateContract { code_id, msg, .. }) => {
                keeper.instantiate(ctx, *code_id, msg).map(|_| ())
            }
            Message::Execute(MsgExecuteContract { contract, msg, .. }) => {
                let addr = AccAddress::try_from(contract.clone())
                    .map_err(|e| TxError::new::<MI>(e.to_string(), nz::u16!(1)))?;
                keeper.execute(ctx, &addr, msg).map(|_| ())
            }
        };

        result.map_err(|e| e.into::<MI>())
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        let mut keeper = self.keeper.lock().expect("poisoned mutex");
        if let Err(e) = init_genesis(ctx, &mut *keeper, genesis) {
            panic!("wasm genesis failed: {e}");
        }
        Vec::new()
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: QueryRequest,
    ) -> Result<Vec<u8>, QueryError> {
        Err(QueryError::PathNotFound)
    }
}
