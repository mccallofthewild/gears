//! ABCI handler for the CosmWasm module.
//!
//! This wires the [`Keeper`] into the Gears application runtime following the
//! same high level flow as `wasmd`. It dispatches Cosmos SDK style messages to
//! the keeper and exposes a minimal query interface.

use crate::{
    genesis::{init_genesis, GenesisState},
    keeper::{Keeper, CODE_PREFIX, CONTRACT_PREFIX},
    message::{Message, MsgExecuteContract, MsgInstantiateContract, MsgStoreCode},
    types::query::{
        QueryCodeRequest, QueryCodeResponse, QueryCodesRequest, QueryCodesResponse,
        QueryContractInfoRequest, QueryContractInfoResponse, QueryContractsByCodeRequest,
        QueryContractsByCodeResponse, QueryRawContractStateRequest, QueryRawContractStateResponse,
        QuerySmartContractStateRequest, QuerySmartContractStateResponse,
    },
};
use address::AccAddress;
use gears::{
    application::handlers::node::{ABCIHandler, ModuleInfo, TxError},
    baseapp::{errors::QueryError, NullQueryRequest, NullQueryResponse, QueryRequest},
    context::{init::InitContext, query::QueryContext, tx::TxContext},
    extensions::gas::GasResultExt,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};
use serde::Serialize;
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

#[derive(Clone, Debug)]
pub enum WasmQuery {
    Code(QueryCodeRequest),
    Codes(QueryCodesRequest),
    ContractInfo(QueryContractInfoRequest),
    ContractsByCode(QueryContractsByCodeRequest),
    Smart(QuerySmartContractStateRequest),
    Raw(QueryRawContractStateRequest),
}

#[derive(Clone, Debug)]
pub struct WasmNodeQueryRequest {
    pub height: u32,
    pub query: WasmQuery,
}

impl QueryRequest for WasmNodeQueryRequest {
    fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Clone, Debug, Serialize, Query)]
#[serde(untagged)]
#[query(response)]
pub enum WasmNodeQueryResponse {
    Code(QueryCodeResponse),
    Codes(QueryCodesResponse),
    ContractInfo(QueryContractInfoResponse),
    ContractsByCode(QueryContractsByCodeResponse),
    Smart(QuerySmartContractStateResponse),
    Raw(QueryRawContractStateResponse),
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

    type QReq = WasmNodeQueryRequest;

    type QRes = WasmNodeQueryResponse;

    fn typed_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: Self::QReq,
    ) -> Self::QRes {
        let keeper = self.keeper.lock().expect("poisoned mutex");
        match query.query {
            WasmQuery::Code(req) => {
                let store = ctx.kv_store(&keeper.store_key).prefix_store(CODE_PREFIX);
                let wasm = store
                    .get(&req.code_id.to_be_bytes())
                    .unwrap_gas()
                    .unwrap_or_default();
                WasmNodeQueryResponse::Code(QueryCodeResponse {
                    wasm_byte_code: wasm,
                })
            }
            WasmQuery::Codes(_) => {
                let store = ctx.kv_store(&keeper.store_key).prefix_store(CODE_PREFIX);
                let codes = store
                    .into_range(..)
                    .map(|(k, _)| u64::from_be_bytes(k.try_into().unwrap_or([0; 8])))
                    .collect();
                WasmNodeQueryResponse::Codes(QueryCodesResponse { code_ids: codes })
            }
            WasmQuery::ContractInfo(req) => {
                let addr = AccAddress::try_from(req.address).unwrap();
                let store = ctx
                    .kv_store(&keeper.store_key)
                    .prefix_store(CONTRACT_PREFIX);
                let id = store
                    .get(addr.as_ref())
                    .unwrap_gas()
                    .map(|v| u64::from_be_bytes(v.as_slice().try_into().unwrap_or([0; 8])))
                    .unwrap_or(0);
                WasmNodeQueryResponse::ContractInfo(QueryContractInfoResponse { code_id: id })
            }
            WasmQuery::ContractsByCode(req) => {
                let store = ctx
                    .kv_store(&keeper.store_key)
                    .prefix_store(CONTRACT_PREFIX);
                let contracts = store
                    .into_range(..)
                    .filter_map(|(k, v)| {
                        let id = u64::from_be_bytes(v.as_slice().try_into().ok()?);
                        if id == req.code_id {
                            AccAddress::try_from(k).ok().map(|a| a.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                WasmNodeQueryResponse::ContractsByCode(QueryContractsByCodeResponse { contracts })
            }
            WasmQuery::Smart(req) => {
                let addr = AccAddress::try_from(req.address).unwrap();
                let data = keeper.query(ctx, &addr, &req.query_data).unwrap();
                WasmNodeQueryResponse::Smart(QuerySmartContractStateResponse { data })
            }
            WasmQuery::Raw(req) => {
                let addr = AccAddress::try_from(req.address).unwrap();
                let data = keeper.query(ctx, &addr, &req.key).unwrap();
                WasmNodeQueryResponse::Raw(QueryRawContractStateResponse { data })
            }
        }
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
            Message::Instantiate(MsgInstantiateContract {
                sender,
                code_id,
                msg,
            }) => {
                let sender = AccAddress::try_from(sender.clone())
                    .map_err(|e| TxError::new::<MI>(e.to_string(), nz::u16!(1)))?;
                keeper
                    .instantiate(ctx, *code_id, &sender, &[], msg)
                    .map(|_| ())
            }
            Message::Execute(MsgExecuteContract {
                sender,
                contract,
                msg,
            }) => {
                let addr = AccAddress::try_from(contract.clone())
                    .map_err(|e| TxError::new::<MI>(e.to_string(), nz::u16!(1)))?;
                let sender = AccAddress::try_from(sender.clone())
                    .map_err(|e| TxError::new::<MI>(e.to_string(), nz::u16!(1)))?;
                keeper.execute(ctx, &addr, &sender, &[], msg).map(|_| ())
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
