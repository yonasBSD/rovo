#![allow(dead_code)]

use rovo::schemars::JsonSchema;
use rovo::{schema, schemars};

/// Verifies that `#[schema]` correctly injects `#[schemars(crate = "::rovo::schemars")]`
/// and works alongside `#[derive(JsonSchema)]`.
#[schema]
#[derive(Debug, JsonSchema)]
struct BasicStruct {
    name: String,
    count: u32,
}

/// Verifies `#[schema]` works with enums.
#[schema]
#[derive(Debug, JsonSchema)]
enum MyEnum {
    A,
    B(String),
}

/// Verifies `#[schema]` is a no-op when `#[schemars(crate = "...")]` is already present.
#[schema]
#[derive(Debug, JsonSchema)]
#[schemars(crate = "::rovo::schemars")]
struct AlreadyAnnotated {
    value: i64,
}

#[test]
fn schema_macro_produces_valid_json_schema() {
    // If this compiles and runs, the derive succeeded through rovo's re-export path
    let schema = schemars::SchemaGenerator::default().into_root_schema_for::<BasicStruct>();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("BasicStruct"));
}

#[test]
fn schema_macro_works_for_enums() {
    let schema = schemars::SchemaGenerator::default().into_root_schema_for::<MyEnum>();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("MyEnum"));
}

#[test]
fn schema_macro_noop_when_crate_already_set() {
    let schema = schemars::SchemaGenerator::default().into_root_schema_for::<AlreadyAnnotated>();
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("AlreadyAnnotated"));
}
