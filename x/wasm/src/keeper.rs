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

/// Keeper managing wasm bytecode and contract state.
#[derive(Debug, Clone)]
pub struct Keeper<SK, PSK, E>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
{
    #[allow(dead_code)]
    store_key: SK,
    #[allow(dead_code)]
    params: WasmParamsKeeper<PSK>,
    #[allow(dead_code)]
    engine: E,
    _pd: PhantomData<fn() -> SK>,
}

impl<SK, PSK, E> Keeper<SK, PSK, E>
where
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    E: Send + Sync,
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
        _ctx: &mut CTX,
        _sender: &gears::types::address::AccAddress,
        _wasm: &[u8],
        _permission: Option<AccessConfig>,
    ) -> Result<u64, WasmError> {
        // TODO: store bytes, update code index and call the engine
        todo!("store_code not yet implemented")
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
        todo!("execute not yet implemented")
    }

    /// Query a contract's state.
    pub fn query<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        _ctx: &CTX,
        _contract: &gears::types::address::AccAddress,
        _msg: Binary,
    ) -> Result<Binary, WasmError> {
        todo!("query not yet implemented")
    }
}
