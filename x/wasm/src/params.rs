//! Module parameter definitions.
//!
//! Parameters control gas costs and permissions for CosmWasm execution. They
//! are persisted in the application's parameter store and mirror the schema
//! used by [`wasmd`](https://github.com/CosmWasm/wasmd). Only a subset is
//! implemented for now but the layout matches the upstream design so that
//! existing genesis files remain compatible.

use gears::{
    application::keepers::params::ParamsKeeper,
    extensions::corruption::UnwrapCorrupt,
    params::{ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const KEY_CODE_UPLOAD_ACCESS: &str = "CodeUploadAccess";
const KEY_INSTANTIATE_DEFAULT_PERMISSION: &str = "InstantiateDefaultPermission";
const KEY_MAX_WASM_SIZE: &str = "MaxWasmSize";

/// Permission levels for uploading or instantiating contracts.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessType {
    /// Undefined variant used when parsing fails.
    Unspecified,
    /// No one is allowed to perform the action.
    Nobody,
    /// Everyone is allowed.
    Everybody,
    /// Only addresses listed in [`AccessConfig::addresses`].
    AnyOfAddresses,
}

/// Access control configuration mirroring `wasmd`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessConfig {
    pub permission: AccessType,
    #[serde(default)]
    pub addresses: Vec<String>,
}

impl Default for AccessConfig {
    fn default() -> Self {
        Self {
            permission: AccessType::Everybody,
            addresses: Vec::new(),
        }
    }
}

/// Parameters governing wasm behaviour.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmParams {
    /// Permission for uploading new contract code.
    pub code_upload_access: AccessConfig,
    /// Default permission applied when instantiating contracts.
    pub instantiate_default_permission: AccessType,
    /// Maximum allowed size for contract byte code in bytes.
    pub max_wasm_size: u64,
}

impl Default for WasmParams {
    fn default() -> Self {
        Self {
            code_upload_access: AccessConfig::default(),
            instantiate_default_permission: AccessType::Everybody,
            // 1 MiB by default, matching wasmd.
            max_wasm_size: 1_048_576,
        }
    }
}

impl ParamsSerialize for WasmParams {
    fn keys() -> HashSet<&'static str> {
        [
            KEY_CODE_UPLOAD_ACCESS,
            KEY_INSTANTIATE_DEFAULT_PERMISSION,
            KEY_MAX_WASM_SIZE,
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (
                KEY_CODE_UPLOAD_ACCESS,
                serde_json::to_vec(&self.code_upload_access)
                    .expect("serialization should not fail"),
            ),
            (
                KEY_INSTANTIATE_DEFAULT_PERMISSION,
                serde_json::to_vec(&self.instantiate_default_permission)
                    .expect("serialization should not fail"),
            ),
            (
                KEY_MAX_WASM_SIZE,
                format!("\"{}\"", self.max_wasm_size).into_bytes(),
            ),
        ]
    }
}

impl ParamsDeserialize for WasmParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            code_upload_access: serde_json::from_slice(
                &fields.remove(KEY_CODE_UPLOAD_ACCESS).unwrap_or_default(),
            )
            .unwrap_or_default(),
            instantiate_default_permission: serde_json::from_slice(
                &fields
                    .remove(KEY_INSTANTIATE_DEFAULT_PERMISSION)
                    .unwrap_or_default(),
            )
            .unwrap_or(AccessType::Unspecified),
            max_wasm_size: ParamKind::U64
                .parse_param(fields.remove(KEY_MAX_WASM_SIZE).unwrap_or_default())
                .unsigned_64()
                .unwrap_or_corrupt(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasmParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for WasmParamsKeeper<PSK> {
    type Param = WasmParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    fn validate(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> bool {
        match String::from_utf8_lossy(key.as_ref()).as_ref() {
            KEY_CODE_UPLOAD_ACCESS => {
                serde_json::from_slice::<AccessConfig>(value.as_ref()).is_ok()
            }
            KEY_INSTANTIATE_DEFAULT_PERMISSION => {
                serde_json::from_slice::<AccessType>(value.as_ref()).is_ok()
            }
            KEY_MAX_WASM_SIZE => ParamKind::U64
                .parse_param(value.as_ref().to_vec())
                .unsigned_64()
                .is_some(),
            _ => false,
        }
    }
}
