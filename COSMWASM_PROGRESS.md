# CosmWasm Module Implementation Checklist

Below is the recommended order for implementing the files within `x/wasm`. Each item includes a short description of its role and key files it interacts with.

- [x] **x/wasm/Cargo.toml** – crate manifest defining dependencies for the module. It underpins compilation of all subsequent files.
- [x] **x/wasm/src/message.rs** – transaction message structures such as `MsgStoreCode` and `MsgInstantiateContract`. Used by `abci_handler.rs` and CLI transaction commands.
- [x] **x/wasm/src/types/query.rs** – request and response types for contract queries. Consumed by the ABCI handler and all client interfaces.
- [x] **x/wasm/src/types/mod.rs** – exposes the query submodule for external use. Acts as the entry point for `crate::types`.
- [x] **x/wasm/src/params.rs** – module parameters controlling wasm behaviour. Accessed from `keeper.rs`.
- [x] **x/wasm/src/error.rs** – common error enum for the wasm module. Imported by the engine and keeper implementations.
- [x] **x/wasm/src/engine.rs** – defines the `WasmEngine` trait and a `CosmwasmEngine` skeleton. Called by the keeper to execute contracts.
- [x] **x/wasm/src/keeper.rs** – core keeper managing state and delegating execution to a `WasmEngine`. Relied on by genesis and the ABCI handler.
- [x] **x/wasm/src/genesis.rs** – handles loading and exporting module state at genesis using the keeper.
- [x] **x/wasm/src/abci_handler.rs** – ABCI entry points wiring transactions and queries to the keeper and message types.
- [x] **x/wasm/src/client/cli/query.rs** – CLI subcommands for querying wasm state, built on the query types.
- [x] **x/wasm/src/client/cli/tx.rs** – CLI subcommands for broadcasting wasm transactions defined in `message.rs`.
- [x] **x/wasm/src/client/cli/mod.rs** – groups the query and transaction CLI into a single module.
- [x] **x/wasm/src/client/grpc.rs** – gRPC service definitions exposing query and transaction helpers for external tooling.
- [x] **x/wasm/src/client/rest.rs** – REST handlers mirroring the gRPC interface for web applications.
- [x] **x/wasm/src/client/mod.rs** – aggregates CLI, gRPC and REST interfaces for consumers.
- [x] **x/wasm/src/lib.rs** – module root re-exporting the keeper, engine, clients and other components; depends on all previous files.

## Outstanding TODOs
- [ ] `gears/src/baseapp/query.rs:28` - TODO regarding `QueryRequest::height` design.
- [ ] `gears/src/context/query.rs:29` - placeholder chain id `"todo-900"` in `QueryContext::new`.
- [ ] `x/bank/src/abci_handler.rs:57` - `todo!()` in `BankNodeQueryRequest::height`.
- [ ] `x/auth/src/abci_handler.rs:38` - `todo!()` in `AuthNodeQueryRequest::height`.
- [ ] `x/staking/src/abci_handler.rs:79` - `todo!()` in `StakingNodeQueryRequest::height`.
- [ ] `x/upgrade/src/types/query.rs:18` - `todo!()` in `UpgradeQueryRequest::height`.
- [ ] `x/distribution/src/keeper/mod.rs:108` - `todo!()` for missing branch.
- [ ] `x/staking/src/keeper/query.rs:182` - `todo!()` placeholder for redelegation query.
- [ ] `x/staking/src/keeper/query.rs:187` - additional redelegation query `todo!()`.
- [ ] `x/gov/src/query/mod.rs:36` - `todo!()` in `GovQuery::height` implementation.
- [ ] `x/gov/src/genesis.rs:30` - `todo!()` in genesis account handling.
- [ ] `x/evidence/src/types/mod.rs:123` - `todo!()` pending YAML formatting logic.
