# ADR: Implement `x/wasm::genesis`

## Status
Proposed

## Context

Modules in Gears load their initial state from genesis and can export the state
at shutdown. `wasmd` provides genesis handling for wasm code and contract
instances, using the keeper for persistence.

## Decision

`genesis.rs` will define `init_genesis` and `export_genesis` functions that
serialise and deserialise a `GenesisState` struct. The functions will call into
the keeper to store contract code and instantiate contracts specified in the
initial state. This mirrors the behaviour of
[`wasmd/x/wasm/genesis.go`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/genesis.go).

### GenesisState Format

`GenesisState` contains:

* `codes: Vec<Code>` – each `Code` includes the wasm byte array, creator address
  and instantiate permission. The engine's `Cache` is pre-populated by calling
  `engine.store_code` during `init_genesis`.
* `contracts: Vec<Contract>` – initial contract instances to create. Fields
  include address, code ID, label, admin, IBC port ID and the initial state
  stored as raw key/value pairs. `init_genesis` will reconstruct the state by
  writing to the contract store and optionally calling `engine.instantiate` if
  `contract.init_msg` is provided.
* `sequences` – the next IDs for codes and contracts to ensure deterministic ID
  assignment matches exported state.

### Export Logic

`export_genesis` reads all code and contract info from the keeper, including
their stored bytecode, and serialises them into the `GenesisState` structure.
Contract states are exported using `iterate_contract_state` and writing each KV
pair. This matches the behaviour in `wasmd`, ensuring parity for migrations.

## Consequences

Supporting genesis import/export ensures the wasm module participates fully in
the chain lifecycle and enables migration from existing chains running `wasmd`.

## Implementation Notes

### Validation

During `init_genesis` every code and contract entry must be validated before
being committed to storage. The function should verify that:

* Checksums of provided wasm code match the metadata included in the genesis
  file. If a mismatch occurs, the process should abort with a clear error.
* Contract addresses and code IDs are unique and do not collide with existing
  entries from earlier modules in the application. This prevents accidental
  overwriting of other modules' state spaces.

### State Initialization

For each `Code` entry the engine's `store_code` method is called, which returns
the assigned code ID. If the `Code` section already contains a code ID field,
the importer must check that the returned ID matches the expected one. This step
ensures deterministic initialization across nodes, matching the behaviour in
[`wasmd`](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/genesis.go#L112).

When constructing contracts, `init_genesis` writes the provided key/value pairs
directly into the contract substore and optionally calls `engine.instantiate` if
an init message is present. The environment passed to the engine should mimic
the first block's `Env`, with `block.height = 1` and `block.time` set to the
chain start time. This replicates the semantics of `wasmd` where genesis
contracts may execute custom logic during chain start.

### Export Considerations

`export_genesis` must iterate over all stored codes and contracts in sorted
order. Sorting ensures reproducible JSON output which is critical for verifying
checksums in genesis files across different nodes. Contract states are exported
by walking each key using the prefix iterator from `kv_store`. Large states may
be split across multiple files in the `assets/` directory to keep the exported
JSON manageable, replicating the approach used in `wasmd` tests.

### Future Upgrades

As the module evolves new fields may be added to `GenesisState`. When upgrading
existing chains a migration function will read the old state, apply transforms
(such as recalculating checksums or converting addresses), and produce a new
`GenesisState` version. Careful adherence to the specification here ensures that
future migrations can be implemented without data loss.

## Testing

Genesis import/export should be covered by end-to-end tests that spin up a node,
export the state, then reload it to verify deterministic behaviour. Reference
test patterns can be found in the
[`cosmwasm-vm` repository](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm/tests)
and in `wasmd`'s own `genesis_test.go`. Each test should compare checksums of
the exported code to ensure byte-for-byte reproducibility.

