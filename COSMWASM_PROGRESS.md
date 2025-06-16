# CosmWasm Module Implementation Checklist

Below is the recommended order for implementing the files within `x/wasm`. Each item now includes subtasks derived from the accompanying ADRs.

- [x] **x/wasm/Cargo.toml** – crate manifest defining dependencies for the module. It underpins compilation of all subsequent files.
  - [x] Declare mandatory dependencies and VM features.
    - [x] Add `cosmwasm-vm`, `cosmwasm-std`, `serde`, `serde_json`, `thiserror`, `anyhow` and `gears`.
    - [x] Use workspace paths for `core-types`, `gas` and `tendermint`.
  - [x] Configure optional features for clients and VM extensions.
    - [x] Provide `grpc`, `rest` and `cli` toggles in addition to `stargate`, `iterator` and `IBC`.
  - [x] Integrate test harness and release settings.
    - [x] Include a `[[test]]` target with `cosmwasm-schema` and set `rust-version` plus `panic = "abort"`.

- [x] **x/wasm/src/message.rs** – transaction message structures such as `MsgStoreCode` and `MsgInstantiateContract`.
  - [x] Define the `Message` enum covering store, instantiate, execute, migrate and admin updates.
    - [x] Ensure variants map one-to-one with `wasmd` messages.
  - [x] Derive serialization and protobuf conversions.
    - [x] Apply `#[serde(tag = "@type")]` and the `FromProto`/`ToProto` macros.
  - [x] Implement `validate_basic()` tests for each variant.

- [x] **x/wasm/src/types/query.rs** – request and response types for contract queries.
  - [x] Implement structs `QuerySmartContractState`, `QueryRawContractState`, `QueryCode`, `QueryContractInfo` and `QueryContractsByCode`.
    - [x] Use `Address` and `Binary` fields matching the ADR specification.
  - [x] Add the `WasmQuery` enum implementing `AppQuery` with serde examples.
    - [x] Document pagination defaults using `PageRequest` and `PageResponse`.
  - [x] Provide unit tests for JSON and protobuf round‑trips.

- [x] **x/wasm/src/types/mod.rs** – exposes the query submodule for external use.
  - [x] Re-export `query` and commonly used structs at the module root.
    - [x] Include rustdoc links to `wasmd` type definitions.
  - [x] Keep the public API stable and show example imports.

- [x] **x/wasm/src/params.rs** – module parameters controlling wasm behaviour.
  - [x] Create the `Params` struct with fields like `code_upload_access`, `query_gas_limit` and `memory_cache_size`.
    - [x] Provide `Default` values mirroring the `wasmd` genesis file.
  - [x] Implement `WasmParamsKeeper` with `get_params`, `set_params` and `on_update`.
    - [x] Notify the engine when parameters change.
  - [x] Add CLI support for displaying and updating params.

- [x] **x/wasm/src/error.rs** – common error enum for the wasm module.
  - [x] Define `WasmError` variants for compile, runtime, not found, unauthorized, invalid request and internal failures.
    - [x] Implement `From<VmError>` and other conversions.
  - [x] Map variants to ABCI codes and provide `Display` messages.
    - [x] Unit test error mappings and logging output.

- [x] **x/wasm/src/engine.rs** – defines the `WasmEngine` trait and a `CosmwasmEngine` skeleton.
  - [x] Specify trait methods mirroring `wasmvm` (`store_code`, `analyze_code`, `instantiate`, `execute`, `migrate`, `query`, `sudo`, `reply`, `ibc_*`).
    - [x] Implement `CosmwasmEngine` using `cosmwasm_vm::Vm` and a disk cache.
  - [x] Handle memory limits, gas accounting and code analysis.
    - [x] Convert `VmError` into `WasmError` and guard the cache with synchronization primitives.
  - [x] Document example usage and note possibilities for alternative engines.

