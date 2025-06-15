//! Common query structures for the wasm module.
//!
//! This submodule hosts request and response types used by the RPC layer. The
//! shapes are inspired by `cosmwasm_std` queries so that existing clients can
//! easily interact with a chain built on Gears.

pub mod query;
