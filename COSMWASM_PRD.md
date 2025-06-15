# CosmWasm Integration Product Requirements Document

## Overview

This document describes the requirements for introducing a new `wasm` module under `x/wasm`. The goal is to enable executing [CosmWasm](https://cosmwasm.com/) smart contracts directly within the Gears framework.

Gears will depend on [`cosmwasm_vm`](https://crates.io/crates/cosmwasm_vm) and implement the entire interface required to store, instantiate, execute and query contracts. Unlike [wasmvm](https://github.com/CosmWasm/wasmvm), which exposes a Go wrapper around the CosmWasm VM via cgo, our implementation will call the Rust VM directly without external bindings.

## Background

[`wasmvm`](https://github.com/CosmWasm/wasmvm) contains both Rust and Go code. The Rust code compiles into a shared library that is called from Go through the FFI. Key structures are generated via `cbindgen` and defined in [`bindings.h`](https://github.com/CosmWasm/wasmvm/blob/main/internal/api/bindings.h). Go implements wrapper types and exposes a `VM` struct in [`lib_libwasmvm.go`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go) when cgo is enabled.

Important bindings include:

* `cache_t` – an opaque handle to the VM cache. Created by `init_cache` and released via `release_cache`.
* `Db`/`DbVtable` – callbacks for reading, writing and iterating over contract storage from Go. Implemented in [`callbacks.go`](https://github.com/CosmWasm/wasmvm/blob/main/internal/api/callbacks.go).
* `GoApi`/`GoApiVtable` – address conversion functions (humanize, canonicalize, validate) passed to the VM.
* `GoQuerier`/`QuerierVtable` – performs external queries on behalf of the contract.
* Execution entry points such as `instantiate`, `execute`, `migrate`, `query`, and numerous IBC hooks defined in `bindings.h`.
* Data types like `Env`, `MessageInfo`, `ContractResult`, `QueryResult` etc. are defined in [`types`](https://github.com/CosmWasm/wasmvm/tree/main/types) to mirror the Rust structures.

The Go `VM` struct wraps all these FFI calls and provides convenience methods for the SDK module [`x/wasm`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm). For example, [`StoreCode`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go) validates and caches WASM bytes, while [`Instantiate`](https://github.com/CosmWasm/wasmvm/blob/main/lib_libwasmvm.go) marshals Go types to JSON, invokes the FFI, and returns a `ContractResult`.

## Opportunity in Gears

In Gears we can interface with `cosmwasm_vm` directly in Rust. This avoids cgo overhead and simplifies dependency management. The new module will still need to provide the same functionality as `x/wasm` in wasmd:

* A keeper managing contract code, instances and state.
* Methods to store, instantiate, execute, migrate and query contracts.
* Integration with the bank module for coin transfers.
* Gas metering and limits matching CosmWasm semantics.
* Optional IBC callbacks if compiled with IBC support.

## Requirements

1. **Crate layout**
   - Create a new crate under `x/wasm` named `wasm` (working name).
   - Add `cosmwasm_vm` as a dependency in `Cargo.toml`.
   - Expose a public `Keeper` and `WasmEngine` trait similar to [`wasmd/x/wasm`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm).

2. **VM integration**
   - Use the `cosmwasm_vm` `Instance` and `InstanceOptions` types directly.
   - Implement storage, API and querier traits that bridge Gears’ KV stores and modules to the VM.
   - Provide serialization helpers for `Env`, `MessageInfo`, `QueryRequest`, etc., mirroring the structures from `cosmwasm_std`.

3. **State management**
   - Store compiled code and contract state within the multistore using new store keys.
   - Support pinning/unpinning code similar to `wasmvm` caches, but implemented in Rust.

4. **Gas metering**
   - Wrap `cosmwasm_vm::Backend` to meter gas usage. Match the gas costs used in `wasmvm` so execution fees are compatible.

5. **IBC support** (optional in initial version)
   - If compiled with the `ibc` feature, implement hooks for `ibc_channel_open`, `ibc_packet_receive`, etc., leveraging `cosmwasm_vm::ibc` helpers.

6. **Testing**
   - Unit tests for the keeper functions.
   - Integration tests with example contracts to validate instantiate/execute/query flows.

## Non-Goals

* We do not reimplement the CosmWasm VM. We rely on `cosmwasm_vm` for execution.
* Direct FFI to Go is unnecessary since the entire stack is Rust.

## Open Questions

* Which subset of wasmd’s features should be included in the first milestone (e.g. reply, sudo, IBC v2)?
* How will contract code be migrated or pinned across nodes without the existing `wasmvm` cache layout?

## References

* [CosmWasm VM](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)
* [wasmvm Go bindings](https://github.com/CosmWasm/wasmvm)
* [wasmd wasm module](https://github.com/CosmWasm/wasmd/tree/main/x/wasm)

