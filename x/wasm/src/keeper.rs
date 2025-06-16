use std::marker::PhantomData;

use cosmwasm_std::{Binary, MessageInfo, Response};
use gears::{
    application::keepers::params::ParamsKeeper,
    context::{QueryableContext, TransactionalContext},
    gas::store::errors::GasStoreErrors,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

use crate::{
    error::WasmError,
    message::AccessConfig,
    params::{Params, WasmParamsKeeper},
};

/// Prefixes used for deriving store keys. These mirror the layout in `wasmd` so
/// that data from existing chains can be reused directly.
#[allow(dead_code)]
const CODE_STORE_PREFIX: [u8; 1] = [0x01];
#[allow(dead_code)]
const CONTRACT_STORE_PREFIX: [u8; 1] = [0x02];
#[allow(dead_code)]
const SEQUENCE_STORE_PREFIX: [u8; 1] = [0x03];
#[allow(dead_code)]
const CODE_INDEX_PREFIX: [u8; 1] = [0x04];

const KEY_SEQ_CODE_ID: &[u8] = b"lastCodeId";
#[allow(dead_code)]
const KEY_SEQ_CONTRACT_ID: &[u8] = b"lastContractId";

/// Return the key under which contract code is stored.
#[allow(dead_code)]
fn code_key(id: u64) -> Vec<u8> {
    [CODE_STORE_PREFIX.as_slice(), &id.to_be_bytes()].concat()
}

/// Return the key for contract metadata associated with `addr`.
#[allow(dead_code)]
fn contract_key(addr: &gears::types::address::AccAddress) -> Vec<u8> {
    [
        CONTRACT_STORE_PREFIX.as_slice(),
        &[addr.as_ref().len() as u8],
        addr.as_ref(),
    ]
    .concat()
}

/// Derive the storage key for a sequence counter.
#[allow(dead_code)]
fn sequence_key(name: &[u8]) -> Vec<u8> {
    [SEQUENCE_STORE_PREFIX.as_slice(), name].concat()
}

fn next_sequence<DB: Database, SKT: StoreKey, CTX: TransactionalContext<DB, SKT>>(
    ctx: &mut CTX,
    store_key: &SKT,
    name: &[u8],
) -> Result<u64, GasStoreErrors> {
    let mut store = ctx.kv_store_mut(store_key);
    let key = sequence_key(name);
    let current = store
        .get(&key)?
        .and_then(|v| v.as_slice().try_into().ok())
        .map(u64::from_be_bytes)
        .unwrap_or(0);
    let next = current + 1;
    store.set(key, next.to_be_bytes())?;
    Ok(next)
}

/// Keeper managing wasm bytecode and contract state.
#[derive(Debug, Clone)]
pub struct Keeper<SK, PSK, E, A, S, Q>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    A: cosmwasm_vm::BackendApi,
    S: cosmwasm_vm::Storage,
    Q: cosmwasm_vm::Querier,
    E: crate::engine::WasmEngine<A, S, Q>,
{
    #[allow(dead_code)]
    store_key: SK,
    #[allow(dead_code)]
    params: WasmParamsKeeper<PSK>,
    #[allow(dead_code)]
    engine: E,
    _pd: PhantomData<fn() -> (SK, A, S, Q)>,
}

impl<SK, PSK, E, A, S, Q> Keeper<SK, PSK, E, A, S, Q>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    A: cosmwasm_vm::BackendApi,
    S: cosmwasm_vm::Storage,
    Q: cosmwasm_vm::Querier,
    E: crate::engine::WasmEngine<A, S, Q> + Send + Sync,
{
    /// Create a new keeper instance bound to a store key and execution engine.
    pub fn new(store_key: SK, params_subspace_key: PSK, engine: E) -> Self {
        Self {
            store_key,
            params: WasmParamsKeeper {
                params_subspace_key,
            },
            engine,
            _pd: PhantomData,
        }
    }

    /// Retrieve current module parameters.
    pub fn params<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Params, GasStoreErrors> {
        self.params.try_get(ctx)
    }

    /// Persist new parameters and notify the engine.
    pub fn set_params<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: Params,
    ) -> Result<(), GasStoreErrors> {
        let _old = self.params.try_get(ctx)?;
        self.params.try_set(ctx, params)?;
        Ok(())
    }

    /// Store compiled code and return its numeric identifier.
    pub fn store_code<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender: &gears::types::address::AccAddress,
        wasm: &[u8],
        permission: Option<AccessConfig>,
    ) -> Result<u64, WasmError> {
        // <COSMWASM_PROGRESS.md#L56-L62>
        // ensure uploaded code size respects the current parameter limits
        let params = self.params.try_get(ctx).map_err(|e| WasmError::Internal {
            reason: e.to_string(),
        })?;
        if wasm.len() as u64 > params.max_contract_size {
            return Err(WasmError::InvalidRequest {
                reason: "wasm bytecode too large".into(),
            });
        }

        // persist the wasm via the execution engine to obtain its checksum
        let checksum = self.engine.store_code(wasm)?;
        // analyze code so we can record required capabilities (ignored result)
        let _ = self.engine.analyze_code(&checksum);

        // reserve a new code id and store metadata
        let code_id = next_sequence(ctx, &self.store_key, KEY_SEQ_CODE_ID).map_err(|e| {
            WasmError::Internal {
                reason: e.to_string(),
            }
        })?;

        // determine instantiate permissions
        let instantiate_cfg = permission.unwrap_or(AccessConfig {
            permission: params.instantiate_default_permission,
            addresses: Vec::new(),
        });

        let info = cosmos_sdk_proto::cosmwasm::wasm::v1::CodeInfo {
            code_hash: Vec::from(checksum),
            creator: sender.to_string(),
            instantiate_config: Some(instantiate_cfg.into()),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&info, &mut buf).expect("encode CodeInfo");
        let mut store = ctx.kv_store_mut(&self.store_key);
        store
            .set(code_key(code_id), buf)
            .map_err(|e| WasmError::Internal {
                reason: e.to_string(),
            })?;

        Ok(code_id)
    }

    /// Instantiate a stored contract.
    #[allow(clippy::too_many_arguments)]
    pub fn instantiate<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _code_id: u64,
        _creator: &gears::types::address::AccAddress,
        _admin: Option<gears::types::address::AccAddress>,
        _label: String,
        _msg: Binary,
        _funds: gears::types::base::coins::UnsignedCoins,
    ) -> Result<gears::types::address::AccAddress, WasmError> {
        // <COSMWASM_PROGRESS.md#L56-L62>
        todo!("instantiate not yet implemented")
    }

    /// Execute a contract method.
    pub fn execute<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        _ctx: &mut CTX,
        _contract: &gears::types::address::AccAddress,
        _info: MessageInfo,
        _msg: Binary,
        _funds: gears::types::base::coins::UnsignedCoins,
    ) -> Result<Response, WasmError> {
        // <COSMWASM_PROGRESS.md#L56-L62>
        todo!("execute not yet implemented")
    }

    /// Query a contract's state.
    pub fn query<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        _ctx: &CTX,
        _contract: &gears::types::address::AccAddress,
        _msg: Binary,
    ) -> Result<Binary, WasmError> {
        // <COSMWASM_PROGRESS.md#L56-L62>
        todo!("query not yet implemented")
    }
}
