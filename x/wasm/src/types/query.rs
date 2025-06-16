use cosmwasm_std::Binary;
use gears::{
    derive::{Protobuf, Query},
    types::{
        address::AccAddress,
        pagination::{request::PaginationRequest, response::PaginationResponse},
    },
};
use serde::{Deserialize, Serialize};

/// Re-export generated protobuf types so users can construct them directly if
/// needed. These correspond to `cosmwasm.wasm.v1.Query*` messages in wasmd.
pub mod proto {
    pub use cosmos_sdk_proto::cosmwasm::wasm::v1::{
        CodeInfoResponse as ProtoCodeInfoResponse, ContractInfo as ProtoContractInfo,
        QueryCodeRequest as ProtoQueryCodeRequest, QueryCodeResponse as ProtoQueryCodeResponse,
        QueryContractInfoRequest as ProtoQueryContractInfoRequest,
        QueryContractInfoResponse as ProtoQueryContractInfoResponse,
        QueryContractsByCodeRequest as ProtoQueryContractsByCodeRequest,
        QueryContractsByCodeResponse as ProtoQueryContractsByCodeResponse,
        QueryRawContractStateRequest as ProtoQueryRawContractStateRequest,
        QueryRawContractStateResponse as ProtoQueryRawContractStateResponse,
        QuerySmartContractStateRequest as ProtoQuerySmartContractStateRequest,
        QuerySmartContractStateResponse as ProtoQuerySmartContractStateResponse,
    };
}

/// Smart contract query request sending an arbitrary JSON message to the
/// contract. The response is raw JSON returned from the contract.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(request, url = "/cosmwasm.wasm.v1.Query/SmartContractState")]
#[proto(raw = "proto::ProtoQuerySmartContractStateRequest")]
pub struct QuerySmartContractState {
    /// Address of the contract to query.
    pub address: AccAddress,
    /// Binary encoded JSON query message.
    pub query_data: Binary,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(response)]
#[proto(raw = "proto::ProtoQuerySmartContractStateResponse")]
pub struct QuerySmartContractStateResponse {
    /// Raw JSON data returned from the contract.
    pub data: Binary,
}

/// Raw contract store query fetching a single key.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(request, url = "/cosmwasm.wasm.v1.Query/RawContractState")]
#[proto(raw = "proto::ProtoQueryRawContractStateRequest")]
pub struct QueryRawContractState {
    /// Address of the contract to inspect.
    pub address: AccAddress,
    /// Key to load from the contract storage.
    pub query_data: Binary,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(response)]
#[proto(raw = "proto::ProtoQueryRawContractStateResponse")]
pub struct QueryRawContractStateResponse {
    /// Raw bytes stored at the given key.
    pub data: Binary,
}

/// Request for retrieving code bytes and metadata by code ID.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(request, url = "/cosmwasm.wasm.v1.Query/Code")]
#[proto(raw = "proto::ProtoQueryCodeRequest")]
pub struct QueryCode {
    pub code_id: u64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(response)]
#[proto(raw = "proto::ProtoQueryCodeResponse")]
pub struct QueryCodeResponse {
    #[proto(optional)]
    pub code_info: Option<proto::ProtoCodeInfoResponse>,
    pub data: Binary,
}

/// Request for contract metadata such as admin and code ID.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(request, url = "/cosmwasm.wasm.v1.Query/ContractInfo")]
#[proto(raw = "proto::ProtoQueryContractInfoRequest")]
pub struct QueryContractInfo {
    pub address: AccAddress,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(response)]
#[proto(raw = "proto::ProtoQueryContractInfoResponse")]
pub struct QueryContractInfoResponse {
    pub address: AccAddress,
    #[proto(optional)]
    pub contract_info: Option<proto::ProtoContractInfo>,
}

/// List all contracts instantiated from a specific code ID.
///
/// Pagination behaviour mirrors `wasmd` where the default limit is
/// 100 contracts and results are ordered lexicographically by address.
/// `pagination` may be omitted to use this default.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(request, url = "/cosmwasm.wasm.v1.Query/ContractsByCode")]
#[proto(raw = "proto::ProtoQueryContractsByCodeRequest")]
pub struct QueryContractsByCode {
    pub code_id: u64,
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Query, Protobuf)]
#[query(response)]
#[proto(raw = "proto::ProtoQueryContractsByCodeResponse")]
pub struct QueryContractsByCodeResponse {
    #[proto(repeated)]
    pub contracts: Vec<AccAddress>,
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// Top-level query enumeration used by the ABCI handler. Each variant
/// corresponds to one of the wasm query endpoints. When serialized to JSON the
/// `@type` field contains the full gRPC service path as shown below.
///
/// ```json
/// {"@type":"/cosmwasm.wasm.v1.Query/Code","code_id":7}
/// ```
#[derive(Debug, Clone, Serialize, Query)]
#[query(request)]
#[serde(tag = "@type")]
pub enum WasmQuery {
    #[serde(rename = "/cosmwasm.wasm.v1.Query/SmartContractState")]
    SmartContractState(QuerySmartContractState),
    #[serde(rename = "/cosmwasm.wasm.v1.Query/RawContractState")]
    RawContractState(QueryRawContractState),
    #[serde(rename = "/cosmwasm.wasm.v1.Query/Code")]
    Code(QueryCode),
    #[serde(rename = "/cosmwasm.wasm.v1.Query/ContractInfo")]
    ContractInfo(QueryContractInfo),
    #[serde(rename = "/cosmwasm.wasm.v1.Query/ContractsByCode")]
    ContractsByCode(QueryContractsByCode),
}
