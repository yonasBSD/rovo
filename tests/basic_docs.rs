use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use axum::extract::{Path, State};
use axum::response::Json;
use rovo::rovo;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    // Mock state for testing
}

#[derive(Clone, Debug, serde::Serialize, JsonSchema, PartialEq)]
struct TodoItem {
    id: Uuid,
    description: String,
    complete: bool,
}

impl Default for TodoItem {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            description: "fix bugs".into(),
            complete: false,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
struct SelectTodo {
    /// The ID of the Todo.
    #[allow(dead_code)]
    id: Uuid,
}

/// Get a single Todo item.
///
/// Retrieve a Todo item by its ID.
///
/// @response 200 Json<TodoItem> A single Todo item.
/// @response 404 () Todo was not found.
#[rovo]
async fn get_todo(
    State(_app): State<AppState>,
    Path(_todo): Path<SelectTodo>,
) -> impl IntoApiResponse {
    Json(TodoItem::default())
}

#[test]
fn test_macro_compiles() {
    // This test just ensures the macro compiles correctly
    let _state = AppState {};

    // The macro should have generated a module with handler and docs
    // that we can use with aide's get_with
    let _router: ApiRouter<AppState> = ApiRouter::new()
        .api_route("/todo/{id}", get_with(get_todo::handler, get_todo::docs));
}
