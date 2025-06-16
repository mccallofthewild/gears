# ADR: Implement `x/wasm::abci_handler`

## Status
Proposed

## Context

The ABCI handler wires transaction and query routing into the module's keeper.
`wasmd` uses a `Handler` type that interprets `Message` variants and dispatches
queries. Gears' application framework expects each module to implement
`application::handlers::node::ABCIHandler`.

## Decision

`abci_handler.rs` will implement an `ABCIHandler` struct containing a `Keeper`.
The `deliver_tx` method will match on `message::Message` variants and invoke the
corresponding keeper methods. Query handling will accept `types::query` requests
and return JSON responses encoded using `serde_json`. Block lifecycle hooks are
initially left unimplemented but stubbed for future use.

### Transaction Processing

`deliver_tx` follows the pattern used in
[`wasmd/x/wasm/handler.go`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/handler.go):

1. Deserialize incoming `Any` messages into our `Message` enum using the
   `FromProto` trait.
2. For each variant, call the respective keeper method (`store_code`,
   `instantiate`, `execute`, etc.) and capture `Response` events emitted by the
   `WasmEngine`.
3. Map `WasmError` into `tendermint::abci::ResponseDeliverTx` codes consistent
   with the Cosmos SDK (e.g. unauthorized -> `CodeUnauthorized`).

### Query Handling

`query` method will parse `WasmQuery` requests and either call into the keeper
or directly invoke the engine for smart queries. Results are encoded as JSON
strings. Pagination and height handling mirror the `query` server in wasmd.

### Block Hooks

Placeholders for `begin_block` and `end_block` will be added to support future
features like pinned code management or per-block metrics collection.

## Consequences

The handler connects the wasm module to the base application, enabling contract
operations via transactions and queries.

## Implementation Outline

The `ABCIHandler` struct stores a reference to the module `Keeper` and optional
configuration such as a `log_level` for debugging. It implements the
`ABCIHandler` trait defined in `application::handlers::node`, which requires
methods for transaction delivery, query execution, and optional block lifecycle
hooks. Below we detail the recommended logic for each method.

### `deliver_tx`

1. **Decode Messages** – Each incoming transaction may contain multiple `Any`
   messages. The handler iterates over these and converts them into the local
   `Message` enum via `Message::from_proto(any_msg)`.
2. **Dispatch** – Using a match statement, the handler calls the corresponding
   keeper method. For example, `Message::StoreCode` triggers
   `keeper.store_code(ctx, msg)`. Each call returns a `Result<Response,
   WasmError>`.
3. **Gas Accounting** – The handler should deduct gas for each operation based
   on the metrics returned by the `WasmEngine`. Gas costs match the tables in the
   [CosmWasm VM repository](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)
   and should be updated when migrating to new VM versions.
4. **Events** – Success results in events from the engine being appended to the
   context’s event manager. These follow the schema defined in
   [`wasmd/events.go`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/types/events.go).
5. **Error Mapping** – Any `WasmError` is translated into an ABCI code using the
   error categories defined in `error.rs`. This ensures parity with the Go
   implementation where, for instance, `ErrNotFound` maps to code `5`.

### `query`

1. **Parse Query** – Incoming ABCI query data is deserialized into the
   `WasmQuery` enum using `Query::from_slice`. Support both JSON and Bincode
   depending on the `path` field of the request.
2. **Execute** – Depending on the variant, call the corresponding keeper method
   (`keeper.query_smart`, `keeper.query_raw`, etc.). Smart queries use a read-only
   `QueryContext` to avoid state mutations.
3. **Encoding** – Results are serialized as JSON using `serde_json`. Height
   information from the context is included in the `ResponseQuery` to signal the
   state version.

### Block Hooks

While initially empty, `begin_block` and `end_block` should log metrics about
contract execution, such as total gas consumed and number of contracts invoked.
Future versions may use these hooks to run housekeeping tasks like pruning the
VM cache or migrating pinned codes.

## Testing Strategy

Unit tests should cover the mapping from messages to keeper calls and ensure
error codes align with [`wasmd`'s handler tests](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/handler_test.go).
Integration tests executed via the CLI will verify that transactions broadcast
through this handler produce the same events and state changes as `wasmd`. Mock
engine implementations can be injected to simulate VM failures without running
actual WASM code, following patterns established in the
[`wasmvm`](https://github.com/CosmWasm/wasmvm) test suite.

