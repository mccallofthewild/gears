# CosmWasm Module Implementation Checklist

Below is the recommended order for implementing the files within `x/wasm`. Each item includes a short description of its role and key files it interacts with.

- [ ] **x/wasm/Cargo.toml** – crate manifest defining dependencies for the module. It underpins compilation of all subsequent files.
- [ ] **x/wasm/src/message.rs** – transaction message structures such as `MsgStoreCode` and `MsgInstantiateContract`. Used by `abci_handler.rs` and CLI transaction commands.
- [ ] **x/wasm/src/types/query.rs** – request and response types for contract queries. Consumed by the ABCI handler and all client interfaces.
- [ ] **x/wasm/src/types/mod.rs** – exposes the query submodule for external use. Acts as the entry point for `crate::types`.
- [ ] **x/wasm/src/params.rs** – module parameters controlling wasm behaviour. Accessed from `keeper.rs`.
- [ ] **x/wasm/src/error.rs** – common error enum for the wasm module. Imported by the engine and keeper implementations.
- [ ] **x/wasm/src/engine.rs** – defines the `WasmEngine` trait and a `CosmwasmEngine` skeleton. Called by the keeper to execute contracts.
- [ ] **x/wasm/src/keeper.rs** – core keeper managing state and delegating execution to a `WasmEngine`. Relied on by genesis and the ABCI handler.
- [ ] **x/wasm/src/genesis.rs** – handles loading and exporting module state at genesis using the keeper.
- [ ] **x/wasm/src/abci_handler.rs** – ABCI entry points wiring transactions and queries to the keeper and message types.
- [ ] **x/wasm/src/client/cli/query.rs** – CLI subcommands for querying wasm state, built on the query types.
- [ ] **x/wasm/src/client/cli/tx.rs** – CLI subcommands for broadcasting wasm transactions defined in `message.rs`.
- [ ] **x/wasm/src/client/cli/mod.rs** – groups the query and transaction CLI into a single module.
- [ ] **x/wasm/src/client/grpc.rs** – gRPC service definitions exposing query and transaction helpers for external tooling.
- [ ] **x/wasm/src/client/rest.rs** – REST handlers mirroring the gRPC interface for web applications.
- [ ] **x/wasm/src/client/mod.rs** – aggregates CLI, gRPC and REST interfaces for consumers.
- [ ] **x/wasm/src/lib.rs** – module root re-exporting the keeper, engine, clients and other components; depends on all previous files.
