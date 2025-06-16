# ADR: Implement REST endpoints for `x/wasm`

## Status
Proposed

## Context

In addition to gRPC, many web applications interact with nodes via REST. `wasmd`
exposes a REST API that mirrors the gRPC services. Gears' HTTP stack is built on
`axum` and follows a similar pattern for other modules.

## Decision

`client/rest.rs` will define axum handlers that map HTTP routes to the query and
transaction helpers provided by the gRPC service. Routes will include
`/wasm/contracts/{addr}` and `/wasm/codes/{id}` among others. JSON payloads will
be parsed using the query and message types.

### Endpoint Structure

* `GET /wasm/contracts/{addr}/smart` – Accepts a `query` parameter containing
  base64 encoded JSON and returns the contract response.
* `GET /wasm/contracts/{addr}/raw/{key}` – Reads a raw key from storage.
* `GET /wasm/codes/{id}` – Returns code bytes and metadata.
* `POST /wasm/instantiate` – Accepts JSON body matching the
  `InstantiateContract` message and forwards to the gRPC service.
* `POST /wasm/execute` – Similar to instantiate but for execution.

The handlers will reuse the `WasmService` gRPC methods internally by calling
them directly through a shared `Keeper` instance, avoiding code duplication.
Responses follow the same JSON structure as the CLI to maintain consistency with
the examples in the [wasmd REST server](https://github.com/CosmWasm/wasmd/blob/main/x/wasm/client/rest/rest.go).

## Consequences

REST support enables browser-based tools to interact with the chain without
requiring gRPC clients.

## Implementation Details

### Routing and Middleware

The REST module uses `axum` to define an HTTP router. Each handler function
takes an `Extension<Keeper>` to access the wasm keeper and a typed request body
where necessary. Middleware such as logging, CORS and rate limiting should be
applied at the router level. For example, `tower_http::cors::CorsLayer` allows
browsers to query contracts from web applications. This mirrors the CORS setup in
`wasmd`'s REST server.

### Request and Response Types

Handlers accept JSON payloads conforming to the structs defined in
`message.rs` and `types::query`. Axum's `Json<T>` extractor automatically
deserializes the body and returns HTTP 400 on failure. Responses are serialized
using the same types, ensuring consistency across CLI, gRPC and REST. Error
cases are mapped to appropriate HTTP status codes—`404` for missing contracts,
`403` for unauthorized actions and `500` for internal errors.

### Example

```rust
async fn execute(
    Extension(keeper): Extension<Keeper>,
    Json(msg): Json<ExecuteContract>,
) -> Result<Json<Response>, StatusCode> {
    let result = keeper.execute(msg.into()).map_err(to_status)?;
    Ok(Json(result.into()))
}
```

### Testing

Unit tests spin up the router using `axum::Server` on a random port and issue
HTTP requests via `reqwest`. These tests ensure that the handlers correctly
translate between HTTP requests and keeper operations. Integration tests may
compare responses with those produced by `wasmd` to guarantee compatibility.

### Deployment

Applications may choose to serve the REST API on a separate port from gRPC. The
server builder should allow optional TLS configuration and request timeouts.
Documentation will include examples of running the REST server behind a reverse
proxy such as Nginx for production deployments.

## Rationale

Providing REST endpoints is essential for browser-based wallets and explorers
that cannot easily use gRPC. By reusing the keeper and query/message types we
avoid duplicate logic and ensure feature parity with the gRPC service. The API
structure closely mirrors that of `wasmd`, simplifying the transition for tools
that already interact with CosmWasm chains.

