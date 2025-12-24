//! Live Operation Tests (Phase 4)
//!
//! Tests for Live resolver integration with HTTP stub.

use godot_mcp_rs::graphql::{build_schema_with_context, GqlContext};
use std::path::PathBuf;

fn test_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_project")
}

fn build_test_schema_with_port(port: u16) -> godot_mcp_rs::graphql::GqlSchema {
    let ctx = GqlContext::new(test_project_path()).with_port(port);
    build_schema_with_context(ctx)
}

// ======================
// Query Tests (no server required - return None on connection error)
// ======================

/// Test: currentScene returns None when no Godot editor is connected
#[tokio::test]
async fn test_current_scene_no_connection() {
    // Use a port that's unlikely to have a server
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        query {
            currentScene {
                path
                root {
                    name
                    type
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    // Should return null when no server is available
    assert!(data["currentScene"].is_null());
}

/// Test: node query returns None when no Godot editor is connected
#[tokio::test]
async fn test_node_query_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        query {
            node(path: "/root/Player") {
                name
                type
                path
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    assert!(data["node"].is_null());
}

// ======================
// Mutation Tests (return error message on connection failure)
// ======================

/// Test: addNode returns error when no server
#[tokio::test]
async fn test_add_node_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        mutation {
            addNode(input: {
                parent: "/root",
                name: "TestNode",
                type: "Node2D"
            }) {
                success
                message
                node {
                    name
                    type
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let add_node = &data["addNode"];

    // Should return success=false with an error message
    assert_eq!(add_node["success"], false);
    assert!(add_node["message"].as_str().is_some());
    assert!(add_node["node"].is_null());
}

/// Test: removeNode returns error when no server
#[tokio::test]
async fn test_remove_node_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        mutation {
            removeNode(path: "/root/Player") {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let remove_node = &data["removeNode"];

    assert_eq!(remove_node["success"], false);
    assert!(remove_node["message"].as_str().is_some());
}

/// Test: setProperty returns error when no server
#[tokio::test]
async fn test_set_property_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        mutation {
            setProperty(input: {
                nodePath: "/root/Player",
                property: "position",
                value: "Vector3(1, 2, 3)"
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let set_property = &data["setProperty"];

    assert_eq!(set_property["success"], false);
    assert!(set_property["message"].as_str().is_some());
}

/// Test: setProperties returns error when no server
#[tokio::test]
async fn test_set_properties_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        mutation {
            setProperties(
                nodePath: "/root/Player",
                properties: [
                    { name: "position", value: "Vector3(1, 2, 3)" },
                    { name: "rotation", value: "Vector3(0, 0, 0)" }
                ]
            ) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let set_properties = &data["setProperties"];

    assert_eq!(set_properties["success"], false);
    assert!(set_properties["message"].as_str().is_some());
}

/// Test: connectSignal returns error when no server
#[tokio::test]
async fn test_connect_signal_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        mutation {
            connectSignal(input: {
                fromNode: "/root/Button",
                signal: "pressed",
                toNode: "/root/Handler",
                method: "_on_button_pressed"
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let connect_signal = &data["connectSignal"];

    assert_eq!(connect_signal["success"], false);
    assert!(connect_signal["message"].as_str().is_some());
}

/// Test: saveScene returns error when no server
#[tokio::test]
async fn test_save_scene_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        mutation {
            saveScene {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let save_scene = &data["saveScene"];

    assert_eq!(save_scene["success"], false);
    assert!(save_scene["message"].as_str().is_some());
}

/// Test: openScene returns error when no server
#[tokio::test]
async fn test_open_scene_no_connection() {
    let schema = build_test_schema_with_port(19999);
    let query = r#"
        mutation {
            openScene(path: "res://scenes/player.tscn") {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let open_scene = &data["openScene"];

    assert_eq!(open_scene["success"], false);
    assert!(open_scene["message"].as_str().is_some());
}

// ======================
// Live Resolver Unit Tests
// ======================

#[cfg(test)]
mod live_resolver_tests {
    use godot_mcp_rs::graphql::live_resolver::*;

    #[test]
    fn test_godot_command_serialization() {
        let cmd = GodotLiveCommand::AddNode {
            parent: "/root".to_string(),
            name: "NewNode".to_string(),
            node_type: "Node2D".to_string(),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("add_node"));
        assert!(json.contains("NewNode"));
    }

    #[test]
    fn test_ping_command_serialization() {
        let cmd = GodotLiveCommand::Ping;
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("ping"));
    }

    #[test]
    fn test_set_property_command_serialization() {
        let cmd = GodotLiveCommand::SetProperty {
            node_path: "/root/Player".to_string(),
            property: "position".to_string(),
            value: serde_json::json!({"x": 1.0, "y": 2.0, "z": 3.0}),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("set_property"));
        assert!(json.contains("position"));
    }

    #[test]
    fn test_live_error_display() {
        let err = LiveError::Connection("test error".to_string());
        assert!(err.to_string().contains("Connection error"));

        let err = LiveError::Timeout;
        assert!(err.to_string().contains("timed out"));

        let err = LiveError::HttpError {
            status: 500,
            message: "Internal Server Error".to_string(),
        };
        assert!(err.to_string().contains("500"));
    }
}
