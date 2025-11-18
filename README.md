# rovo

A lightweight proc-macro crate for generating OpenAPI documentation with [aide](https://github.com/tamasfe/aide).

## Features

- 📝 **Doc-comment driven**: Write API docs as Rust doc comments
- 🎯 **Type-safe**: Full type checking for response types and examples
- 🔄 **DRY**: No need for separate `_docs` functions
- ⚡ **Lightweight**: Minimal dependencies for fast compilation
- 🚀 **Easy integration**: Works seamlessly with aide and axum

## Installation

```toml
[dependencies]
rovo = "0.1"
aide = { version = "0.13", features = ["axum"] }
axum = "0.7"
```

## Quick Start

### Basic Example

```rust
use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{extract::State, response::Json};
use rovo::rovo;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
struct User {
    id: u64,
    name: String,
}

/// Get user information.
///
/// Returns the current user's profile information.
///
/// @response 200 Json<User> User profile retrieved successfully.
#[rovo]
async fn get_user(State(_state): State<AppState>) -> impl IntoApiResponse {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
    })
}

// The macro generates `get_user_docs` automatically
fn routes(state: AppState) -> ApiRouter {
    ApiRouter::new()
        .api_route("/user", get_with(get_user, get_user_docs))
        .with_state(state)
}
```

### CRUD Example

```rust
use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use rovo::rovo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
struct Todo {
    id: Uuid,
    title: String,
    completed: bool,
}

#[derive(Deserialize, JsonSchema)]
struct TodoId {
    id: Uuid,
}

/// Get a todo by ID.
///
/// Retrieves a specific todo item by its unique identifier.
///
/// @response 200 Json<Todo> Todo found successfully.
/// @example 200 Todo { id: Uuid::nil(), title: "Example".into(), completed: false }
/// @response 404 () Todo not found.
#[rovo]
async fn get_todo(
    State(state): State<AppState>,
    Path(TodoId { id }): Path<TodoId>,
) -> impl IntoApiResponse {
    match state.todos.get(&id) {
        Some(todo) => Json(todo.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// Routes use `:id` (axum syntax) not `{id}` (OpenAPI syntax)
fn routes(state: AppState) -> ApiRouter {
    ApiRouter::new()
        .api_route("/todos/:id", get_with(get_todo, get_todo_docs))
        .with_state(state)
}
```

**Important:** Use `:id` in route paths (axum syntax), not `{id}` (OpenAPI syntax).

## Documentation Syntax

### Title and Description

The first line of the doc comment becomes the summary, subsequent lines become the description:

```rust
/// Get a todo item.        // <- Summary
///
/// Retrieves a todo item    // <- Description
/// by its unique ID.        //    (continues)
```

### Response Annotations

Use `@response` to document different response codes:

```rust
/// @response 200 Json<TodoItem> Successfully retrieved the todo item.
/// @response 404 () Todo item was not found.
/// @response 500 Json<ErrorResponse> Internal server error occurred.
```

Format: `@response <status_code> <response_type> <description>`

### Example Annotations

Use `@example` to provide examples for responses:

```rust
/// @response 200 Json<User> User information.
/// @example 200 User::default()
```

For complex examples, you can use any Rust expression:

```rust
/// @example 200 User { id: Uuid::nil(), name: "John".into(), email: "john@example.com".into() }
```

Format: `@example <status_code> <rust_expression>`

## Examples

See the [examples](./examples) directory for a complete CRUD API example with Swagger UI.

```bash
cargo run --example todo_api
# Visit http://127.0.0.1:3000 for interactive API documentation
```

The example includes:
- Full CRUD operations (Create, Read, Update, Delete)
- Interactive Swagger UI documentation
- Proper HTTP status codes
- Request/response validation

## Comparison with aide

| Feature | aide | rovo |
|---------|------|------|
| Documentation location | Separate function | With handler |
| Example generation | Manual | Can use Default trait |
| Lines of code | ~15-20 per endpoint | ~5-10 per endpoint |

## How it works

The `#[rovo]` macro:

1. Parses doc comments and extracts documentation metadata
2. Extracts title, description, responses, and examples
3. Generates a `{function_name}_docs` function automatically
4. Uses aide's `TransformOperation` API to build documentation

## Troubleshooting

### Routes return 404

**Problem:** Routes with path parameters like `/todos/:id` return 404.

**Solution:** Make sure you're using `:id` (axum syntax) in your route definitions, not `{id}` (OpenAPI syntax).

```rust
// ✅ Correct
.api_route("/todos/:id", get_with(get_todo, get_todo_docs))

// ❌ Wrong
.api_route("/todos/{id}", get_with(get_todo, get_todo_docs))
```

### Path parameters not documented in OpenAPI

**Problem:** Path parameters don't appear in the OpenAPI specification.

**Solution:** Use a struct with `JsonSchema` for path parameters instead of raw types:

```rust
// ✅ Correct - parameters will be documented
#[derive(Deserialize, JsonSchema)]
struct TodoId {
    id: Uuid,
}

async fn get_todo(Path(TodoId { id }): Path<TodoId>) -> impl IntoApiResponse {
    // ...
}

// ❌ Wrong - parameters won't be documented properly
async fn get_todo(Path(id): Path<Uuid>) -> impl IntoApiResponse {
    // ...
}
```

### Handler doesn't implement `OperationHandler`

**Problem:** Compiler error about `OperationHandler` trait not being implemented.

**Solution:** Make sure your handler returns `impl IntoApiResponse` (from `aide::axum`), not `impl IntoResponse` (from `axum`):

```rust
use aide::axum::IntoApiResponse;

// ✅ Correct
async fn handler() -> impl IntoApiResponse {
    Json(data)
}

// ❌ Wrong
async fn handler() -> impl IntoResponse {
    Json(data)
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT OR Apache-2.0
