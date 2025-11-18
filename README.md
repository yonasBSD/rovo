# rovo

A lightweight proc-macro crate for generating OpenAPI documentation with [aide](https://github.com/tamasfe/aide).

## Features

- 📝 **Doc-comment driven**: Write API docs as Rust doc comments
- 🎯 **Type-safe**: Full type checking for response types and examples
- 🔄 **DRY**: No need for separate `_docs` functions
- ⚡ **Lightweight**: Minimal dependencies for fast compilation

## Installation

```toml
[dependencies]
rovo = "0.1"
aide = { version = "0.13", features = ["axum"] }
```

## Usage

### Before (with aide)

```rust
use aide::axum::routing::get_with;

async fn get_todo(
    State(app): State<AppState>,
    Path(todo): Path<SelectTodo>,
) -> impl IntoApiResponse {
    // handler code
}

fn get_todo_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Get a single Todo item.")
      .description("Retrieve a Todo item by its ID.")
        .response_with::<200, Json<TodoItem>,_>(|res| {
            res.example(TodoItem {
                complete: false,
                description: "fix bugs".into(),
                id: Uuid::nil(),
            })
        })
        .response_with::<404, (),_>(|res| res.description("todo was not found"))
}

ApiRouter::new()
    .api_route("/{id}", get_with(get_todo, get_todo_docs))
```

### After (with rovo)

```rust
use rovo::rovo;
use aide::axum::routing::get_with;

/// Get a single Todo item.
///
/// Retrieve a Todo item by its ID.
///
/// @response 200 Json<TodoItem> Successfully retrieved the todo item.
/// @example 200 TodoItem::default()
/// @response 404 () Todo item was not found.
#[rovo]
async fn get_todo(
    State(app): State<AppState>,
    Path(todo): Path<SelectTodo>,
) -> impl IntoApiResponse {
    // handler code
}

ApiRouter::new()
    .api_route("/{id}", get_with(get_todo, get_todo_docs))
```

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

## License

MIT OR Apache-2.0
