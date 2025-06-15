//! CosmWasm execution engine and trait definitions.
//!
//! This module defines the [`WasmEngine`] trait which abstracts over the
//! underlying execution backend for smart contracts. The default implementation
//! uses `cosmwasm_vm` and mirrors the high level API provided by the Go
//! [`wasmvm`](https://github.com/CosmWasm/wasmvm) `VM` struct. It exposes
//! operations for storing code, instantiating contracts, executing calls,
//! migrating, querying and handling IBC events.
//!
//! Responsibilities:
//! - Manage a `cosmwasm_vm::Cache` of compiled WASM binaries.
//! - Provide functions corresponding to the `wasmvm` API such as `store_code`,
//!   `instantiate`, `execute`, `query` and the IBC callbacks.
//! - Bridge the VM's `Backend` trait with concrete storage, API and querier
//!   implementations defined elsewhere in this crate.
//!
//! Constraints & Security:
//! - All execution must be deterministic and respect the gas limits supplied via
//!   Gears' gas meter integration. No unsafe code is allowed.
//! - Persistent caches must be pinned/unpinned carefully to avoid leaking memory
//!   or compiling untrusted code more than once.
//! - Inputs should be validated to avoid injection of malformed WASM modules.
//!
//! This file only defines the trait and a skeletal struct. Implementation will
//! follow the design laid out in `COSMWASM_ADR.md` and `COSMWASM_PRD.md`.
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    to_json_vec,
};
use cosmwasm_vm::{
    backend::{Backend, BackendApi, Querier, Storage},
    cache::{Cache, CacheOptions},
    calls::{call_execute_raw, call_instantiate_raw, call_query_raw},
    checksum::Checksum,
    instance::InstanceOptions,
    VmError, VmResult,
};
use std::{collections::HashMap, convert::TryInto};

/// High level interface used by the `Keeper` to execute contracts.
///
/// These calls map closely to the public API exposed by the Go
/// [`wasmvm`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go)
/// [`VM`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go#L17-L24)
/// type as well as the helper functions in
/// [`cosmwasm_vm::calls`](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/calls.rs).
pub trait WasmEngine {
    /// Stores new contract code under a user supplied identifier.
    ///
    /// In `wasmd` the next `code_id` is tracked in the keeper and provided to
    /// the wasm VM when uploading code. To mirror that behaviour the caller
    /// passes the desired `code_id` here and the engine persists the compiled
    /// module in its cache.
    fn store_code(&mut self, code_id: u64, wasm: &[u8]) -> Result<(), VmError>;

    /// Instantiates a contract from previously stored code.
    ///
    /// Modeled after [`VM.Instantiate`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go#L147-L188).
    /// The engine must load the module referenced by `code_id` from the cache,
    /// create an [`Instance`](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/instance.rs)
    /// with a concrete backend and call `instantiate` with the provided message.
    /// The call returns the raw binary response from the contract.
    fn instantiate(&mut self, code_id: u64, msg: &[u8]) -> Result<Vec<u8>, VmError>;

