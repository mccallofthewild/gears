//! Client-facing interfaces for the CosmWasm module.
//!
//! This directory gathers the various ways an external application can
//! interact with the module:
//! - Command line utilities under [`cli`]
//! - gRPC services under [`grpc`]
//! - REST endpoints under [`rest`]
//!
//! The semantics of these interfaces are modelled after the original Go
//! implementation in
//! [`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm) and rely on
//! the [`cosmwasm_vm`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)
//! runtime. Behaviour mirrors the `wasmvm` bindings used in Go
//! (<https://github.com/CosmWasm/wasmvm>) but is implemented natively in Rust.
//! This crate exposes them separately so consumers can pick the pieces they
//! require.
pub use cli::*;
pub use grpc::*;
pub use rest::*;
pub mod cli;
pub mod grpc;
pub mod rest;
