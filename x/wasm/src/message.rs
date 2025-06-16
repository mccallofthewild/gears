// CosmWasm transaction messages used by the wasm module.
//
// These structures mirror the message definitions from `wasmd` so that
// external tooling can interact with the node using identical JSON payloads.
// Each message derives `AppMessage` which provides the `TxMessage` trait
// implementation required by the framework.

use cosmwasm_std::Binary;
use gears::{
    core::errors::CoreError,
    derive::AppMessage,
    types::{
        address::AccAddress,
        base::{coins::UnsignedCoins, errors::CoinError},
    },
};
use serde::{Deserialize, Serialize};

// Re-export proto types from `cosmos-sdk-proto` so that callers may
// construct the generated protobuf structs directly if desired. These
// mirror the definitions in `wasmd` under `x/wasm/types`.
mod proto {
    pub use cosmos_sdk_proto::cosmwasm::wasm::v1::{
        AccessConfig as ProtoAccessConfig, MsgClearAdmin as ProtoMsgClearAdmin,
        MsgExecuteContract as ProtoMsgExecuteContract,
        MsgInstantiateContract as ProtoMsgInstantiateContract,
        MsgMigrateContract as ProtoMsgMigrateContract, MsgStoreCode as ProtoMsgStoreCode,
        MsgUpdateAdmin as ProtoMsgUpdateAdmin,
    };
}

/// Access control configuration determining who may instantiate a contract.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccessConfig {
    /// Permission variant matching `AccessType` in `wasmd`.
    pub permission: AccessType,
    /// Additional addresses allowed when using `AnyOfAddresses`.
    #[serde(default)]
    pub addresses: Vec<AccAddress>,
}

impl Default for AccessConfig {
    fn default() -> Self {
        Self {
            permission: AccessType::Unspecified,
            addresses: Vec::new(),
        }
    }
}

