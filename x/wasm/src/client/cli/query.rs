//! CLI query subcommands for the wasm module.
//!
//! This mirrors the high level queries available in the original
//! [`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/client/cli/query.go)
//! implementation while using the Gears `QueryHandler` trait. Queries are routed
//! through the `WasmKeeper` and ultimately executed by the `CosmWasm VM`.
//! Command names and semantics intentionally follow the Go bindings in
//! [`wasmvm`](https://github.com/CosmWasm/wasmvm) for familiarity.

use crate::types::query::{
    QueryCodeRequest, QueryCodeResponse, QueryCodesRequest, QueryCodesResponse,
    QueryContractInfoRequest, QueryContractInfoResponse, QueryContractsByCodeRequest,
    QueryContractsByCodeResponse, QueryRawContractStateRequest, QueryRawContractStateResponse,
    QuerySmartContractStateRequest, QuerySmartContractStateResponse,
};
use address::AccAddress;
use clap::{Args, Subcommand};
use gears::{application::handlers::client::QueryHandler, baseapp::Query, core::Protobuf};
use serde::{Deserialize, Serialize};

/// CLI entrypoint for wasm queries.
#[derive(Args, Debug)]
pub struct WasmQueryCli {
    #[command(subcommand)]
    pub command: WasmQueryCommands,
}

/// Individual wasm query commands.
#[derive(Subcommand, Debug)]
pub enum WasmQueryCommands {
    /// Download raw bytecode by code id.
    Code { code_id: u64 },
    /// List all uploaded code identifiers.
    Codes,
    /// List contracts that were instantiated from a given code id.
    ContractsByCode { code_id: u64 },
    /// Fetch metadata for a contract address.
    ContractInfo { address: AccAddress },
    /// Execute a contract defined smart query. The argument is hex encoded JSON.
    Smart { address: AccAddress, query: String },
    /// Read raw storage key from a contract. `key` is hex encoded.
    Raw { address: AccAddress, key: String },
}

#[derive(Clone, Debug)]
pub struct WasmQueryHandler;

impl QueryHandler for WasmQueryHandler {
    type QueryRequest = WasmQuery;
    type QueryCommands = WasmQueryCli;
    type QueryResponse = WasmQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let req = match &command.command {
            WasmQueryCommands::Code { code_id } => {
                WasmQuery::Code(QueryCodeRequest { code_id: *code_id })
            }
            WasmQueryCommands::Codes => WasmQuery::Codes(QueryCodesRequest {}),
            WasmQueryCommands::ContractsByCode { code_id } => {
                WasmQuery::ContractsByCode(QueryContractsByCodeRequest { code_id: *code_id })
            }
            WasmQueryCommands::ContractInfo { address } => {
                WasmQuery::ContractInfo(QueryContractInfoRequest {
                    address: address.to_string(),
                })
            }
            WasmQueryCommands::Smart { address, query } => {
                let data = hex::decode(query).unwrap_or_else(|_| query.as_bytes().to_vec());
                WasmQuery::Smart(QuerySmartContractStateRequest {
                    address: address.to_string(),
                    query_data: data,
                })
            }
            WasmQueryCommands::Raw { address, key } => {
                let key = hex::decode(key).unwrap_or_else(|_| key.as_bytes().to_vec());
                WasmQuery::Raw(QueryRawContractStateRequest {
                    address: address.to_string(),
                    key,
                })
            }
        };
        Ok(req)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let resp = match command.command {
            WasmQueryCommands::Code { .. } => {
                WasmQueryResponse::Code(QueryCodeResponse::decode_vec(&query_bytes)?)
            }
            WasmQueryCommands::Codes => {
                WasmQueryResponse::Codes(QueryCodesResponse::decode_vec(&query_bytes)?)
            }
            WasmQueryCommands::ContractsByCode { .. } => WasmQueryResponse::ContractsByCode(
                QueryContractsByCodeResponse::decode_vec(&query_bytes)?,
            ),
            WasmQueryCommands::ContractInfo { .. } => WasmQueryResponse::ContractInfo(
                QueryContractInfoResponse::decode_vec(&query_bytes)?,
            ),
            WasmQueryCommands::Smart { .. } => {
                WasmQueryResponse::Smart(QuerySmartContractStateResponse::decode_vec(&query_bytes)?)
            }
            WasmQueryCommands::Raw { .. } => {
                WasmQueryResponse::Raw(QueryRawContractStateResponse::decode_vec(&query_bytes)?)
            }
        };
        Ok(resp)
    }
}

/// Enum covering all possible wasm queries. The derivation implements the
/// `Query` trait and serializes to protobuf bytes which the node accepts.
#[derive(Clone, Debug, PartialEq, Query)]
#[query(request)]
pub enum WasmQuery {
    Code(QueryCodeRequest),
    Codes(QueryCodesRequest),
    ContractsByCode(QueryContractsByCodeRequest),
    ContractInfo(QueryContractInfoRequest),
    Smart(QuerySmartContractStateRequest),
    Raw(QueryRawContractStateRequest),
}

/// Responses for each query variant. Mirrors the structures returned by
/// `wasmd` and `wasmvm`.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[serde(untagged)]
#[query(response)]
pub enum WasmQueryResponse {
    Code(QueryCodeResponse),
    Codes(QueryCodesResponse),
    ContractsByCode(QueryContractsByCodeResponse),
    ContractInfo(QueryContractInfoResponse),
    Smart(QuerySmartContractStateResponse),
    Raw(QueryRawContractStateResponse),
}
