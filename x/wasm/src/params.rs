// Parameter definitions for the CosmWasm module.
//
// Mirrors the behaviour of `wasmd` by exposing upload permissions and default
// instantiation access. Additional fields like `query_gas_limit` and
// `memory_cache_size` provide runtime tuning knobs for the `WasmEngine`.

use serde::{Deserialize, Serialize};
use gears::{
    application::keepers::params::ParamsKeeper,
    core::{errors::CoreError, Protobuf},
    params::{ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
};

use crate::message::{AccessConfig, AccessType};

/// String constants used when storing parameters in the params subspace.
const KEY_CODE_UPLOAD_ACCESS: &str = "code_upload_access";
const KEY_INSTANTIATE_DEFAULT_PERMISSION: &str = "instantiate_default_permission";
const KEY_MAX_CONTRACT_SIZE: &str = "max_contract_size";
const KEY_QUERY_GAS_LIMIT: &str = "query_gas_limit";
const KEY_MEMORY_CACHE_SIZE: &str = "memory_cache_size";

/// Module parameters controlling wasm behaviour.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Params {
    pub code_upload_access: AccessConfig,
    pub instantiate_default_permission: AccessType,
    /// Maximum allowed size of uploaded contract code in bytes.
    pub max_contract_size: u64,
    /// Gas limit applied to smart queries executed via ABCI.
    pub query_gas_limit: u64,
    /// Number of compiled contracts cached in memory.
    pub memory_cache_size: u32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            code_upload_access: AccessConfig {
                permission: AccessType::Everybody,
                addresses: Vec::new(),
            },
            instantiate_default_permission: AccessType::Everybody,
            max_contract_size: 1_000_000,
            query_gas_limit: 3_000_000,
            memory_cache_size: 40,
        }
    }
}

impl From<Params> for cosmos_sdk_proto::cosmwasm::wasm::v1::Params {
    fn from(value: Params) -> Self {
        Self {
            code_upload_access: Some(value.code_upload_access.into()),
            instantiate_default_permission: value.instantiate_default_permission as i32,
        }
    }
}

impl TryFrom<cosmos_sdk_proto::cosmwasm::wasm::v1::Params> for Params {
    type Error = CoreError;

    fn try_from(value: cosmos_sdk_proto::cosmwasm::wasm::v1::Params) -> Result<Self, Self::Error> {
        let code_upload_access = match value.code_upload_access {
            Some(cfg) => cfg.try_into()?,
            None => AccessConfig {
                permission: AccessType::Unspecified,
                addresses: Vec::new(),
            },
        };
        let instantiate_default_permission = match value.instantiate_default_permission {
            1 => AccessType::Nobody,
            3 => AccessType::Everybody,
            4 => AccessType::AnyOfAddresses,
            _ => AccessType::Unspecified,
        };
        Ok(Params {
            code_upload_access,
            instantiate_default_permission,
            ..Default::default()
        })
    }
}

impl Protobuf<cosmos_sdk_proto::cosmwasm::wasm::v1::Params> for Params {}

impl ParamsSerialize for Params {
    fn keys() -> std::collections::HashSet<&'static str> {
        [
            KEY_CODE_UPLOAD_ACCESS,
            KEY_INSTANTIATE_DEFAULT_PERMISSION,
            KEY_MAX_CONTRACT_SIZE,
            KEY_QUERY_GAS_LIMIT,
            KEY_MEMORY_CACHE_SIZE,
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (
                KEY_CODE_UPLOAD_ACCESS,
                serde_json::to_vec(&self.code_upload_access).expect("serialize"),
            ),
            (
                KEY_INSTANTIATE_DEFAULT_PERMISSION,
                (self.instantiate_default_permission as i32).to_string().into_bytes(),
            ),
            (
                KEY_MAX_CONTRACT_SIZE,
                self.max_contract_size.to_string().into_bytes(),
            ),
            (
                KEY_QUERY_GAS_LIMIT,
                self.query_gas_limit.to_string().into_bytes(),
            ),
            (
                KEY_MEMORY_CACHE_SIZE,
                self.memory_cache_size.to_string().into_bytes(),
            ),
        ]
    }
}

impl ParamsDeserialize for Params {
    fn from_raw(mut fields: std::collections::HashMap<&'static str, Vec<u8>>) -> Self {
        let code_upload_access: AccessConfig = serde_json::from_slice(
            fields.remove(KEY_CODE_UPLOAD_ACCESS).unwrap_or_default().as_slice(),
        )
        .unwrap_or_default();
        let instantiate_default_permission = ParamKind::U64
            .parse_param(fields.remove(KEY_INSTANTIATE_DEFAULT_PERMISSION).unwrap_or_default())
            .unsigned_64()
            .unwrap_or(0) as i32;
        let instantiate_default_permission = match instantiate_default_permission {
            1 => AccessType::Nobody,
            3 => AccessType::Everybody,
            4 => AccessType::AnyOfAddresses,
            _ => AccessType::Unspecified,
        };
        let max_contract_size = ParamKind::U64
            .parse_param(fields.remove(KEY_MAX_CONTRACT_SIZE).unwrap_or_default())
            .unsigned_64()
            .unwrap_or(1_000_000);
        let query_gas_limit = ParamKind::U64
            .parse_param(fields.remove(KEY_QUERY_GAS_LIMIT).unwrap_or_default())
            .unsigned_64()
            .unwrap_or(3_000_000);
        let memory_cache_size = ParamKind::U64
            .parse_param(fields.remove(KEY_MEMORY_CACHE_SIZE).unwrap_or_default())
            .unsigned_64()
            .unwrap_or(40) as u32;
        Params {
            code_upload_access,
            instantiate_default_permission,
            max_contract_size,
            query_gas_limit,
            memory_cache_size,
        }
    }
}

/// Keeper managing wasm module parameters stored in a subspace.
#[derive(Debug, Clone)]
pub struct WasmParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for WasmParamsKeeper<PSK> {
    type Param = Params;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    #[cfg(feature = "governance")]
    fn validate(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> bool {
        match std::str::from_utf8(key.as_ref()).unwrap_or_default() {
            KEY_CODE_UPLOAD_ACCESS => serde_json::from_slice::<AccessConfig>(value.as_ref()).is_ok(),
            KEY_INSTANTIATE_DEFAULT_PERMISSION
            | KEY_MAX_CONTRACT_SIZE
            | KEY_QUERY_GAS_LIMIT
            | KEY_MEMORY_CACHE_SIZE => ParamKind::U64
                .parse_param(value.as_ref().to_vec())
                .unsigned_64()
                .is_some(),
            _ => false,
        }
    }
}

impl<PSK: ParamsSubspaceKey> WasmParamsKeeper<PSK> {
    /// Hook invoked after parameters are updated via governance.
    ///
    /// The current implementation simply logs the change. Integration
    /// with `WasmEngine` will allow resizing caches or adjusting limits at
    /// runtime once the engine is implemented.
    /// Forward parameter changes to the [`WasmEngine`].
    ///
    /// The keeper will call this after executing a governance proposal that
    /// modifies the wasm module's parameters. Engines can react by resizing
    /// internal caches or adjusting limits.
    pub fn on_update<E, A, S, Q>(&self, engine: &E, old: &Params, new: &Params)
    where
        E: crate::engine::WasmEngine<A, S, Q>,
        A: cosmwasm_vm::BackendApi,
        S: cosmwasm_vm::Storage,
        Q: cosmwasm_vm::Querier,
    {
        if let Err(err) = engine.on_params_change(old, new) {
            tracing::warn!("failed to notify wasm engine of param change: {err}");
        }
    }
}