impl AccessConfig {
    /// Basic validation used during message checks.
    pub fn validate_basic(&self) -> Result<(), anyhow::Error> {
        match self.permission {
            AccessType::Unspecified => Err(anyhow::anyhow!("access type unspecified")),
            AccessType::Nobody | AccessType::Everybody => Ok(()),
            AccessType::AnyOfAddresses => {
                if self.addresses.is_empty() {
                    Err(anyhow::anyhow!("addresses required for AnyOfAddresses"))
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl From<AccessConfig> for proto::ProtoAccessConfig {
    fn from(cfg: AccessConfig) -> Self {
        Self {
            permission: cfg.permission as i32,
            addresses: cfg.addresses.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto::ProtoAccessConfig> for AccessConfig {
    type Error = CoreError;

    fn try_from(value: proto::ProtoAccessConfig) -> Result<Self, Self::Error> {
        let addresses = value
            .addresses
            .into_iter()
            .map(|a| {
                AccAddress::from_bech32(&a).map_err(|e| CoreError::DecodeAddress(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let permission = match value.permission {
            1 => AccessType::Nobody,
            3 => AccessType::Everybody,
            4 => AccessType::AnyOfAddresses,
            _ => AccessType::Unspecified,
        };
        Ok(AccessConfig {
            permission,
            addresses,
        })
    }
}

impl core_types::Protobuf<proto::ProtoAccessConfig> for AccessConfig {}

/// Enumeration of access types supported by CosmWasm.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessType {
    /// Placeholder value used when no permission is specified.
    Unspecified = 0,
    /// No account may instantiate.
    Nobody = 1,
    /// Any account may instantiate.
    Everybody = 3,
    /// One of the addresses in the list may instantiate.
    AnyOfAddresses = 4,
}

/// Uploads new WASM bytecode.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgStoreCode")]
pub struct MsgStoreCode {
    #[msg(signer)]
    pub sender: AccAddress,
    /// The raw or gzip compressed WASM bytecode.
    pub wasm_byte_code: Vec<u8>,
    /// Optional instantiation permission for the stored code.
    pub instantiate_permission: Option<AccessConfig>,
}

impl MsgStoreCode {
    pub fn validate_basic(&self) -> Result<(), anyhow::Error> {
        if self.wasm_byte_code.is_empty() {
            return Err(anyhow::anyhow!("wasm byte code cannot be empty"));
        }
        if let Some(ref cfg) = self.instantiate_permission {
            cfg.validate_basic()?;
        }
        Ok(())
    }
}

impl From<MsgStoreCode> for proto::ProtoMsgStoreCode {
    fn from(msg: MsgStoreCode) -> Self {
        Self {
            sender: msg.sender.into(),
            wasm_byte_code: msg.wasm_byte_code,
            instantiate_permission: msg.instantiate_permission.map(Into::into),
        }
    }
}

impl TryFrom<proto::ProtoMsgStoreCode> for MsgStoreCode {
    type Error = CoreError;

    fn try_from(value: proto::ProtoMsgStoreCode) -> Result<Self, Self::Error> {
        let sender = AccAddress::from_bech32(&value.sender)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let instantiate_permission = match value.instantiate_permission {
            Some(p) => Some(p.try_into()?),
            None => None,
        };
        Ok(MsgStoreCode {
            sender,
            wasm_byte_code: value.wasm_byte_code,
            instantiate_permission,
        })
    }
}

impl core_types::Protobuf<proto::ProtoMsgStoreCode> for MsgStoreCode {}

/// Instantiate a stored contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgInstantiateContract")]
pub struct MsgInstantiateContract {
    #[msg(signer)]
    pub sender: AccAddress,
    /// Optional admin allowed to perform migrations.
    pub admin: Option<AccAddress>,
    pub code_id: u64,
    pub label: String,
    pub msg: Binary,
    pub funds: UnsignedCoins,
}

impl MsgInstantiateContract {
    pub fn validate_basic(&self) -> Result<(), anyhow::Error> {
        if self.code_id == 0 {
            return Err(anyhow::anyhow!("code id is required"));
        }
        if self.label.trim().is_empty() {
            return Err(anyhow::anyhow!("label cannot be empty"));
        }
        Ok(())
    }
}

impl From<MsgInstantiateContract> for proto::ProtoMsgInstantiateContract {
    fn from(msg: MsgInstantiateContract) -> Self {
        Self {
            sender: msg.sender.into(),
            admin: msg.admin.map(Into::into).unwrap_or_default(),
            code_id: msg.code_id,
            label: msg.label,
            msg: msg.msg.into(),
            funds: msg.funds.into(),
        }
    }
}

impl TryFrom<proto::ProtoMsgInstantiateContract> for MsgInstantiateContract {
    type Error = CoreError;

    fn try_from(value: proto::ProtoMsgInstantiateContract) -> Result<Self, Self::Error> {
        let sender = AccAddress::from_bech32(&value.sender)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let admin = if value.admin.is_empty() {
            None
        } else {
            Some(
                AccAddress::from_bech32(&value.admin)
                    .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            )
        };
        let funds = value
            .funds
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e: CoinError| CoreError::Coins(e.to_string()))?;
        Ok(MsgInstantiateContract {
            sender,
            admin,
            code_id: value.code_id,
            label: value.label,
            msg: Binary::from(value.msg),
            funds: UnsignedCoins::new(funds).map_err(|e| CoreError::Coins(e.to_string()))?,
        })
    }
}

impl core_types::Protobuf<proto::ProtoMsgInstantiateContract> for MsgInstantiateContract {}

/// Execute a contract method.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgExecuteContract")]
pub struct MsgExecuteContract {
    #[msg(signer)]
    pub sender: AccAddress,
    pub contract: AccAddress,
    pub msg: Binary,
    pub funds: UnsignedCoins,
}

impl MsgExecuteContract {
    pub fn validate_basic(&self) -> Result<(), anyhow::Error> {
        if self.msg.0.is_empty() {
            return Err(anyhow::anyhow!("execute message cannot be empty"));
        }
        Ok(())
    }
}

impl From<MsgExecuteContract> for proto::ProtoMsgExecuteContract {
    fn from(msg: MsgExecuteContract) -> Self {
        Self {
            sender: msg.sender.into(),
            contract: msg.contract.into(),
            msg: msg.msg.into(),
            funds: msg.funds.into(),
        }
    }
}

impl TryFrom<proto::ProtoMsgExecuteContract> for MsgExecuteContract {
    type Error = CoreError;

    fn try_from(value: proto::ProtoMsgExecuteContract) -> Result<Self, Self::Error> {
        let sender = AccAddress::from_bech32(&value.sender)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let contract = AccAddress::from_bech32(&value.contract)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let funds = value
            .funds
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e: CoinError| CoreError::Coins(e.to_string()))?;
        Ok(MsgExecuteContract {
            sender,
            contract,
            msg: Binary::from(value.msg),
            funds: UnsignedCoins::new(funds).map_err(|e| CoreError::Coins(e.to_string()))?,
        })
    }
}

impl core_types::Protobuf<proto::ProtoMsgExecuteContract> for MsgExecuteContract {}

/// Migrate an existing contract to new code.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgMigrateContract")]
pub struct MsgMigrateContract {
    #[msg(signer)]
    pub sender: AccAddress,
    pub contract: AccAddress,
    pub code_id: u64,
    pub msg: Binary,
}

impl MsgMigrateContract {
    pub fn validate_basic(&self) -> Result<(), anyhow::Error> {
        if self.code_id == 0 {
            return Err(anyhow::anyhow!("code id is required"));
        }
        Ok(())
    }
}

impl From<MsgMigrateContract> for proto::ProtoMsgMigrateContract {
    fn from(msg: MsgMigrateContract) -> Self {
        Self {
            sender: msg.sender.into(),
            contract: msg.contract.into(),
            code_id: msg.code_id,
            msg: msg.msg.into(),
        }
    }
}

impl TryFrom<proto::ProtoMsgMigrateContract> for MsgMigrateContract {
    type Error = CoreError;

    fn try_from(value: proto::ProtoMsgMigrateContract) -> Result<Self, Self::Error> {
        let sender = AccAddress::from_bech32(&value.sender)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let contract = AccAddress::from_bech32(&value.contract)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        Ok(MsgMigrateContract {
            sender,
            contract,
            code_id: value.code_id,
            msg: Binary::from(value.msg),
        })
    }
}

impl core_types::Protobuf<proto::ProtoMsgMigrateContract> for MsgMigrateContract {}

/// Update a contract's admin address.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgUpdateAdmin")]
pub struct MsgUpdateAdmin {
    #[msg(signer)]
    pub sender: AccAddress,
    pub new_admin: AccAddress,
    pub contract: AccAddress,
}

impl MsgUpdateAdmin {
    pub fn validate_basic(&self) -> Result<(), anyhow::Error> {
        if self.sender == self.new_admin {
            return Err(anyhow::anyhow!("new admin is the same as current"));
        }
        Ok(())
    }
}

impl From<MsgUpdateAdmin> for proto::ProtoMsgUpdateAdmin {
    fn from(msg: MsgUpdateAdmin) -> Self {
        Self {
            sender: msg.sender.into(),
            new_admin: msg.new_admin.into(),
            contract: msg.contract.into(),
        }
    }
}

impl TryFrom<proto::ProtoMsgUpdateAdmin> for MsgUpdateAdmin {
    type Error = CoreError;

    fn try_from(value: proto::ProtoMsgUpdateAdmin) -> Result<Self, Self::Error> {
        let sender = AccAddress::from_bech32(&value.sender)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let new_admin = AccAddress::from_bech32(&value.new_admin)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let contract = AccAddress::from_bech32(&value.contract)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        Ok(MsgUpdateAdmin {
            sender,
            new_admin,
            contract,
        })
    }
}

impl core_types::Protobuf<proto::ProtoMsgUpdateAdmin> for MsgUpdateAdmin {}

/// Remove the current admin from a contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AppMessage)]
#[msg(url = "/cosmwasm.wasm.v1.MsgClearAdmin")]
pub struct MsgClearAdmin {
    #[msg(signer)]
    pub sender: AccAddress,
    pub contract: AccAddress,
}

impl MsgClearAdmin {
    pub fn validate_basic(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl From<MsgClearAdmin> for proto::ProtoMsgClearAdmin {
    fn from(msg: MsgClearAdmin) -> Self {
        Self {
            sender: msg.sender.into(),
            contract: msg.contract.into(),
        }
    }
}

impl TryFrom<proto::ProtoMsgClearAdmin> for MsgClearAdmin {
    type Error = CoreError;

    fn try_from(value: proto::ProtoMsgClearAdmin) -> Result<Self, Self::Error> {
        let sender = AccAddress::from_bech32(&value.sender)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        let contract = AccAddress::from_bech32(&value.contract)
            .map_err(|e| CoreError::DecodeAddress(e.to_string()))?;
        Ok(MsgClearAdmin { sender, contract })
    }
}

impl core_types::Protobuf<proto::ProtoMsgClearAdmin> for MsgClearAdmin {}

/// Union type covering all wasm messages.
#[derive(Debug, Clone, Serialize, AppMessage)]
#[serde(tag = "@type")]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    #[serde(rename = "/cosmwasm.wasm.v1.MsgStoreCode")]
    #[msg(url(path = MsgStoreCode::TYPE_URL))]
    StoreCode(MsgStoreCode),
    #[serde(rename = "/cosmwasm.wasm.v1.MsgInstantiateContract")]
    #[msg(url(path = MsgInstantiateContract::TYPE_URL))]
    InstantiateContract(MsgInstantiateContract),
    #[serde(rename = "/cosmwasm.wasm.v1.MsgExecuteContract")]
    #[msg(url(path = MsgExecuteContract::TYPE_URL))]
    ExecuteContract(MsgExecuteContract),
    #[serde(rename = "/cosmwasm.wasm.v1.MsgMigrateContract")]
    #[msg(url(path = MsgMigrateContract::TYPE_URL))]
    MigrateContract(MsgMigrateContract),
    #[serde(rename = "/cosmwasm.wasm.v1.MsgUpdateAdmin")]
    #[msg(url(path = MsgUpdateAdmin::TYPE_URL))]
    UpdateAdmin(MsgUpdateAdmin),
    #[serde(rename = "/cosmwasm.wasm.v1.MsgClearAdmin")]
    #[msg(url(path = MsgClearAdmin::TYPE_URL))]
    ClearAdmin(MsgClearAdmin),
}
