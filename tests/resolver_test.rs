//! Resolver Tests (Phase 2)
//!
//! Tests for GraphQL resolvers using the test_project fixture.

use godot_mcp_rs::graphql::{build_schema_with_context, GqlContext};
use std::path::PathBuf;

fn test_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_project")
}

fn build_test_schema() -> godot_mcp_rs::graphql::GqlSchema {
    let ctx = GqlContext::new(test_project_path());
    build_schema_with_context(ctx)
}

/// Test: Project query returns correct data
#[tokio::test]
async fn test_project_query() {
    let schema = build_test_schema();
    let query = r#"
        query {
            project {
                name
                path
                stats {
                    sceneCount
                    scriptCount
                    resourceCount
                }
                scenes {
                    path
                }
                scripts {
                    path
                }
                validation {
                    isValid
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let project = &data["project"];

    // Project name from project.godot
    assert_eq!(project["name"], "TestProject");

    // Stats should reflect actual files in test_project
    let stats = &project["stats"];
    assert!(
        stats["sceneCount"].as_i64().unwrap() >= 1,
        "Should have at least 1 scene"
    );
    assert!(
        stats["scriptCount"].as_i64().unwrap() >= 1,
        "Should have at least 1 script"
    );

    // Scenes should include player.tscn
    let scenes = project["scenes"].as_array().unwrap();
    let scene_paths: Vec<&str> = scenes
        .iter()
        .map(|s| s["path"].as_str().unwrap())
        .collect();
    assert!(
        scene_paths.iter().any(|p| p.contains("player.tscn")),
        "Should include player.tscn: {:?}",
        scene_paths
    );

    // Scripts should include player.gd
    let scripts = project["scripts"].as_array().unwrap();
    let script_paths: Vec<&str> = scripts
        .iter()
        .map(|s| s["path"].as_str().unwrap())
        .collect();
    assert!(
        script_paths.iter().any(|p| p.contains("player.gd")),
        "Should include player.gd: {:?}",
        script_paths
    );

    // Validation
    assert_eq!(project["validation"]["isValid"], true);
}

/// Test: Project query snapshot
#[tokio::test]
async fn test_project_query_snapshot() {
    let schema = build_test_schema();
    let query = r#"
        query {
            project {
                name
                stats {
                    sceneCount
                    scriptCount
                    resourceCount
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    insta::assert_json_snapshot!("project_stats", data);
}

/// Test: Scene query returns correct structure
#[tokio::test]
async fn test_scene_query() {
    let schema = build_test_schema();
    let query = r#"
        query {
            scene(path: "res://scenes/player.tscn") {
                path
                root {
                    name
                    type
                }
                allNodes {
                    name
                    type
                    path
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let scene = &data["scene"];

    // Scene should exist
    assert!(!scene.is_null(), "Scene should be found");

    // Path should match
    assert_eq!(scene["path"], "res://scenes/player.tscn");

    // Root node
    let root = &scene["root"];
    assert_eq!(root["name"], "Player");
    assert_eq!(root["type"], "CharacterBody3D");

    // All nodes should be returned
    let all_nodes = scene["allNodes"].as_array().unwrap();
    assert!(all_nodes.len() >= 1, "Should have at least 1 node");
}

/// Test: Scene query snapshot
#[tokio::test]
async fn test_scene_query_snapshot() {
    let schema = build_test_schema();
    let query = r#"
        query {
            scene(path: "res://scenes/player.tscn") {
                path
                root {
                    name
                    type
                }
                allNodes {
                    name
                    type
                    path
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    insta::assert_json_snapshot!("scene_player", data);
}

/// Test: Script query returns correct structure
#[tokio::test]
async fn test_script_query() {
    let schema = build_test_schema();
    let query = r#"
        query {
            script(path: "res://scripts/player.gd") {
                path
                extends
                className
                functions {
                    name
                    arguments
                }
                variables {
                    name
                    type
                    defaultValue
                }
                exports {
                    name
                    type
                    defaultValue
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let script = &data["script"];

    // Script should exist
    assert!(!script.is_null(), "Script should be found");

    // Path should match
    assert_eq!(script["path"], "res://scripts/player.gd");

    // Extends CharacterBody3D
    assert_eq!(script["extends"], "CharacterBody3D");

    // Functions should include _physics_process
    let functions = script["functions"].as_array().unwrap();
    let func_names: Vec<&str> = functions
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    assert!(
        func_names.contains(&"_physics_process"),
        "Should have _physics_process: {:?}",
        func_names
    );

    // Exports should include speed
    let exports = script["exports"].as_array().unwrap();
    let export_names: Vec<&str> = exports
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert!(
        export_names.contains(&"speed"),
        "Should have speed export: {:?}",
        export_names
    );
}

/// Test: Script query snapshot
#[tokio::test]
async fn test_script_query_snapshot() {
    let schema = build_test_schema();
    let query = r#"
        query {
            script(path: "res://scripts/player.gd") {
                path
                extends
                functions {
                    name
                }
                exports {
                    name
                    type
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    insta::assert_json_snapshot!("script_player", data);
}

/// Test: Scene query returns None for non-existent file
#[tokio::test]
async fn test_scene_not_found() {
    let schema = build_test_schema();
    let query = r#"
        query {
            scene(path: "res://nonexistent.tscn") {
                path
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty());

    let data = result.data.into_json().unwrap();
    assert!(data["scene"].is_null());
}

/// Test: Script query returns None for non-existent file
#[tokio::test]
async fn test_script_not_found() {
    let schema = build_test_schema();
    let query = r#"
        query {
            script(path: "res://nonexistent.gd") {
                path
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty());

    let data = result.data.into_json().unwrap();
    assert!(data["script"].is_null());
}
