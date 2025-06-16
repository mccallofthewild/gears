# ADR: Implement CLI queries for `x/wasm`

## Status
Proposed

## Context

Command line users need to query contract state and metadata. `wasmd` exposes
subcommands like `wasmd query wasm contract-state smart` and `wasmd query wasm
code`. These rely on the query types defined in `types/query.rs`.

## Decision

`client/cli/query.rs` will use `clap` to define subcommands mirroring the wasmd
CLI. Each command will construct the appropriate query type and use
`gears::client` utilities to send it to a node. Output will be rendered as JSON
or text depending on flags.

### Command Set

1. `contract-state smart <addr> <query>` – Sends a smart query to the given
   contract. The `<query>` argument accepts a JSON string or path to a file. The
   command converts it to `Binary` and displays the contract's JSON response.
2. `contract-state raw <addr> <key>` – Fetches a raw key from contract storage
   and outputs the hex-encoded value.
3. `code <id>` – Downloads the original wasm code bytes and writes them to a
   file if `--output` is provided. Otherwise prints metadata as JSON.
4. `contract-info <addr>` – Displays code ID, admin and label for the specified
   contract.
5. `list-contract-by-code <id>` – Uses pagination flags `--limit` and `--page`
   to list all contracts created from a code ID.

Each command will leverage `gears::client::QueryClient` for network transport
and respect global node connection flags. Examples will be documented in the
module-level comments to match those found in `wasmd/docs`.

## Consequences

Providing a familiar CLI experience helps users migrate from wasmd-based chains
and aids testing during development.

## Implementation Guidelines

### Parsing Arguments

For JSON arguments the command should accept either an inline string or a path
prefixed with `@`. This mirrors `wasmd`'s behaviour and allows large payloads to
be stored in separate files. The CLI will read the file contents and convert them
to `Binary`. Address arguments use the `address::Address` type for automatic
Bech32 validation.

### Output Formatting

Results can be printed as raw JSON or as a pretty-printed table using the
standard output helpers from the `gears` crate. Users can specify `--output` to
save binary contract code directly to a file. Examples in the documentation will
show both interactive and script-friendly usage patterns.

### Error Handling

When queries fail due to `WasmError` responses from the keeper, the CLI should
display a human-readable message and exit with a non-zero status code. This
mirrors the standard behaviour of `wasmd` CLI commands and enables shell scripts
to detect failures easily.

### Extensibility

Future query types such as historical code history or pinned code lists can be
added by extending the command set. Each new command should follow the same
pattern of argument parsing and output rendering so that users experience a
consistent interface.

### Testing

Command parsing tests ensure that flags and arguments are handled correctly.
Integration tests run the CLI against a local node started by `gaia-rs` and
compare the results with those from the `wasmd` CLI to confirm identical
behaviour.

\nBy adhering to these conventions the query CLI will be a reliable tool for debugging and day-to-day chain operations.
