# ADR: Implement CLI transactions for `x/wasm`

## Status
Proposed

## Context

Users need command line tools to upload and interact with contracts. `wasmd`
provides subcommands such as `wasmd tx wasm store` and `wasmd tx wasm execute`
which construct the message types defined in `message.rs`.

## Decision

`client/cli/tx.rs` will use `clap` to define these subcommands. Each command will
parse arguments (e.g. wasm file paths, instantiate JSON, funds) and create the
corresponding `message::Message` variant. It will then submit the transaction via
`gears::client` utilities.

### Command Set

1. `store <wasm>` – Reads a WASM file from disk, encodes it as base64 and
   constructs a `Message::StoreCode`. Supports flags for setting instantiate
   permissions.
2. `instantiate <code-id> <init-json>` – Builds an `InstantiateContract` message
   reading the init message from a JSON string or file. Accepts `--label` and
   `--admin` options.
3. `execute <addr> <exec-json>` – Executes a contract method. Allows specifying
   funds via `--amount` repeating flag using the coin parsing utilities in the
   bank module.
4. `migrate <addr> <new-code-id> <migrate-json>` – Migrates a contract to a new
   code.
5. `update-admin <addr> <new-admin>` and `clear-admin <addr>` – Manage contract
   administration.

All commands integrate with the global transaction flags (fees, gas, memo) and
output the resulting transaction hash for user confirmation. Examples will be
taken from `wasmd` documentation but adapted to the Gears CLI style.

## Consequences

Having a full-featured CLI greatly improves developer ergonomics and allows
manual testing without writing gRPC or REST clients.

## Implementation Guidelines

### Argument Parsing

Each subcommand uses `clap` argument macros to validate required parameters. For
coin amounts the CLI reuses the parsing utilities from the bank module so that
users can specify multiple `--amount` flags in the standard `<denom><amount>`
format. WASM files are read and base64-encoded to match the expected message
format. JSON payloads accept either inline strings or `@file` syntax similar to
the query commands.

### Transaction Broadcasting

`gears::client` provides helpers to sign and broadcast transactions. The CLI
should support both synchronous and asynchronous broadcast modes, controlled via
a `--broadcast-mode` flag. This ensures parity with the `wasmd` CLI and allows
developers to inspect the transaction before it is included in a block.

### Gas and Fees

Global flags for gas limit, gas price and fee amount should be integrated with
each transaction command. The CLI will compute the final fee using the standard
`tx::fees` utilities and attach it to the transaction. Examples in the
documentation will show typical values for testing networks.

### Output

On success the command prints the transaction hash and any events emitted by the
keeper. When run with the `--json` flag, the entire `TxResponse` structure is
printed as JSON for machine-readable processing. This is consistent with the
behaviour of `wasmd tx` commands.

### Error Handling

If the transaction fails validation or broadcasting, the CLI will exit with an
appropriate error code and message. Tests should cover scenarios such as invalid
addresses, missing wasm files and insufficient fees to ensure user-friendly
feedback.

### Extensibility

Future enhancements, such as multisig support or offline signing, can be added
by extending the underlying client utilities. The subcommand definitions should
remain stable so that shell scripts written today continue to work with newer
versions of the CLI.

