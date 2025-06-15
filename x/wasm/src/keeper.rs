//! Core keeper for the wasm module.
//!
//! This keeper owns persistent storage keys for contract code and contract
//! instances. Execution of the Wasm bytecode is delegated to a [`WasmEngine`]
//! which mirrors the behaviour of [`wasmvm`](https://github.com/CosmWasm/wasmvm).
//!
//! The structure below is intentionally small but shows how state would be
//! managed in a production implementation. Code and contract info are stored
//! under dedicated prefixes while the engine handles compilation and caching.
//! The API matches the high level calls exposed by `wasmd`.

use crate::{engine::WasmEngine, error::WasmError, params::WasmParamsKeeper};
use address::AccAddress;
use bytes::Bytes;
use gears::{
    context::{QueryableContext, TransactionalContext},
    core::Protobuf,
    gas::store::errors::GasStoreErrors,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
};

/// Prefix under which raw wasm code is stored.
const CODE_PREFIX: [u8; 1] = [0x01];
/// Prefix under which contract info is stored.
const CONTRACT_PREFIX: [u8; 1] = [0x02];
/// Key tracking the next available contract id.
const NEXT_CONTRACT_ID_KEY: [u8; 1] = [0x03];
/// Key tracking the next available code id.
const NEXT_CODE_ID_KEY: [u8; 1] = [0x04];

fn code_key(id: u64) -> Vec<u8> {
    [CODE_PREFIX.to_vec(), id.to_be_bytes().to_vec()].concat()
}

fn contract_key(addr: &AccAddress) -> Vec<u8> {
    [CONTRACT_PREFIX.to_vec(), Vec::<u8>::from(addr.clone())].concat()
}

fn next_contract_id<SK: StoreKey, DB: Database, CTX: TransactionalContext<DB, SK>>(
    sk: &SK,
    ctx: &mut CTX,
) -> Result<u64, GasStoreErrors> {
    let mut store = ctx.kv_store_mut(sk);
    let current = store.get(&NEXT_CONTRACT_ID_KEY)?;
    let id: u64 = match current {
        Some(raw) => u64::decode(Bytes::from(raw)).ok().unwrap_or(0),
        None => 0,
    };
    store.set(NEXT_CONTRACT_ID_KEY, (id + 1).encode_vec())?;
    Ok(id)
}

fn next_code_id<SK: StoreKey, DB: Database, CTX: TransactionalContext<DB, SK>>(
    sk: &SK,
    ctx: &mut CTX,
) -> Result<u64, GasStoreErrors> {
    let mut store = ctx.kv_store_mut(sk);
    let current = store.get(&NEXT_CODE_ID_KEY)?;
    let id: u64 = match current {
        Some(raw) => u64::decode(Bytes::from(raw)).ok().unwrap_or(0),
        None => 0,
    };
    store.set(NEXT_CODE_ID_KEY, (id + 1).encode_vec())?;
    // NOTE: This counter lives in the IAVL store so any forks that rewrite
    // history could cause mismatches between the engine cache and persistent
    // state. Aligning this with `wasmd`'s behaviour requires careful replay of
    // all writes during genesis and upgrades.
    Ok(id)
}

fn contract_address(id: u64) -> Vec<u8> {
    // Contracts are addressed by appending the numeric id to a 12 byte zero
    // prefix which roughly matches the 20 byte address format used in
    // `wasmd`. Any change here must keep the address stable across nodes or
    // previously instantiated contracts will become unreachable.
    let mut out = vec![0u8; 12];
    out.extend_from_slice(&id.to_be_bytes());
    out
}

/// Keeper implementation wrapping a [`WasmEngine`].
#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey, E: WasmEngine> {
    /// Store key used for persisting wasm state.
    pub store_key: SK,
    /// Access to module parameters.
    pub params: WasmParamsKeeper<PSK>,
    /// Execution engine used for running contracts.
    pub engine: E,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, E: WasmEngine> Keeper<SK, PSK, E> {
    /// Create a new keeper instance.
    pub fn new(store_key: SK, params_subspace_key: PSK, engine: E) -> Self {
        Self {
            store_key,
            params: WasmParamsKeeper {
                params_subspace_key,
            },
            engine,
        }
    }

    /// Store new contract code after validation by the engine.
    ///
    /// Equivalent to `StoreCode` in `wasmd`. The wasm bytecode is compiled
    /// via the [`WasmEngine`] and persisted under a sequential identifier.  A
    /// major footgun here is the truncated mapping of checksums to `u64`
    /// identifiers which can lead to collisions if not handled carefully.
    pub fn store_code<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &mut self,
        ctx: &mut CTX,
        wasm: &[u8],
    ) -> Result<u64, WasmError> {
        let params = self.params.try_get(ctx)?;
        if wasm.len() > params.max_wasm_size as usize {
            return Err(WasmError::InvalidCode("wasm too large".into()));
        }
        let id = next_code_id(&self.store_key, ctx)?;
        self.engine.store_code(id, wasm)?;
        ctx.kv_store_mut(&self.store_key)
            .set(code_key(id), wasm.to_vec())?;
        Ok(id)
    }

    /// Instantiate a contract from previously stored code.
    ///
    /// Mirrors the `Instantiate` call in both `wasmd` and the Go `wasmvm`
    /// bindings. A new bech32 address is created by appending the contract id to
    /// a zeroed prefix. Be aware that address construction is chain specific and
    /// must match the expected prefix and checksum format.
    pub fn instantiate<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &mut self,
        ctx: &mut CTX,
        code_id: u64,
        msg: &[u8],
    ) -> Result<(AccAddress, Vec<u8>), WasmError> {
        let addr_id = next_contract_id(&self.store_key, ctx)?;
        let raw_addr = contract_address(addr_id);
        let addr =
            AccAddress::try_from(raw_addr.clone()).map_err(|e| WasmError::Keeper(e.to_string()))?;
        let resp = self.engine.instantiate(code_id, msg)?;
        ctx.kv_store_mut(&self.store_key)
            .set(contract_key(&addr), code_id.encode_vec())?;
        Ok((addr, resp))
    }

    /// Execute a contract call.
    ///
    /// Looks up the contract's code id and delegates execution to the engine.
    /// Currently there is no access control or admin logic implemented which is
    /// a deviation from `wasmd`'s production behaviour.
    pub fn execute<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &mut self,
        ctx: &mut CTX,
        addr: &AccAddress,
        msg: &[u8],
    ) -> Result<Vec<u8>, WasmError> {
        // ensure the contract exists
        if ctx
            .kv_store(&self.store_key)
            .get(&contract_key(addr))?
            .is_none()
        {
            return Err(WasmError::Keeper("unknown contract".into()));
        }
        self.engine.execute(addr.as_ref(), msg).map_err(Into::into)
    }

    /// Run a read-only query against a contract.
    ///
    /// Queries pass through the engine in read-only mode. The caller must
    /// ensure the contract exists otherwise a generic keeper error is returned.
    pub fn query<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &AccAddress,
        msg: &[u8],
    ) -> Result<Vec<u8>, WasmError> {
        if ctx
            .kv_store(&self.store_key)
            .get(&contract_key(addr))?
            .is_none()
        {
            return Err(WasmError::Keeper("unknown contract".into()));
        }
        self.engine.query(addr.as_ref(), msg).map_err(Into::into)
    }
}
