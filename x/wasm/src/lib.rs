//! CosmWasm module interface.
//!
//! This crate exposes the keeper, message types and runtime engine required
//! to execute CosmWasm smart contracts. The implementation mirrors
//! [`wasmd`](https://github.com/CosmWasm/wasmd) and wraps the `cosmwasm_vm`
//! crate for contract execution.

pub mod engine;
pub mod error;
pub mod keeper;
pub mod message;
pub mod params;
pub mod types;

pub use keeper::Keeper;
