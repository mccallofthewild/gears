//! CosmWasm module scaffolding.
//!
//! This crate integrates the CosmWasm virtual machine
//! ([`cosmwasm_vm`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm))
//! with the Gears application framework. It mirrors the design of the Go
//! `x/wasm` module in [`wasmd`](https://github.com/CosmWasm/wasmd) while staying
//! entirely in Rust.  The public API is modelled after the Go bindings provided
//! by [`wasmvm`](https://github.com/CosmWasm/wasmvm).
//!
//! The crate exposes a public `Keeper` responsible for managing contract code
//! and instances. Execution is delegated to a pluggable [`WasmEngine`] trait,
//! whose default implementation wraps `cosmwasm_vm`.
//!
//! Files are organised following the common xmodule layout:
//! - `keeper`: state access and business logic
//! - `engine`: bindings to `cosmwasm_vm`
//! - `abci_handler`: ABCI entry points
//! - `genesis`: genesis loading and export
//! - `message`: transaction message definitions
//! - `params`: module parameters
//! - `types`: query structures and internal shared types
//! - `client`: CLI/REST/GRPC interfaces
//! - `error`: error types
//!
//! Each file contains a more detailed description of its responsibilities and
//! security considerations.

mod abci_handler;
mod client;
mod engine;
mod error;
mod genesis;
mod keeper;
mod message;
mod params;
mod types;

pub use abci_handler::*;
pub use client::*;
pub use engine::*;
pub use error::*;
pub use genesis::*;
pub use keeper::*;
pub use message::*;
pub use params::*;
pub use types::*;
