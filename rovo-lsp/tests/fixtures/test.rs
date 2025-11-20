use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct Error {
    message: String,
}

/// @tag users
/// @response 200 Json<User> Successfully retrieved user
/// @response 404 Json<Error> User not found
/// @example 200 {"id": 1, "name": "John Doe", "email": "john@example.com"}
/// @id get_user
#[rovo]
async fn get_user(id: i32) -> Result<Json<User>, StatusCode> {
    Ok(Json(User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    }))
}

/// @tag users
/// @response 201 Json<User> User created successfully
/// @response 400 Json<Error> Invalid request
/// @example 201 {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
#[rovo]
async fn create_user(user: Json<User>) -> Result<Json<User>, StatusCode> {
    Ok(user)
}

/// This should trigger an error - invalid status code
/// @response 999 Json<User> Invalid status
#[rovo]
async fn invalid_handler() -> Json<User> {
    Json(User {
        id: 1,
        name: "Test".to_string(),
        email: "test@example.com".to_string(),
    })
}

/// @security bearer
/// @tag admin
/// @response 200 Json<Vec<User>> List of all users
/// @response 401 Json<Error> Unauthorized
#[rovo]
async fn list_users() -> Json<Vec<User>> {
    Json(vec![])
}

/// @hidden
#[rovo]
async fn internal_endpoint() -> String {
    "Internal use only".to_string()
}
