# ADR: Implement `x/wasm::params`

## Status
Proposed

## Context

The wasm module requires configurable parameters such as the maximum gas for
smart queries and the in-memory cache size for compiled code. `wasmd` exposes
these via a `Params` struct and a `ParamsKeeper` helper.

## Decision

`params.rs` will define a `Params` struct with fields like `query_gas_limit` and
`memory_cache_size` following the configuration found in
[`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/types/params.proto).
A `WasmParamsKeeper` will implement `gears::params::ParamsKeeper` so the values
are stored in the module's parameter subspace.

### Required Fields

* `code_upload_access` – Controls who may store new code. Mirrors the
  `AccessConfig` used by wasmd and should implement conversions from the
  protobuf enum.
* `instantiate_default_permission` – Default instantiation permission for new
  code. When unset, only the uploader may instantiate. Implementation follows
  `wasmd/types/params.pb.go` behaviour.
* `max_contract_size` – Limit (in bytes) for uploaded wasm binaries. Helps
  mitigate memory usage of the VM.
* `query_gas_limit` – Maximum gas allowed for smart queries executed during ABCI
  queries. Should default to the value recommended in the
  [CosmWasm VM README](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm#query-gas-limit).
* `memory_cache_size` – Number of compiled modules cached in memory for faster
  instantiation. Exposed through the keeper to allow dynamic resizing.

### ParamsKeeper Integration

`WasmParamsKeeper` exposes `get_params` and `set_params` methods used by the
keeper at startup and during parameter change proposals. The struct will map to
the `ParamsSubspace` key defined in the module and store serialized `Params`
using the `kv_store` crate. Unit tests will ensure default values match the
ones shipped in wasmd's genesis file.

## Consequences

Parameter handling integrates the module with the existing parameters keeper and
allows node operators to tune wasm behaviour.

## Implementation Details

### Defaults and Validation

The `Params` struct should implement `Default` with values that match those used
in the `wasmd` genesis file. For example, `max_contract_size` defaults to
1_000_000 bytes, while `query_gas_limit` defaults to 3_000_000. Each field's
documentation should cite the upstream constant in
[`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/types/params.go) so
that future updates remain consistent. Validation functions enforce bounds when
parameters are updated via governance proposals.

### Serialization Format

`Params` needs to implement `Encode` and `Decode` using `prost` to support
protobuf-based parameter storage. Derive macros from `core-types` provide these
implementations automatically. The `Params` struct must also derive
`serde::Serialize` and `Deserialize` to support CLI configuration files and REST
endpoints. When serializing to JSON, coin amounts and addresses use the same
string formats as other modules.

### Dynamic Updates

`WasmParamsKeeper` exposes an `on_update` callback that is triggered when a
parameter change proposal is executed. This callback notifies the `WasmEngine`
so it can adjust its cache size or update compile options. The design mirrors
the dynamic parameter updates found in `wasmvm`'s `lib.go` where the VM cache can
be resized at runtime.

### CLI Integration

The `client/cli` module will provide a `wasm params` subcommand that prints the
current parameter values and allows updates via governance proposals. This uses
the `get_params` and `set_params` methods from `WasmParamsKeeper`. Example
commands replicate those in the `wasmd` documentation so operators familiar with
the Go-based CLI can reuse their workflows.

### Testing

Unit tests should verify that default values round trip through serialization
and that invalid updates are rejected. Integration tests running a full node can
submit parameter change proposals and confirm that the keeper and engine react
appropriately. Refer to `wasmd/x/wasm/keeper/params_test.go` for a suite of test
cases that can be ported to Rust.

## Rationale

Clearly defining parameters and update mechanisms ensures that contract
execution remains predictable across network upgrades. By following the
structure of `wasmd` and the guidelines in the
[`CosmWasm VM` repository](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm),
the module stays aligned with the broader Cosmos ecosystem.

