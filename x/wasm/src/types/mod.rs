//! Public query and metadata types for the wasm module.
//!
//! These mirror the protobuf definitions in
//! [`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/proto/cosmwasm/wasm/v1).
//!
//! # Example
//!
//! ```rust
//! use wasm::types::{QuerySmartContractState, WasmQuery};
//! # use gears::types::address::AccAddress;
//! # let addr = AccAddress::try_from([0u8; 20].as_slice()).unwrap();
//! # let _ = WasmQuery::SmartContractState(QuerySmartContractState {
//! #     address: addr,
//! #     query_data: cosmwasm_std::Binary::default(),
//! # });
//! ```
//!
//! The re-exported items here are stable and match the structures
//! found in `wasmd`'s `query.proto`. Refer to the [`query`]
//! module for field documentation with links to the corresponding
//! `cosmwasm.wasm.v1` protobuf messages.

pub mod query;

pub use self::query::{
    QueryCode, QueryCodeResponse, QueryContractInfo, QueryContractInfoResponse,
    QueryContractsByCode, QueryContractsByCodeResponse, QueryRawContractState,
    QueryRawContractStateResponse, QuerySmartContractState, QuerySmartContractStateResponse,
    WasmQuery,
};