- [ ] **x/wasm/src/keeper.rs** – core keeper managing state and delegating execution to a `WasmEngine`.
  - [x] Set up stores for code, contracts, sequences and `code_index` as described in the ADR.
    - [x] Provide helper functions for key derivation compatible with `wasmd`.
  - [ ] Implement contract lifecycle methods (`store_code`, `instantiate`, `execute`, `query`, `migrate`, admin updates, `contracts_by_code`).
    - [ ] Integrate parameter access and gas metering with the engine.
  - [ ] Support concurrency via interior mutability and interact with bank and IBC keepers.

- [ ] **x/wasm/src/genesis.rs** – handles loading and exporting module state at genesis using the keeper.
  - [ ] Define `GenesisState` and implement `init_genesis` plus `export_genesis`.
    - [ ] Validate code checksums and ensure deterministic ID assignment.
  - [ ] Pre-populate the engine cache and reconstruct contract state.
    - [ ] Export data in sorted order for reproducible genesis files.
  - [ ] Add end‑to‑end tests verifying import/export round‑trips.

- [ ] **x/wasm/src/abci_handler.rs** – ABCI entry points wiring transactions and queries to the keeper and message types.
  - [ ] Implement an `ABCIHandler` struct holding a `Keeper` reference.
    - [ ] `deliver_tx` decodes `Any` messages and dispatches to keeper methods with gas accounting.
  - [ ] Handle `WasmQuery` requests and return JSON via `serde_json`.
    - [ ] Provide begin and end block hooks for future metrics.
  - [ ] Test message routing and error code mapping.

- [ ] **x/wasm/src/client/cli/query.rs** – CLI subcommands for querying wasm state.
  - [ ] Add commands for smart queries, raw storage, code download, contract info and contract lists.
    - [ ] Accept JSON inline or from files and validate Bech32 addresses.
  - [ ] Format results as JSON or write binaries to disk.
    - [ ] Cover parsing and network errors in unit tests.

- [ ] **x/wasm/src/client/cli/tx.rs** – CLI subcommands for broadcasting wasm transactions.
  - [ ] Implement `store`, `instantiate`, `execute`, `migrate`, `update-admin` and `clear-admin` commands.
    - [ ] Parse coin amounts and WASM files, supporting `--broadcast-mode`.
  - [ ] Attach gas and fee options and print transaction hashes.
    - [ ] Ensure graceful error handling when validation fails.

- [ ] **x/wasm/src/client/cli/mod.rs** – groups the query and transaction CLI into a single module.
  - [ ] Provide a `command()` function returning the root `wasm` command.
    - [ ] Register the `tx` and `query` subcommand trees with helpful `about` strings.
  - [ ] Enable shell completion generation and document examples.

- [ ] **x/wasm/src/client/grpc.rs** – gRPC service definitions exposing query and transaction helpers for external tooling.
  - [ ] Implement `WasmService` using the generated protobuf traits.
    - [ ] Convert requests with `FromProto` and stream paginated results when necessary.
  - [ ] Map `WasmError` to `tonic::Status` and register the service with the server.
    - [ ] Include integration tests against a mock keeper.

- [ ] **x/wasm/src/client/rest.rs** – REST handlers mirroring the gRPC interface for web applications.
  - [ ] Build `axum` routes for contract queries and transaction helpers.
    - [ ] Reuse `WasmService` functions and apply middleware such as CORS.
  - [ ] Return JSON responses and proper HTTP status codes.
    - [ ] Document TLS options and deployment behind a reverse proxy.

- [ ] **x/wasm/src/client/mod.rs** – aggregates CLI, gRPC and REST interfaces for consumers.
  - [ ] Re-export submodules conditionally via `cli`, `grpc` and `rest` features.
    - [ ] Provide `register_grpc` and `register_routes` helpers.
  - [ ] Demonstrate usage in application builders and test feature combinations.

- [ ] **x/wasm/src/lib.rs** – module root re-exporting the keeper, engine, clients and other components; depends on all previous files.
  - [ ] Declare all submodules and public re-exports.
    - [ ] Gate optional clients behind feature flags.
  - [ ] Supply crate-level documentation linking to CosmWasm VM, wasmvm and wasmd.
    - [ ] Include test utilities for spawning an in-memory engine and keeper.
