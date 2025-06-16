# ADR: Implement `x/wasm::types` module

## Status
Proposed

## Context

To expose query types externally, Gears modules typically have a `types` module
with submodules for queries or other shared structs. The `crate::types` entry
point is used by the CLI, gRPC and REST layers.

## Decision

`mod.rs` will simply re-export the `query` submodule so that external crates can
import `wasm::types::query::*`. Additional type definitions may be added later.

The file also documents the stable API surface for client developers. Any
future additions such as message types or event definitions should be exposed
here to maintain a single location for type discovery. Documentation comments
will point to the upstream [`wasmd/x/wasm/types`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/types)
package so developers familiar with the Go implementation can easily find the
Rust equivalents.

## Consequences

Providing this entry point keeps the public API consistent with other modules and
matches the layout in [`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/types).

## Implementation Details

### Module Organization

The `types` module should be organised hierarchically so that additional type
definitions can be added in the future without breaking existing imports. Start
with a `pub mod query;` declaration and re-export commonly used structs at the
module root:

```rust
pub use self::query::{WasmQuery, QuerySmartContractState, QueryCode};
```

This approach mirrors the design used in other Gears modules (e.g. `x/bank`) and
makes the public API intuitive.

### Protobuf Integration

Query types are defined in `proto/cosmwasm/wasm/v1/query.proto`. During the
build process `tonic-build` generates Rust structs which are then wrapped by our
handwritten types to provide nicer ergonomics. For example,
`QueryContractInfoRequest` from protobuf maps to `QueryContractInfo` in this
module. Comments should clearly link each struct to its counterpart in the
`wasmd` source tree so that maintainers know where to look when updating
protos.

### Documentation

Each public type must include rustdoc comments that show example JSON encoding
and reference the underlying proto message. Following the style used in
[`cosmwasm-vm`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm), we
include a table summarizing the fields and their meaning. This documentation is
valuable for client developers who rely on the `types` module for generating API
requests.

### Stability

This module acts as the stable boundary for external dependencies. Once a type
is exposed here it should not be removed without a deprecation period. New types
should be added cautiously, with consideration for how they map to the Go
implementation so that cross-chain tooling remains compatible.

### Example Usage

```rust
use gears::x::wasm::types::QuerySmartContractState;

let req = QuerySmartContractState {
    address: "cosmos1...".parse().unwrap(),
    msg: to_binary(&MyQuery { ... }).unwrap(),
};
```

This snippet demonstrates constructing a query request using the types from this
module. The same struct can be serialized to JSON for CLI input or sent over
gRPC using the generated protobuf conversions.

## Rationale

By centralising type exports in `mod.rs` we provide a single location for users
to discover available queries and other data structures. This replicates the
developer experience of `wasmd` where all important types live under the `types`
package. Keeping the structure consistent makes migrating code from Go to Rust
much simpler.

## Future Work

As new query endpoints are introduced—such as advanced IBC packet tracking or
governance proposal metadata—corresponding request and response structs will be
added under this module. Each addition should include its own ADR describing the
functionality and referencing upstream changes in `wasmd` and
`cosmwasm-vm` to maintain parity.


