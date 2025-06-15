//! Client-facing interfaces for the wasm module.
//!
//! This directory defines CLI commands, REST handlers and gRPC services that
//! expose wasm functionality to external users. The layout mirrors the
//! structure used by other modules in this repository for consistency.
pub mod cli;
pub mod grpc;
pub mod rest;
