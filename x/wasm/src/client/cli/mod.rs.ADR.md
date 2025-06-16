# ADR: Implement CLI module wrapper for `x/wasm`

## Status
Proposed

## Context

Like other modules, the wasm CLI groups query and transaction subcommands under a
single top-level command. This mirrors the structure in `wasmd` where
`Cmd()` returns a root command combining `tx` and `query`.

## Decision

`client/cli/mod.rs` will expose a `command()` function returning a `clap::Command`
that registers the query and transaction subcommand trees defined in
`query.rs` and `tx.rs`. This function is invoked from the application CLI to add
the wasm module commands.

The layout mirrors the [`wasmd/cmd/wasmd`](https://github.com/CosmWasm/wasmd/blob/main/cmd/wasmd/main.go)
setup where the root `wasm` command includes `tx` and `query` subcommands.
`command()` will accept a mutable reference to the global `Command` so it can be
composed with other module CLI integrations. Each subcommand tree will also set
`about` strings explaining usage and link to examples in the docs.

## Consequences

Bundling the subcommands simplifies integration with the node CLI and provides a
familiar interface for users.

## Implementation Details

### Command Layout

The top-level `wasm` command will include two subcommands: `tx` and `query`.
Each subcommand tree is defined in separate files to keep the code manageable.
`command()` constructs these subcommands using `clap::Command::new` and attaches
them to the parent command. Flags shared across all wasm commands, such as the
RPC address or output format, are configured at this level.

### Example Usage

```text
gearsd wasm tx store <file.wasm>
gearsd wasm query contract-state smart <addr> '{"balance":{}}'
```

These examples mimic the `wasmd` CLI so existing users can easily switch. The
command function will be documented with additional examples for each variant.

### Extensibility

Future features, such as IBC packet queries or migration proposals, can be added
by extending the `tx` or `query` submodules. The root function simply merges the
updated subcommand trees, meaning applications automatically gain new commands
when they upgrade the `x/wasm` crate.

### Testing

Unit tests build the command tree and assert that parsing example arguments
produces the expected message types. Integration tests run the CLI against a
local node to ensure transactions are correctly serialized and broadcast.

## Rationale

Providing a familiar CLI is crucial for developer adoption. By mirroring the
layout and behaviour of `wasmd`'s command set we ensure that documentation and
tutorials translate directly to Gears-based nodes. The separation into query and
transaction trees keeps the code organized while allowing advanced users to
compose commands with global flags as needed.

### Cross-Module Integration

The CLI module integrates with the application by appending its commands to the
global `Command` instance used by `gearsd`. This is typically done in the
application's `main.rs` where each module registers its CLI via a builder
pattern. The wasm module should follow this pattern so that it can be easily
included in tutorial chains alongside bank and staking commands.

### Autocompletion and Help Output

`clap` supports generating shell completion scripts and rich help messages. The
`command()` function should enable these features so users can discover the
available wasm options quickly. Each subcommand's `about` string must be concise
yet descriptive, referencing the relevant ADR or documentation section for
further reading.

### Future Improvements

As CosmWasm adds new transaction types—such as contract governance proposals—the
CLI module can extend the `tx` tree accordingly. Developers should ensure that
new commands follow the same argument conventions to minimize surprises for end
users.


