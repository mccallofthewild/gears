# ADR: Integrating CosmWasm via `cosmwasm_vm`

## Status
Proposed

## Context

The [CosmWasm Integration PRD](COSMWASM_PRD.md) outlines the need for a new `x/wasm` crate that executes smart contracts using the `cosmwasm_vm` crate. The reference implementation in Go uses [`wasmvm`](https://github.com/CosmWasm/wasmvm), which compiles Rust code to a C library and wraps it in Go. Our Rust code can directly depend on `cosmwasm_vm`, eliminating the cgo layer.

### wasmvm Analysis

`wasmvm` exposes a C API defined in [`bindings.h`](https://github.com/CosmWasm/wasmvm/blob/main/internal/api/bindings.h). Go wrappers in [`internal/api`](https://github.com/CosmWasm/wasmvm/tree/main/internal/api) provide implementations for storage, address conversion and queries via structs such as `Db` and `GoApi`. The `VM` type in [`lib_libwasmvm.go`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go) offers high level methods like `StoreCode`, `Instantiate`, `Execute`, `Query`, `Migrate`, `Sudo`, `Reply` and the full suite of IBC callbacks. These functions:

- Marshal Go structs (e.g. `Env`, `MessageInfo`) to JSON
- Allocate `Db`, `GoApi` and `GoQuerier` vtables pointing to callback functions in [`callbacks.go`](https://github.com/CosmWasm/wasmvm/blob/main/internal/api/callbacks.go)
- Call into the Rust VM via cgo functions such as `C.instantiate` or `C.execute`
- Deserialize results using `DeserializeResponse` and report gas usage

The callbacks manage iterators and gas metering by passing a `gas_meter_t` pointer to Rust. They map Go interfaces (`KVStore`, `GasMeter`, `Querier`) to the VM's `Backend` abstraction. Each FFI call uses `startCall`/`endCall` to track iterator lifetimes.

### wasmd References

`wasmd` defines keepers and message handlers in [`x/wasm`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm). Interfaces like `ContractOpsKeeper` describe methods for uploading code, instantiating contracts, executing calls and handling IBC packets. The keeper ultimately delegates to the `wasmvm` `VM` type.

## Decision

We will implement a Rust-native equivalent of the Go `VM` by directly using the `cosmwasm_vm` crate. The new `x/wasm` crate will depend on `cosmwasm_vm` and expose a `WasmEngine` trait with methods matching those in the Go bindings:

- `store_code`, `store_code_unchecked`, `remove_code`, `get_code`
- `pin`, `unpin`, `analyze_code`, `get_metrics`, `get_pinned_metrics`
- `instantiate`, `execute`, `migrate`, `migrate_with_info`
- `sudo`, `reply`, and all IBC hooks (`ibc_channel_open`, `ibc_channel_connect`, ...)

The implementation will use `cosmwasm_vm::Vm` and the `Backend` trait. We will build concrete backend types that map to Gears abstractions:

* **Storage** – wrap `kv_store` interfaces to implement `cosmwasm_vm::Storage`. This replaces `Db` and `iterator` callbacks.
* **API** – implement `Api` for address conversion using Gears’ Bech32 utilities, mirroring `GoApi` logic in `callbacks.go` (`humanize_address`, `canonicalize_address`, `validate_address`).
* **Querier** – implement `Querier` to route queries to other Gears modules, similar to `GoQuerier` calling into the Cosmos SDK.
* **Gas Meter** – implement `cosmwasm_vm::gas::GasMeter` that wraps the existing `gas` crate. This plays the role of the `gas_meter_t` passed through FFI.

Instead of `startCall` and iterator IDs, the Rust backend will manage iterators with Rust lifetimes. Caches for compiled modules will be handled using `cosmwasm_vm::cache::Cache` just like `InitCache`/`ReleaseCache` in Go, but without the C pointer layer.

The keeper inside `x/wasm` will own a `Cache` instance and expose the high level `ContractOpsKeeper` methods defined in `wasmd`. Contract info and code bytes will be stored under dedicated store keys in the multistore. The keeper will translate Gears messages into calls on the `WasmEngine` trait, attach appropriate gas limits and track events.

## Consequences

* Gears gains full CosmWasm support with no cgo dependency.
* We mirror the feature set of `wasmd` while staying within Rust.
* Gas accounting and iterator handling are implemented using native Rust types, simplifying memory management.
* Future updates to CosmWasm only require bumping the `cosmwasm_vm` crate version.

