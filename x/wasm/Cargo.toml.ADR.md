# ADR: Implement `x/wasm` crate manifest

## Status
Proposed

## Context

The new `x/wasm` crate hosts the CosmWasm integration. Its `Cargo.toml` defines
all dependencies required to compile the module and the contract execution
engine. The manifest mirrors [`wasmd`'s `x/wasm` module](https://github.com/CosmWasm/wasmd/tree/main/x/wasm)
and directly depends on [`cosmwasm-vm`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm).

## Decision

`Cargo.toml` will declare the following mandatory dependencies:

* `cosmwasm-vm` – pulled from the [CosmWasm repository](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm) to execute contracts.
* `cosmwasm-std` – shared message and query types.
* `serde`, `serde_json`, `thiserror` and `anyhow` for message encoding and structured errors.
* `gears` – provides keeper and client traits used throughout the module.

Optional features mirror [`wasmvm`](https://github.com/CosmWasm/wasmvm):

* `cosmwasm-vm/stargate` to enable Stargate queries.
* `cosmwasm-vm/iterator` for multi-key iterator support in the keeper.
* `cosmwasm-vm/IBC` to compile contract IBC entry points.

Dev-dependencies will include `cosmwasm-schema` and `gears` with the `test-utils` feature so integration tests can spin up a node and run example contracts stored under `assets/`.

## Consequences

Defining the manifest first lets all subsequent source files compile against the
correct versions of the CosmWasm libraries. Updating CosmWasm simply requires
bumping the versions in this manifest.

## Implementation Details

The `Cargo.toml` should specify a package name of `x-wasm` and set the edition
to `2021` to match the rest of the workspace. The module depends heavily on the
`core-types`, `gas`, and `tendermint` crates located in this repository, so the
manifest must use relative path dependencies to ensure workspace resolution
without needing published versions. An example snippet is shown below:

```toml
[dependencies]
cosmwasm-vm = { version = "1", default-features = false, features = [
    "iterator",
    "stargate",
    "cranelift",
] }
cosmwasm-std = "1"
gears = { path = "../../gears", default-features = false }
core-types = { path = "../../core-types" }
gas = { path = "../../gas" }
tendermint = { path = "../../tendermint" }
```

This mirrors how [`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/go.mod)
pins specific module versions to guarantee consistent behaviour between nodes.
For local testing we recommend using the exact git revision of the
`cosmwasm-vm` crate that is known to compile with the rest of the workspace.

### Feature Flags

The manifest should expose a set of features that allow consumers to tailor the
build footprint. We replicate `wasmvm`'s feature set as well as a few extra
flags for optional client components:

* `default = ["staking"]` – builds with staking and bank integration enabled.
* `grpc` – compiles the gRPC service implementation found in `client/grpc.rs`.
* `rest` – pulls in `axum` and enables the HTTP routes defined in
  `client/rest.rs`.
* `cli` – enables clap-based CLI commands under `client/cli`.

Each feature should be documented inside the manifest with comments so that
downstream crates understand the default capabilities. In CI we will test both
the default features and a minimal `--no-default-features` build to ensure the
optional components do not introduce hidden dependencies.

### Workspace Integration

The `x/wasm` crate is part of a larger Cargo workspace. To enable integration
tests across crates we expose a `[[test]]` target referencing the example
contracts in `assets/`. This test harness depends on `cosmwasm-schema` for
verifying query and execute message shapes. The manifest also sets
`rust-version = "1.79"` to align with the repository's MSRV as described in the
root `AGENTS.md`.

When building release binaries the `panic` strategy is set to `abort` to reduce
binary size. This is consistent with the build approach in the
[`wasmvm`](https://github.com/CosmWasm/wasmvm/blob/main/Makefile) project and
ensures deterministic behaviour during contract execution.

## Rationale

The structure outlined above provides a clear mapping between Gears' Rust-based
module and the Go-based implementation in `wasmd`. By closely mirroring the
feature flags and dependency structure found in the
[`CosmWasm VM`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)
repository we reduce friction when porting code or updating to new versions. The
manifest acts as the foundation for all other source files in the `x/wasm`
directory, so a precise specification here avoids many integration issues later
in development.

