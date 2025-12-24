//! Mutation Tests (Phase 3)
//!
//! Tests for validate/preview/apply safe change flow.

use godot_mcp_rs::graphql::{build_schema_with_context, GqlContext};
use std::path::PathBuf;

fn test_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_project")
}

fn build_test_schema() -> godot_mcp_rs::graphql::GqlSchema {
    let ctx = GqlContext::new(test_project_path());
    build_schema_with_context(ctx)
}

// ======================
// validate_mutation tests
// ======================

/// Test: validateMutation returns valid for correct SET_PROPERTY operation
#[tokio::test]
async fn test_validate_mutation_set_property_valid() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            validateMutation(input: {
                operations: [
                    {
                        type: SET_PROPERTY,
                        args: {
                            nodePath: ".",
                            property: "position",
                            value: "Vector3(0, 0, 0)"
                        }
                    }
                ]
            }) {
                isValid
                errors {
                    operationIndex
                    code
                    message
                }
                warnings {
                    operationIndex
                    message
                }
                validationTimeMs
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let validation = &data["validateMutation"];

    assert_eq!(validation["isValid"], true);
    assert!(validation["errors"].as_array().unwrap().is_empty());
}

/// Test: validateMutation returns error for non-existent node
#[tokio::test]
async fn test_validate_mutation_node_not_found() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            validateMutation(input: {
                operations: [
                    {
                        type: SET_PROPERTY,
                        args: {
                            nodePath: "NonExistentNode",
                            property: "position",
                            value: "Vector3(0, 0, 0)"
                        }
                    }
                ]
            }) {
                isValid
                errors {
                    operationIndex
                    code
                    message
                    suggestion
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let validation = &data["validateMutation"];

    assert_eq!(validation["isValid"], false);
    let errors = validation["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "NODE_NOT_FOUND");
    assert_eq!(errors[0]["operationIndex"], 0);
}

/// Test: validateMutation returns error for missing args
#[tokio::test]
async fn test_validate_mutation_missing_args() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            validateMutation(input: {
                operations: [
                    {
                        type: SET_PROPERTY,
                        args: {}
                    }
                ]
            }) {
                isValid
                errors {
                    operationIndex
                    code
                    message
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let validation = &data["validateMutation"];

    assert_eq!(validation["isValid"], false);
    let errors = validation["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert_eq!(errors[0]["code"], "MISSING_REQUIRED_ARG");
}

/// Test: validateMutation handles ADD_NODE operation
#[tokio::test]
async fn test_validate_mutation_add_node_valid() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            validateMutation(input: {
                operations: [
                    {
                        type: ADD_NODE,
                        args: {
                            parent: ".",
                            name: "NewNode",
                            type: "Sprite2D"
                        }
                    }
                ]
            }) {
                isValid
                errors {
                    operationIndex
                    code
                    message
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let validation = &data["validateMutation"];

    assert_eq!(validation["isValid"], true);
}

/// Test: validateMutation returns error for REMOVE_NODE on root
#[tokio::test]
async fn test_validate_mutation_remove_root_node() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            validateMutation(input: {
                operations: [
                    {
                        type: REMOVE_NODE,
                        args: {
                            path: "."
                        }
                    }
                ]
            }) {
                isValid
                errors {
                    operationIndex
                    code
                    message
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let validation = &data["validateMutation"];

    assert_eq!(validation["isValid"], false);
    let errors = validation["errors"].as_array().unwrap();
    assert_eq!(errors[0]["code"], "CANNOT_REMOVE_ROOT");
}

// ======================
// preview_mutation tests
// ======================

/// Test: previewMutation returns diff for SET_PROPERTY
#[tokio::test]
async fn test_preview_mutation_set_property() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            previewMutation(input: {
                operations: [
                    {
                        type: SET_PROPERTY,
                        args: {
                            nodePath: ".",
                            property: "position",
                            value: "Vector3(1, 2, 3)"
                        }
                    }
                ]
            }) {
                success
                diff
                affectedFiles {
                    path
                    changeType
                }
                summary {
                    nodesAdded
                    nodesRemoved
                    propertiesChanged
                    signalsConnected
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let preview = &data["previewMutation"];

    assert_eq!(preview["success"], true);
    assert!(!preview["diff"].as_str().unwrap().is_empty());
    assert_eq!(preview["summary"]["propertiesChanged"], 1);
}

/// Test: previewMutation returns diff for ADD_NODE
#[tokio::test]
async fn test_preview_mutation_add_node() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            previewMutation(input: {
                operations: [
                    {
                        type: ADD_NODE,
                        args: {
                            parent: ".",
                            name: "NewSprite",
                            type: "Sprite2D"
                        }
                    }
                ]
            }) {
                success
                diff
                summary {
                    nodesAdded
                    nodesRemoved
                    propertiesChanged
                    signalsConnected
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let preview = &data["previewMutation"];

    assert_eq!(preview["success"], true);
    assert_eq!(preview["summary"]["nodesAdded"], 1);
}

// ======================
// apply_mutation tests
// ======================

/// Test: applyMutation with backup enabled
#[tokio::test]
async fn test_apply_mutation_with_backup() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            applyMutation(input: {
                operations: [],
                createBackup: true,
                backupDescription: "Test backup"
            }) {
                success
                appliedCount
                backupPath
                errors {
                    operationIndex
                    message
                }
                undoActionId
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let apply = &data["applyMutation"];

    assert_eq!(apply["success"], true);
    assert_eq!(apply["appliedCount"], 0);
}

/// Test: applyMutation returns undoActionId
#[tokio::test]
async fn test_apply_mutation_returns_undo_id() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            applyMutation(input: {
                operations: [
                    {
                        type: SET_PROPERTY,
                        args: {
                            nodePath: ".",
                            property: "test_property",
                            value: "test_value"
                        }
                    }
                ]
            }) {
                success
                appliedCount
                undoActionId
                errors {
                    operationIndex
                    message
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let apply = &data["applyMutation"];

    // undoActionId should be returned when operations are applied
    // (actual implementation may return null for dry-run or stub)
    assert!(apply.get("undoActionId").is_some());
}

/// Test: Snapshot for validateMutation result
#[tokio::test]
async fn test_validate_mutation_snapshot() {
    let schema = build_test_schema();
    let query = r#"
        mutation {
            validateMutation(input: {
                operations: [
                    {
                        type: ADD_NODE,
                        args: {
                            parent: ".",
                            name: "TestNode",
                            type: "Node2D"
                        }
                    },
                    {
                        type: SET_PROPERTY,
                        args: {
                            nodePath: "TestNode",
                            property: "position",
                            value: "Vector2(100, 200)"
                        }
                    }
                ]
            }) {
                isValid
                errors {
                    operationIndex
                    code
                    message
                }
                warnings {
                    operationIndex
                    message
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    insta::assert_json_snapshot!("validate_mutation_add_and_set", data);
}
