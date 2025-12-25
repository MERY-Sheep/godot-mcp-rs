//! Input Action & Project Settings Tests (Phase 2.2)
//!
//! TDD tests for addInputAction and setProjectSetting mutations.

use godot_mcp_rs::graphql::{build_schema_with_context, GqlContext};
use std::path::PathBuf;

fn test_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_project")
}

fn build_test_schema() -> godot_mcp_rs::graphql::GqlSchema {
    let ctx = GqlContext::new(test_project_path());
    build_schema_with_context(ctx)
}

/// Backup of original project.godot for cleanup
const ORIGINAL_PROJECT_GODOT: &str = r#"; Engine configuration file.
; It's best edited using the editor UI and not directly,
; since the parameters that go here are not all obvious.
;
; Format:
;   [section] ; section goes between []
;   param=value ; assign values to parameters

config_version=5

[application]

config/name="TestProject"
run/main_scene="res://scenes/main.tscn"
config/features=PackedStringArray("4.5", "Forward Plus")

[editor_plugins]

enabled=PackedStringArray("res://addons/godot_mcp/plugin.cfg")
"#;

/// Reset project.godot to original state after tests
fn cleanup_test_project() {
    let project_godot = test_project_path().join("project.godot");
    std::fs::write(&project_godot, ORIGINAL_PROJECT_GODOT).ok();
}

// ======================
// addInputAction tests
// ======================

/// Test: addInputAction with valid key event
#[tokio::test]
async fn test_add_input_action_valid_key() {
    let schema = build_test_schema();
    // Use a unique action name that doesn't exist in test_project
    let query = r#"
        mutation {
            addInputAction(input: {
                actionName: "test_unique_action_12345",
                events: [{ type: KEY, key: "Space" }]
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let op_result = &data["addInputAction"];
    assert_eq!(
        op_result["success"], true,
        "Expected success but got: {:?}",
        op_result
    );

    // Cleanup: Remove the added action from project.godot
    cleanup_test_project();
}

/// Test: addInputAction with multiple events
#[tokio::test]
async fn test_add_input_action_multiple_events() {
    let schema = build_test_schema();
    // Use a unique action name
    let query = r#"
        mutation {
            addInputAction(input: {
                actionName: "test_move_left_12345",
                events: [
                    { type: KEY, key: "A" },
                    { type: KEY, key: "Left" },
                    { type: JOY_BUTTON, button: 14, device: 0 }
                ]
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let op_result = &data["addInputAction"];
    assert_eq!(
        op_result["success"], true,
        "Expected success but got: {:?}",
        op_result
    );

    // Cleanup
    cleanup_test_project();
}

/// Test: addInputAction with empty action name should fail
#[tokio::test]
async fn test_add_input_action_empty_name() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            addInputAction(input: {
                actionName: "",
                events: [{ type: KEY, key: "Space" }]
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let op_result = &data["addInputAction"];
    assert_eq!(op_result["success"], false);
}

// ======================
// setProjectSetting tests
// ======================

/// Test: setProjectSetting with valid path
#[tokio::test]
async fn test_set_project_setting_valid() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            setProjectSetting(input: {
                path: "application/config/name",
                value: "\"MyGame\""
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let op_result = &data["setProjectSetting"];
    assert_eq!(op_result["success"], true);
}

/// Test: setProjectSetting with integer value
#[tokio::test]
async fn test_set_project_setting_integer() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            setProjectSetting(input: {
                path: "display/window/size/viewport_width",
                value: "1920",
                type: "int"
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let op_result = &data["setProjectSetting"];
    assert_eq!(op_result["success"], true);
}

/// Test: setProjectSetting with empty path should fail
#[tokio::test]
async fn test_set_project_setting_empty_path() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            setProjectSetting(input: {
                path: "",
                value: "test"
            }) {
                success
                message
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let op_result = &data["setProjectSetting"];
    assert_eq!(op_result["success"], false);
}
