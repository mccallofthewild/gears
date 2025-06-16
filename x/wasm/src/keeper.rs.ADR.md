# ADR: Implement `x/wasm::keeper`

## Status
Proposed

## Context

The keeper is responsible for persisting contract code and state, enforcing
module parameters and delegating execution to the `WasmEngine`. `wasmd`
implements this logic under [`keeper`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/keeper).

## Decision

`keeper.rs` will define a generic `Keeper` struct parameterised over a `StoreKey`
and `ParamsSubspaceKey`. It will manage two substores: one for contract code and
one for contract instances. Methods will include `store_code`, `instantiate`,
`execute`, `query`, `migrate` and admin updates. The keeper will hold a boxed
`dyn WasmEngine` so alternate implementations can be swapped in.

### Storage Layout

The keeper mirrors the design in
[`wasmd/x/wasm/keeper`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm/keeper):

* `codes` store – maps code ID to wasm bytecode and metadata.
* `contracts` store – maps contract address to `ContractInfo` including code ID
  and admin.
* `sequences` store – maintains counters for assigning code IDs and contract
  addresses.
* `code_index` store – for each code ID stores the list of contract addresses
  instantiated from it. This powers the `contracts_by_code` query and mirrors
  the secondary index in [`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/keeper/keeper.go#L92).

Keys are stored using `kv_store::PrefixedStore` with constants defined in
`constants.rs` (to be created). The `StoreKey` type parameter allows embedding
these stores into the application's `MultiStore`.

### Core Methods

1. `store_code` – Saves raw wasm code, performs optional checksum validation and
   assigns a new code ID. Persists metadata such as uploader address. Calls
   `engine.analyze_code` if enabled by module parameters.
2. `instantiate` – Creates a new contract instance. Reserves a contract address
   from the sequences store, writes initial state to the `contracts` store and
   delegates execution of the init message to the engine with appropriate gas
   limits.
3. `execute` – Invokes a contract with a provided message and funds. Uses the
   engine to run the call and then records emitted events and submessages.
4. `query` – Invokes the smart contract `query` entry point using a read-only
   store and returns JSON `Binary` data.
5. `migrate` – Updates a contract to use a new code ID. Checks admin
   authorization and stores a migration history entry for auditing.
6. `update_admin` / `clear_admin` – Manage contract admin addresses.
7. `contracts_by_code` – Returns an iterator over all contracts created from a
   given code ID. Uses the `code_index` store to gather addresses for pagination
   queries.

### Parameter and Engine Integration

The keeper holds a `WasmParamsKeeper` instance to read and update parameters at
runtime. It also caches a boxed `dyn WasmEngine` which may be replaced by tests
or alternate implementations (e.g. a faster engine). Gas metering integrates via
`cosmwasm_vm`'s gas system hooking into the `gas` crate.

## Consequences

A fully featured keeper enables the module to participate in genesis loading,
ABCI message handling and cross-module queries.

## Implementation Guidelines

### Key Derivation

Store keys should be computed using deterministic prefixes inspired by
[`wasmd`'s constants](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/types/keys.go).
For example, code information lives under `codes/{code_id}` and contract state
under `contract/{addr}`. Using the same prefixes allows data imported from a
`wasmd` chain to be recognised without transformation. The keeper will expose
helper functions like `code_key(code_id)` to centralise this logic.

### Gas Metering and Event Handling

Each keeper method accepts a mutable `TxContext` which includes a gas meter and
event manager. When delegating to the `WasmEngine`, the returned gas usage is
subtracted from the context to ensure contracts cannot exceed the configured
block limits. Events emitted by the engine are prefixed with the contract
address for easier tracing, following the pattern in
[`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/keeper/keeper.go#L419).

### Concurrency Concerns

The keeper must maintain thread safety when accessed from multiple ABCI
requests. All internal stores use `RefCell` or `RwLock` wrappers to permit safe
borrowing across asynchronous query handlers. The design mirrors the `Cosmos SDK`'s
multi-store concurrency model but adapted to Rust's ownership rules.

### Integration with Other Modules

Funds transferred to or from contracts rely on the bank keeper. The wasm keeper
stores a reference to `BankKeeper` and uses it to deduct coins from the sender
before calling `engine.execute`. Similarly, IBC callbacks interact with the IBC
keeper to send packets or open channels. Those interactions should mimic the
APIs exposed in `wasmd` to ease porting of existing modules.

## Testing

Comprehensive tests should cover each keeper method. Mock engines can simulate
successful and failing executions. Regression tests comparing state roots with a
`wasmd` reference chain ensure compatibility with existing tools. Benchmarks
should measure gas consumption for common workloads to validate that our costs
align with the values in the `CosmWasm VM` repository.

