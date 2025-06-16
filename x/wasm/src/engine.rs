use std::{collections::HashSet, path::PathBuf, sync::RwLock};

use cosmwasm_std::{Binary, Env, MessageInfo, Response};
use cosmwasm_vm::{cache::{Cache, CacheOptions, Size}, BackendApi, Querier, Storage};

use crate::{error::WasmError, params::Params};

/// Runtime configuration for the [`CosmwasmEngine`].
#[derive(Debug, Clone)]
pub struct EngineOptions {
    /// Directory where compiled modules and caches are stored.
    pub base_dir: PathBuf,
    /// Capabilities advertised to contracts during static analysis.
    pub capabilities: HashSet<String>,
    /// Maximum memory available to a contract instance in bytes.
    pub instance_memory_limit: u32,
    /// Number of contracts kept in the in‑memory cache.
    pub memory_cache_size: u32,
    /// Print contract debug logs to stdout when enabled.
    pub debug: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("wasm_cache"),
            capabilities: HashSet::new(),
            instance_memory_limit: 32, // MiB
            memory_cache_size: 10,     // MiB
            debug: false,
        }
    }
}

/// Abstraction over the CosmWasm execution engine.
///
/// Methods mirror the Go `wasmvm` API so that the keeper can
/// be implemented without depending on specific engine details.
pub trait WasmEngine<A: BackendApi, S: Storage, Q: Querier>: Send + Sync {
    /// Store validated wasm bytecode and return its checksum.
    fn store_code(&self, wasm: &[u8]) -> Result<cosmwasm_vm::Checksum, WasmError>;

    /// Run static analysis on previously stored code.
    fn analyze_code(&self, checksum: &cosmwasm_vm::Checksum) -> Result<cosmwasm_vm::AnalysisReport, WasmError>;

    /// Notify the engine that module parameters have changed.
    fn on_params_change(&self, old: &Params, new: &Params) -> Result<(), WasmError>;

    /// Instantiate a contract. Full execution support will be added later.
    fn instantiate(
        &self,
        _checksum: &cosmwasm_vm::Checksum,
        _env: Env,
        _info: MessageInfo,
        _msg: Binary,
        _store: &mut S,
        _api: A,
        _querier: Q,
        _gas_limit: u64,
    ) -> Result<Response, WasmError> {
        todo!("instantiate not yet implemented")
    }

    /// Execute a contract.
    fn execute(
        &self,
        _checksum: &cosmwasm_vm::Checksum,
        _env: Env,
        _info: MessageInfo,
        _msg: Binary,
        _store: &mut S,
        _api: A,
        _querier: Q,
        _gas_limit: u64,
    ) -> Result<Response, WasmError> {
        todo!("execute not yet implemented")
    }

    /// Query a contract.
    fn query(
        &self,
        _checksum: &cosmwasm_vm::Checksum,
        _env: Env,
        _msg: Binary,
        _store: &S,
        _api: A,
        _querier: Q,
        _gas_limit: u64,
    ) -> Result<Binary, WasmError> {
        todo!("query not yet implemented")
    }
}

/// Default engine based on [`cosmwasm_vm`].
pub struct CosmwasmEngine<A: BackendApi, S: Storage, Q: Querier> {
    cache: Cache<A, S, Q>,
    options: RwLock<EngineOptions>,
}

impl<A, S, Q> CosmwasmEngine<A, S, Q>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    /// Create a new engine with the provided options.
    pub fn new(options: EngineOptions) -> Result<Self, WasmError> {
        let cache_opts = CacheOptions::new(
            &options.base_dir,
            options.capabilities.clone(),
            Size::mebi(options.memory_cache_size as usize),
            Size::mebi(options.instance_memory_limit as usize),
        );
        // Safety: directory is created if missing and considered trusted similar
        // to `wasmvm`'s InitCache.
        let cache = unsafe { Cache::<A, S, Q>::new(cache_opts) }?;
        Ok(Self {
            cache,
            options: RwLock::new(options),
        })
    }
}

impl<A, S, Q> WasmEngine<A, S, Q> for CosmwasmEngine<A, S, Q>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    fn store_code(&self, wasm: &[u8]) -> Result<cosmwasm_vm::Checksum, WasmError> {
        self.cache
            .store_code(wasm, true, true)
            .map_err(WasmError::from)
    }

    fn analyze_code(&self, checksum: &cosmwasm_vm::Checksum) -> Result<cosmwasm_vm::AnalysisReport, WasmError> {
        self.cache.analyze(checksum).map_err(WasmError::from)
    }

    fn on_params_change(&self, _old: &Params, new: &Params) -> Result<(), WasmError> {
        let mut opts = self.options.write().unwrap();
        opts.memory_cache_size = new.memory_cache_size;
        opts.instance_memory_limit = new.max_contract_size as u32;
        Ok(())
    }
}


/// Example
/// ```
/// let opts = EngineOptions::default();
/// let engine: CosmwasmEngine<MyApi, MyStorage, MyQuerier> = CosmwasmEngine::new(opts).unwrap();
/// ```

