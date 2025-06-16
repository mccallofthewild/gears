# ADR: Implement client module for `x/wasm`

## Status
Proposed

## Context

The wasm module exposes CLI, gRPC and REST interfaces. A single module should
re-export these so consumers can enable whichever interfaces they need.

## Decision

`client/mod.rs` will publicise the `cli`, `grpc` and `rest` submodules created in
adjacent files. It may also provide helper functions to register gRPC and REST
routes with the node application.

The module exposes two convenience functions:

* `pub fn register_grpc(router: &mut Server, keeper: Keeper)` – adds the
  `WasmService` to the given gRPC server.
* `pub fn register_routes(app: Router, keeper: Keeper) -> Router` – mounts the
  REST endpoints under `/wasm` using axum.

These mirror the integration helpers in
[`wasmd/x/wasm/client`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/client)
and allow an application to opt-in to whichever interfaces it requires.

## Consequences

Aggregating the client interfaces simplifies imports for downstream crates and
keeps the module structure consistent with other parts of Gears.

## Implementation Notes

### Feature Flags

The submodules are compiled conditionally via the `cli`, `grpc`, and `rest`
feature flags defined in the crate manifest. `client/mod.rs` should use the
`cfg_if!` macro to only expose functions when the corresponding feature is
enabled. This prevents unused dependencies from being pulled in when an
application does not require a particular interface.

### Example Usage

In the application root, a developer might register the wasm clients as
follows:

```rust
#[cfg(feature = "grpc")]
wasm::client::register_grpc(&mut grpc_server, wasm_keeper.clone());

#[cfg(feature = "rest")]
router = wasm::client::register_routes(router, wasm_keeper.clone());
```

This pattern mirrors the initialization sequence used in `gaia-rs` for other
modules and ensures consistent startup behaviour across applications.

### Documentation

`client/mod.rs` should include rustdoc links pointing to the upstream
[`wasmd` client package](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/client)
so that users can reference familiar command and query semantics. Example code
snippets in the documentation will demonstrate how to build a CLI application
using Clap, gRPC and REST simultaneously.

### Testing

Unit tests should verify that the registration helpers correctly wire the
services into mock routers. For integration tests, spin up a full node with each
feature combination to ensure there are no compilation or runtime issues when
features are toggled on and off.

### Interaction with Application Framework

The registration helpers should be called from the application's builder phase.
`register_grpc` takes a mutable reference to a `tonic::transport::Server` so it
can register the `WasmService` while other modules add their own services. For
REST, the helper returns an updated `axum::Router` that mounts the wasm routes
under `/wasm`. This pattern keeps the initialization order deterministic and
matches the startup flow used by `gaia-rs`.

### Future Extensions

As new interfaces such as GraphQL or WebSockets are added to Gears, the client
module can be extended with additional feature flags and registration helpers.
Documentation should clearly describe how to integrate these to maintain a
consistent developer experience.

## Rationale

Keeping all client integrations under a single module simplifies dependencies
for downstream crates. By mirroring `wasmd`'s client package structure and
exposing ergonomic helper functions, we make it straightforward for application
developers to include or exclude wasm functionality as needed without wading
through low-level initialization details.


\nBy following these guidelines, developers can confidently expose wasm functionality to end users across multiple transport layers.