    /// Executes a contract call.
    ///
    /// Mirrors [`VM.Execute`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go#L192-L236)
    /// and [`call_execute`](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/calls.rs#L126-L144).
    /// It should prepare an execution environment for the given contract
    /// address, load the instance from the cache and invoke the `execute`
    /// export. The return value is the serialized contract response.
    fn execute(&mut self, contract_addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, VmError>;

    /// Runs a read-only query against a contract.
    ///
    /// Follows [`VM.Query`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go#L238-L278)
    /// and [`call_query`](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/calls.rs#L354-L375).
    /// Queries must not mutate state so the instance should be configured in
    /// read-only mode. The returned bytes are expected to be JSON data.
    fn query(&self, contract_addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, VmError>;
}

/// Placeholder engine using `cosmwasm_vm` directly.
///
/// The design follows the [`Cache`](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/cache.rs)
/// and [`Instance`](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/instance.rs)
/// abstractions from the upstream VM. The goal is to mirror the behaviour of
/// the Go `VM` wrapper in pure Rust.
pub struct CosmwasmEngine<A, S, Q>
where
    A: BackendApi + Default + 'static,
    S: Storage + Default + 'static,
    Q: Querier + Default + 'static,
{
    /// Cache of compiled modules.
    pub cache: Cache<A, S, Q>,
    /// Mapping from numeric IDs to checksums.
    code_map: HashMap<u64, Checksum>,
}

fn id_from_contract_addr(addr: &[u8]) -> Option<u64> {
    if addr.len() < 8 {
        return None;
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&addr[addr.len() - 8..]);
    Some(u64::from_be_bytes(bytes))
}

impl<A, S, Q> CosmwasmEngine<A, S, Q>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    /// Create a new engine with the given cache options.
    ///
    /// This mirrors [`NewVMWithConfig`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go#L36-L48)
    /// where the Go bindings initialise an FFI cache. The call is `unsafe` here
    /// because [`Cache::new`](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/cache.rs)
    /// relies on the caller to ensure the given directory is exclusively used by
    /// one process at a time.
    pub unsafe fn new(options: CacheOptions) -> VmResult<Self> {
        Ok(Self {
            cache: Cache::new(options)?,
            code_map: HashMap::new(),
        })
    }
}

impl<A, S, Q> WasmEngine for CosmwasmEngine<A, S, Q>
where
    A: BackendApi + Default + 'static,
    S: Storage + Default + 'static,
    Q: Querier + Default + 'static,
{
    fn store_code(&mut self, code_id: u64, wasm: &[u8]) -> Result<(), VmError> {
        // Delegate to `Cache::store_code` which performs wasm validation and
        // compilation. This mirrors the behaviour of `VM.StoreCode` in the Go
        // bindings.
        let checksum = self.cache.store_code(wasm, true, true)?;

        // Associate the supplied code id with the produced checksum so we can
        // retrieve it on instantiation. The caller ensures the id is unique.
        self.code_map.insert(code_id, checksum);
        Ok(())
    }

    fn instantiate(&mut self, code_id: u64, msg: &[u8]) -> Result<Vec<u8>, VmError> {
        let checksum = self
            .code_map
            .get(&code_id)
            .ok_or_else(|| VmError::cache_err("unknown code id"))?;

        let backend = Backend {
            api: A::default(),
            storage: S::default(),
            querier: Q::default(),
        };

        let options = InstanceOptions {
            gas_limit: u64::MAX,
            print_debug: false,
        };

        let mut instance = self.cache.get_instance(checksum, backend, options)?;

        let env = to_json_vec(&mock_env()).map_err(|e| VmError::serialize_err("Env", e))?;
        let info = to_json_vec(&mock_info("creator", &[]))
            .map_err(|e| VmError::serialize_err("Info", e))?;

        call_instantiate_raw(&mut instance, &env, &info, msg)
    }

    fn execute(&mut self, contract_addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, VmError> {
        let contract_id = id_from_contract_addr(contract_addr)
            .ok_or_else(|| VmError::cache_err("malformed contract address"))?;
        let checksum = self
            .code_map
            .get(&contract_id)
            .ok_or_else(|| VmError::cache_err("unknown contract"))?;

        let backend = Backend {
            api: A::default(),
            storage: S::default(),
            querier: Q::default(),
        };
        let options = InstanceOptions {
            gas_limit: u64::MAX,
            print_debug: false,
        };
        let mut instance = self.cache.get_instance(checksum, backend, options)?;
        let env = to_json_vec(&mock_env()).map_err(|e| VmError::serialize_err("Env", e))?;
        let info = to_json_vec(&mock_info("caller", &[]))
            .map_err(|e| VmError::serialize_err("Info", e))?;
        call_execute_raw(&mut instance, &env, &info, msg)
    }

    fn query(&self, contract_addr: &[u8], msg: &[u8]) -> Result<Vec<u8>, VmError> {
        let contract_id = id_from_contract_addr(contract_addr)
            .ok_or_else(|| VmError::cache_err("malformed contract address"))?;
        let checksum = self
            .code_map
            .get(&contract_id)
            .ok_or_else(|| VmError::cache_err("unknown contract"))?;

        let backend = Backend {
            api: A::default(),
            storage: S::default(),
            querier: Q::default(),
        };
        let options = InstanceOptions {
            gas_limit: u64::MAX,
            print_debug: false,
        };
        let mut instance = self.cache.get_instance(checksum, backend, options)?;
        let env = to_json_vec(&mock_env()).map_err(|e| VmError::serialize_err("Env", e))?;
        call_query_raw(&mut instance, &env, msg)
    }
}
