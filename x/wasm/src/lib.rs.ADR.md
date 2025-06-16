# ADR: Expand `x/wasm` module root

## Status
Proposed

## Context

Currently `lib.rs` contains only a stub keeper for compilation. Once the module
components are implemented we need to re-export the keeper, engine and client
modules so that other crates can depend on them. This mirrors the entry point in
`wasmd` where `x/wasm` exports its keeper and types.

## Decision

`lib.rs` will define the full module structure:

```rust
pub mod message;
pub mod types;
pub mod params;
pub mod error;
pub mod engine;
pub mod keeper;
pub mod genesis;
pub mod abci_handler;
pub mod client;
```

It will re-export the main `Keeper`, `CosmwasmEngine` and client modules for
external use. Documentation comments will link to the upstream CosmWasm and
wasmvm projects as references.

The root module will also define feature gates:

* `#[cfg(feature = "cli")]` for CLI commands.
* `#[cfg(feature = "grpc")]` for gRPC services.
* `#[cfg(feature = "rest")]` for REST routes.

This mirrors the optional build tags present in `wasmd` and allows lightweight
builds for embedded scenarios. The public API surface should remain stable so
that other modules, such as governance, can depend on `wasm::Keeper` for
on-chain proposals.

## Consequences

A complete module root enables applications to depend on the wasm crate as a
single cohesive unit.

## Detailed Structure

### Module Visibility

Every submodule declared here must also be `pub` so that downstream crates can
import types such as `Keeper` and `Message`. To maintain a clean namespace the
root module should re-export the most frequently used items:

```rust
pub use crate::keeper::Keeper;
pub use crate::engine::{WasmEngine, CosmwasmEngine};
pub use crate::client::{register_grpc, register_routes};
```

This mirrors the practice in [`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/module.go)
where the module struct exposes convenience methods for initialization and
service registration.

### Feature Flags Explained

* `cli` – builds the `client::cli` submodule and enables Clap dependencies. This
  feature is optional to allow lightweight binaries in environments where a CLI
  is not required.
* `grpc` – pulls in `tonic` and compiles the gRPC service implementation.
* `rest` – uses `axum` to provide HTTP endpoints for contract interaction.

Conditional compilation guards around these modules ensure that unused features
do not appear in the public API when they are disabled. The pattern follows the
approach recommended in the Rust book's chapter on conditional compilation.

### Documentation

`lib.rs` should contain module-level documentation describing the high-level
functionality of the wasm integration. This includes referencing the
[`CosmWasm VM`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm), the
Go bindings in [`wasmvm`](https://github.com/CosmWasm/wasmvm), and the module
structure of [`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm). The
docs should highlight the equivalence of types and entry points so developers
familiar with the Go implementation can navigate the Rust codebase easily.

### Crate-Level Attributes

The crate should specify `#![deny(unsafe_code)]` to uphold the repository's
policy of forbidding unsafe Rust. Additional lint groups such as
`#![deny(rust_2018_idioms)]` can be enabled to keep the codebase consistent.

### Integration Test Helpers

Expose test helper functions under a `#[cfg(test)] mod test_utils` section. These
helpers might spin up an in-memory keeper and engine instance for quick
contract execution tests. Following the pattern in `cosmwasm-vm`, we can provide
an `instantiate_contract` helper that sets up default `Env` and `MessageInfo`
structures.

### Future Extensions

Over time additional modules such as governance proposal handlers or IBC hooks
may be introduced. `lib.rs` should remain the central place to export these so
applications do not need to know the internal directory layout. When new modules
are added, update the documentation to point to the relevant ADRs and upstream
references.

