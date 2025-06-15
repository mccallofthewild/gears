//! REST handlers for the wasm module.
//!
//! Provide HTTP endpoints that mirror gRPC functionality for simple integration
//! with web applications.

use crate::{
    types::query::{
        QueryCodeRequest, QueryCodesRequest, QueryContractInfoRequest, QueryContractsByCodeRequest,
        QueryRawContractStateRequest, QuerySmartContractStateRequest,
    },
    WasmNodeQueryRequest, WasmNodeQueryResponse,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct SmartQuery {
    /// Hex or plain string encoded JSON query.
    query_data: String,
}

#[derive(Deserialize)]
struct RawQuery {
    /// Hex or plain string encoded key bytes.
    query_data: String,
}

/// Get contract metadata by address.
pub async fn contract_info<
    QReq: QueryRequest + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(address): Path<String>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = WasmNodeQueryRequest::ContractInfo(QueryContractInfoRequest { address });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

/// Download raw wasm code by id.
pub async fn code<
    QReq: QueryRequest + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(code_id): Path<u64>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = WasmNodeQueryRequest::Code(QueryCodeRequest { code_id });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

/// List metadata for all stored wasm codes.
pub async fn codes<
    QReq: QueryRequest + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = WasmNodeQueryRequest::Codes(QueryCodesRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

/// List contracts instantiated from a given code id.
pub async fn contracts_by_code<
    QReq: QueryRequest + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(code_id): Path<u64>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = WasmNodeQueryRequest::ContractsByCode(QueryContractsByCodeRequest { code_id });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

/// Execute a smart query against a contract.
pub async fn smart_contract_state<
    QReq: QueryRequest + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path((address, SmartQuery { query_data })): Path<(String, SmartQuery)>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let data = hex::decode(&query_data).unwrap_or_else(|_| query_data.into_bytes());
    let req = WasmNodeQueryRequest::Smart(QuerySmartContractStateRequest {
        address,
        query_data: data,
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

/// Read a raw key from a contract's storage.
pub async fn raw_contract_state<
    QReq: QueryRequest + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path((address, RawQuery { query_data })): Path<(String, RawQuery)>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let key = hex::decode(&query_data).unwrap_or_else(|_| query_data.into_bytes());
    let req = WasmNodeQueryRequest::Raw(QueryRawContractStateRequest { address, key });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .route("/v1/contract/:address", get(contract_info))
        .route("/v1/code/:code_id", get(code))
        .route("/v1/code", get(codes))
        .route("/v1/code/:code_id/contracts", get(contracts_by_code))
        .route(
            "/v1/contract/:address/smart/:query_data",
            get(smart_contract_state),
        )
        .route(
            "/v1/contract/:address/raw/:query_data",
            get(raw_contract_state),
        )
}
