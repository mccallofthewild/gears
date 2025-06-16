# ADR: Implement gRPC services for `x/wasm`

## Status
Proposed

## Context

External tools such as dashboards and SDK clients interact with nodes via gRPC.
`wasmd` defines protobuf service definitions and server implementations under
`x/wasm/client`. Gears will provide similar services using `tonic`.

## Decision

`client/grpc.rs` will define a `WasmService` implementing gRPC methods for the
query types in `types/query.rs` and transaction helpers for broadcasting
`message::Message` variants. The service will use the keeper internally and will
be registered with the node's gRPC router.

### Service Definition

Protobuf definitions will reside under `proto/cosmwasm/wasm/v1/query.proto` and
`tx.proto` similar to the files in
[`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/proto/cosmwasm/wasm/v1).
`build.rs` will generate Rust structs via `tonic-build`.

The `WasmService` struct implements the generated `wasm_server::Wasm` trait with
methods such as `ContractInfo`, `SmartContractState`, `StoreCode` and so on.
Each method converts the protobuf request into the corresponding keeper call and
wraps the result in a `tonic::Response`.

### Registration and Auth

The service is added to the node's router using `grpc::Server::add_service` in
the `client` module. Authentication for broadcast endpoints will rely on the
existing `TxService` rather than custom auth logic. Query endpoints are read
only and accessible to all clients.

## Consequences

Implementing gRPC ensures compatibility with standard Cosmos tooling and allows
programmatic access to contract queries and transactions.

## Implementation Details

### Service Construction

The `WasmService` struct will hold a cloned `Keeper` instance and implement the
generated `wasm_server::Wasm` trait. Each method begins by converting the
incoming protobuf request to the internal query or message type using the
`FromProto` derive. After invoking the keeper, results are wrapped into the
corresponding protobuf response struct using `IntoProto` and returned via
`Ok(Response::new(proto))`.

### Streaming Responses

Some queries, such as listing all contracts by code ID, may return a stream of
results. `tonic` supports streaming via `Response<tonic::codec::Streaming<_>>`.
Following [`wasmd`'s gRPC server](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/client/grpc/server.go),
the Rust implementation will page through results using `keeper.contracts_by_code` and send
each item through a channel to the client. This design prevents large responses
from exhausting memory.

### Error Mapping

Errors returned by the keeper must be converted into gRPC `Status` objects. A
helper `to_status(err: WasmError)` function maps common cases to canonical codes
(e.g. `NotFound` -> `not_found`, `Unauthorized` -> `permission_denied`). The
error messages should include the contract address or code ID when relevant to
aid debugging.

### Server Registration

`register_grpc` in `client/mod.rs` will call `Server::builder().add_service(...)`.
This function should also configure reflection if the `grpc-reflection` feature
is enabled, mirroring the optional server reflection support in `wasmd`. Secure
transport (TLS) can be configured using the same `tonic` APIs as other modules.

### Testing

Integration tests spawn a gRPC server in the background and issue queries using
the generated gRPC client. Contract responses are compared against the output of
`wasmd` running the same test contracts to ensure behavioural parity. Mocking the
keeper allows testing error paths without running a full node.

## Future Considerations

As CosmWasm evolves, new query and message types will be added to the protobuf
definitions. The gRPC service should be kept up to date with these changes to
maintain interoperability. Additionally, features such as rate limiting or
authentication middleware may be incorporated to better secure public nodes.


