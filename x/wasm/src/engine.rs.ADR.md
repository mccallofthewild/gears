# ADR: Implement `x/wasm::engine`

## Status
Proposed

## Context

The keeper delegates contract execution to a dedicated engine. In `wasmd` this is
backed by the `wasmvm` `VM` type. In Gears we will use
[`cosmwasm-vm`](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)
directly as outlined in [COSMWASM_ADR.md](../COSMWASM_ADR.md).

## Decision

`engine.rs` will define a `WasmEngine` trait with methods mirroring the Go
bindings: `store_code`, `analyze_code`, `instantiate`, `execute`, `migrate`,
`query`, `sudo`, `reply` and the IBC callbacks. A default `CosmwasmEngine` struct will implement
this trait using `cosmwasm_vm::Vm` and the native backend implementations for
storage, API and gas metering.

### Trait Methods

The trait will be heavily inspired by the
[`VM` interface in wasmvm](https://github.com/CosmWasm/wasmvm/blob/main/lib.go).
Required methods include:

* `store_code(store: &mut dyn Storage, wasm: &[u8]) -> Result<CodeId, WasmError>`
* `analyze_code(wasm: &[u8]) -> Result<CodeFeatures, WasmError>`
* `instantiate(store: &mut dyn Storage, code_id: CodeId, msg: Binary, info: MessageInfo, env: Env) -> Result<Address, WasmError>`
* `execute(store: &mut dyn Storage, contract: Address, msg: Binary, info: MessageInfo, env: Env) -> Result<Response, WasmError>`
* `migrate(store: &mut dyn Storage, contract: Address, new_code_id: CodeId, msg: Binary, info: MessageInfo, env: Env) -> Result<Response, WasmError>`
* `query(store: &dyn Storage, contract: Address, msg: Binary, env: Env) -> Result<Binary, WasmError>`
* `sudo`, `reply`, plus `ibc_*` callbacks following the same signature shapes.

Each method mirrors the corresponding call in `cosmwasm_vm::Vm` so that
`CosmwasmEngine` simply delegates to an inner `Vm` instance. The trait is
object-safe allowing alternative engines (e.g. mocks for tests or a fast native
VM) to plug into the keeper.

### Engine Creation and Caching

`CosmwasmEngine` will expose a constructor that takes a
`cosmwasm_vm::cache::Cache` path and configuration options such as memory limit
and debug features. The cache will be managed similarly to the `InitCache`
function in wasmvm, storing compiled modules on disk and loading them on demand.

The engine must implement thread safety so that multiple keepers or ABCI calls
can execute concurrently. This is achieved using the sync primitives provided by
`cosmwasm_vm` around its `Cache` struct.

### Code Analysis

`analyze_code` parses an uploaded wasm binary and returns a `CodeFeatures`
structure describing which optional VM capabilities are used (e.g. IBC entry
points or required iterator support). The keeper consults this when verifying
`store_code` operations. The implementation delegates to
`cosmwasm_vm::analysis::analyze` so the results match those of the Go
`wasmvm`.

## Consequences

Abstracting the execution engine allows future engines or mock engines for tests
without changing the keeper logic.

## Additional Details

### Memory Management and Gas Costs

`CosmwasmEngine` must carefully track memory allocations so that contract
execution cannot exceed the limits configured in `Params`. The underlying VM
uses `Region` structures to manage guest memory; these correspond directly to
the concepts outlined in the
[`cosmwasm-vm` source](https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/src/memory.rs).
When creating a new `Vm`, the engine passes a `MemoryLimiter` implementation that
reads gas usage from the `gas` module. This ensures parity with the gas model in
`wasmvm`'s Go bindings where each allocation consumes gas.

### Error Propagation

All trait methods should convert low-level `VmError` variants into our unified
`WasmError` enum defined in `error.rs`. For example, `VmError::CompileErr` maps
to `WasmError::CompileErr`. This mapping must be consistent so that the ABCI
handler can convert errors into deterministic ABCI codes. The `cosmwasm_vm` API
also exposes detailed backtraces when the `backtraces` feature is enabled. The
engine will expose a configuration flag to toggle these in order to reduce noise
in production environments.

### Multithreaded Access

The keeper may execute multiple contracts concurrently during block processing.
Therefore, `CosmwasmEngine` should wrap the internal `Cache` and `Vm` objects in
a `std::sync::Mutex` or `RwLock`. The design can follow the concurrency model in
[`wasmvm`](https://github.com/CosmWasm/wasmvm/blob/main/lib.go) where the cache
is guarded by a `sync.Mutex`. Benchmarks should be added to ensure the lock does
not become a bottleneck when many contracts are instantiated simultaneously.

### Example Usage

```rust
let engine = CosmwasmEngine::new(cache_path, EngineOptions::default());
let code_id = engine.store_code(&mut store, &wasm_bytes)?;
let address = engine.instantiate(&mut store, code_id, init_msg, info, env)?;
let resp = engine.execute(&mut store, address, exec_msg, info, env)?;
```

This snippet demonstrates the basic lifecycle from code storage to execution and
mirrors the flow shown in `wasmd`'s keeper tests.

### Future Extensions

Because the `WasmEngine` trait is object-safe, we can add alternative
implementations, such as an ahead-of-time compiled engine or a mocked engine for
unit tests. Implementors must adhere to the semantics described in the
[`CosmWasm VM` documentation](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)
to remain compatible with stored contracts.

