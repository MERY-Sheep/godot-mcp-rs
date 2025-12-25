//! Introspect Tests (Phase 1: Dynamic Type Discovery)
//!
//! Tests for dynamic type introspection via Godot ClassDB.

use godot_mcp_rs::graphql::{build_schema_with_context, GqlContext};
use std::path::PathBuf;

fn test_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn build_test_schema() -> godot_mcp_rs::graphql::GqlSchema {
    let ctx = GqlContext::new(test_project_path());
    build_schema_with_context(ctx)
}

// ======================
// Static Fallback Tests (no server required)
// ======================

/// Test: nodeTypeInfo query returns static data when no Godot editor is connected
#[tokio::test]
async fn test_node_type_info_static_fallback() {
    let schema = build_test_schema();

    let query = r#"
        query {
            nodeTypeInfo(typeName: "CharacterBody3D") {
                typeName
                properties { name type }
                signals { name }
            }
        }
    "#;

    let result = schema.execute(query).await;
    let data = result.data.into_json().unwrap();

    // Static fallback should return data for known types
    let type_info = &data["nodeTypeInfo"];
    assert_eq!(type_info["typeName"], "CharacterBody3D");

    // Should have some properties (from static database)
    let properties = type_info["properties"].as_array().unwrap();
    assert!(!properties.is_empty());

    // Should have velocity property
    let property_names: Vec<&str> = properties
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(property_names.contains(&"velocity"));
}

/// Test: nodeTypeInfo returns None for unknown types in static database
#[tokio::test]
async fn test_node_type_info_unknown_type() {
    let schema = build_test_schema();

    let query = r#"
        query {
            nodeTypeInfo(typeName: "CompletelyUnknownType") {
                typeName
            }
        }
    "#;

    let result = schema.execute(query).await;
    let data = result.data.into_json().unwrap();

    // Should return null for unknown types
    assert!(data["nodeTypeInfo"].is_null());
}

/// Test: nodeTypeInfo returns data for common node types in static database
#[tokio::test]
async fn test_node_type_info_common_types() {
    let schema = build_test_schema();

    let common_types = [
        "Node", "Node3D", "Timer", "Button", "Label", "Camera3D", "Area3D",
    ];

    for type_name in common_types {
        let query = format!(
            r#"
            query {{
                nodeTypeInfo(typeName: "{}") {{
                    typeName
                    properties {{ name }}
                    signals {{ name }}
                }}
            }}
        "#,
            type_name
        );

        let result = schema.execute(&query).await;
        let data = result.data.into_json().unwrap();

        let type_info = &data["nodeTypeInfo"];
        assert!(
            !type_info.is_null(),
            "Expected nodeTypeInfo for {} but got null",
            type_name
        );
        assert_eq!(type_info["typeName"], type_name);
    }
}

// ======================
// Live Introspection Tests (require Godot editor)
// ======================

/// Test: nodeTypeInfoLive command serialization
#[test]
fn test_get_type_info_command_serialization() {
    use godot_mcp_rs::graphql::live_resolver::GodotLiveCommand;

    let command = GodotLiveCommand::GetTypeInfo {
        type_name: "CharacterBody3D".to_string(),
    };

    let json = serde_json::to_string(&command).unwrap();
    assert!(json.contains("get_type_info"));
    assert!(json.contains("CharacterBody3D"));
}

/// Test: listAllTypes command serialization
#[test]
fn test_list_all_types_command_serialization() {
    use godot_mcp_rs::graphql::live_resolver::GodotLiveCommand;

    let command = GodotLiveCommand::ListAllTypes {
        parent_class: "Node3D".to_string(),
    };

    let json = serde_json::to_string(&command).unwrap();
    assert!(json.contains("list_all_types"));
    assert!(json.contains("Node3D"));
}
