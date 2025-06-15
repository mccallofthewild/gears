//! Command line interface for the wasm module.
//!
//! This module simply re-exports the [`query`] and [`tx`] submodules which
//! implement the concrete `clap` commands.  The design mirrors the original
//! Go implementation from
//! [`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/client/cli)
//! and interacts with the
//! [`CosmWasm VM`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)
//! via the keeper and
//! [`wasmvm`](https://github.com/CosmWasm/wasmvm) bindings.
//! Commands allow uploading code, instantiating and executing contracts as well
//! as querying state, all wired through the rest of the Gears CLI.
pub mod query;
pub mod tx;
