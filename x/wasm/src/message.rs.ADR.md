# ADR: Implement `x/wasm::message`

## Status
Proposed

## Context

CosmWasm transactions are submitted as SDK messages such as `MsgStoreCode` and
`MsgInstantiateContract`. `wasmd` defines these structs under
[`x/wasm/types`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/internal/types).
Gears will mirror these message types so that CLI, gRPC and REST clients can
construct and broadcast contract operations.

## Decision

`message.rs` will declare a top level `enum Message` implementing
`gears::derive::AppMessage`. Each variant corresponds to a transaction message
from the [wasmd module](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/internal/types/msgs.go).

### Required Variants

1. **StoreCode** – Uploads new WASM bytecode to the chain. The variant wraps a
   struct containing the sender address, a vector of raw bytes for the code,
   optional instantiate permission info and a source URL/checksum. The field
   layout should mirror `MsgStoreCode` from wasmd and derive `serde::Serialize`
   and `serde::Deserialize` for CLI and REST usage.
2. **InstantiateContract** – Creates a new contract instance from existing
   stored code. Fields include code ID, a JSON-encoded init `msg`, label,
   admin address and funds. Implementation must convert CLI JSON strings into
   `cosmwasm_std::Binary` for the VM.
3. **ExecuteContract** – Executes a method on a deployed contract. Contains the
   contract address, execution message, sender and attached funds. Both sync and
   asynchronous broadcasts are supported via `gears::client` helpers.
4. **MigrateContract** – Allows upgrading contract code to a new code ID.
   Includes the contract address, new code ID and migration JSON message. The
   keeper will verify that the caller is the contract admin before invoking the
   engine.
5. **UpdateAdmin** – Changes a contract's admin. Contains contract address,
   new admin and sender. If the admin is set to `None`, the action is considered
   `ClearAdmin` which removes admin privileges entirely.
6. **ClearAdmin** – Explicit variant for removing a contract admin. Provided for
   parity with wasmd even though it maps to `UpdateAdmin(None)` internally.

### Serialization Strategy

Each struct and the outer `Message` enum will derive `serde::Serialize` and
`Deserialize` with `#[serde(tag = "@type")]` on the enum so that message types
can be encoded like the protobuf `Any` representation used by Cosmos SDK.
The same approach is used in the `x/bank::message` module and ensures CLI and
gRPC clients can pass JSON payloads.

### Derive Macros

`Message` will derive `FromProto`/`ToProto` via macros from the
`macros/tx-derive` crate. This provides automatic conversion from the protobuf
definitions generated from the official `.proto` files in
[`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/proto/cosmwasm/wasm/v1).
It also enables command generation for `clap` via `macros::tx_derive::Message`.

## Consequences

Having a dedicated message enum keeps transaction routing consistent with the
rest of Gears. Contracts can be uploaded and executed using the same CLI tools as
other modules.

## Implementation Guidelines

### Protobuf Generation

Message structs must correspond to the protobuf definitions under
`proto/cosmwasm/wasm/v1/tx.proto`. Use `prost-build` in the workspace `build.rs`
to generate the types and enable `#[derive(FromProto, ToProto)]` on the Rust
structs via macros. By referencing the official `.proto` files from
[`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/proto/cosmwasm/wasm/v1)
we ensure wire compatibility with existing CosmWasm-based chains.

### Validation

Each message implements a `validate_basic()` method similar to the one in
`wasmd`. This performs lightweight checks such as verifying that addresses are
valid Bech32 strings and that JSON payloads do not exceed configured size limits
from the `Params` struct. Deeper validation requiring state access is performed
inside the keeper.

### Testing

Unit tests will construct each message variant and serialize it to JSON and
protobuf to verify that the encoding matches the expectations of external tools
(CLI, gRPC). Example payloads can be borrowed from `wasmd`'s integration tests to
ensure cross compatibility. Additional property tests may be added to assert that
round-tripping a message through serialization does not lose information.

