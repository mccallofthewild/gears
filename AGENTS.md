# Gears Repository Overview

This repository hosts **Gears**, a Rust implementation of the Cosmos SDK. It is organised as a Cargo workspace consisting of many crates, each providing a portion of the blockchain stack. The codebase can be used to build a Cosmos style blockchain (see `gaia-rs` for an example node).

## Repository Structure

- `address` – utilities for Bech32 account, validator, and consensus addresses.
- `core-types` – fundamental protocol types such as transaction bodies, query types and protobuf helpers.
- `database` – abstract database layer with implementations for RocksDB, sled and in-memory storage.
- `extensions` – small utility modules used across the workspace (locks, infallible conversions, pagination helpers, etc.).
- `gaia-rs` – example Cosmos Hub (Gaia) node built on Gears. Contains CLI, gRPC/REST servers and integration tests.
- `gas` – gas accounting abstractions and storage wrappers.
- `gears` – main application framework implementing the base app, context types, CLI utilities and runtime.
- `keyring` – key management library for local and ledger based keys.
- `kv_store` – multi store implementation backed by the `trees` crate.
- `macros/*` – procedural macro crates used to derive transaction messages, query types, protobuf conversions and key definitions.
- `tendermint` – minimal Tendermint RPC/ABCI interfaces and helper types.
- `trees` – IAVL tree and Merkle utilities used for persistent storage.
- `x/*` – modules corresponding to Cosmos SDK modules (auth, bank, staking, etc.).

Documentation for various components can be found under `docs/` and within each crate’s `Readme.md`.

## Building and Testing

The workspace targets Rust 1.79 or later. Standard development workflows are driven via Cargo:

```bash
cargo build --workspace            # build all crates
cargo test --workspace             # run all tests
```

Some crates have additional Makefile helpers. Running a local Gaia example node can be done with:

```bash
make init           # initialise local chain state
make run            # run gaia-rs node (listens on 127.0.0.1:26658)
make tendermint-start    # start Tendermint (RPC on 127.0.0.1:26657)
```

## Code Style and Lints

The workspace enforces `rust_2018_idioms` and forbids `unsafe` code at the workspace level (see `Cargo.toml`). Clippy lints are configured for the workspace as well.

When contributing, run `cargo fmt --all` and `cargo clippy --all-targets` before submitting patches.

## Additional Notes

- Most crates are `#![no_std]` friendly except where explicitly requiring standard library features.
- The `x` directory mirrors Cosmos SDK modules and is intended for extending the framework with additional functionality.
- Example tutorials for gRPC queries, REST endpoints and transactions are provided in `docs/tutorials/`.

For more details on any crate, consult its `Readme.md` or the module level documentation.
## Module Integration and Shared Infrastructure

The `x/*` crates mirror Cosmos SDK modules and rely on a common set of building blocks exposed by the `gears` crate.
Every module defines a **Keeper** which encapsulates its state storage and parameters. Keepers are generic over a
`StoreKey` and, when needed, a `ParamsSubspaceKey`. Storage is handled by the `kv_store` crate which provides
`ApplicationKVBank`/`TransactionKVBank`/`QueryKVStore` and a `MultiStore` ensuring each keeper operates in an isolated
prefix. Parameters live under their own subspace and are manipulated through the `ParamsKeeper` trait.

Modules expose an `ABCIHandler` implementing `application::handlers::node::ABCIHandler`. This trait defines how
messages are dispatched, how queries are executed and how genesis and block lifecycle hooks are wired. Each handler
invokes its keeper to read or mutate state. Queries are executed via a `QueryContext` giving readonly access to the
`QueryKVStore` at a particular height. Transactions are executed inside a `TxContext` which provides a writeable
`TransactionKVBank` plus gas metering and event collection.

Routing of CLI/gRPC/REST requests happens through the application `Node` implementation. When adding a new module you
create its keeper, ABCI handler, message and query types, then register them in your node's handler and router. The
base app automatically routes transactions and queries to the correct module handler.
