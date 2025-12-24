//! Schema Contract Tests (Phase 1)
//!
//! These tests ensure the GraphQL schema remains stable and consistent.
//! - SDL snapshot test: Detects unintended schema changes
//! - Query validation tests: Ensures representative queries are type-valid

use godot_mcp_rs::graphql::{build_schema, GqlSchema};

/// Get the SDL from the built schema
fn get_schema_sdl() -> String {
    let schema = build_schema();
    schema.sdl()
}

/// Test: SDL snapshot (golden test)
///
/// This test ensures the schema doesn't change unexpectedly.
/// If the schema intentionally changes, update the snapshot with:
/// `cargo insta review` or `cargo insta accept`
#[test]
fn test_schema_sdl_snapshot() {
    let sdl = get_schema_sdl();
    insta::assert_snapshot!("schema_sdl", sdl);
}

/// Test: Representative queries are syntactically valid
///
/// These queries should parse and validate against the schema.
mod query_validation {
    use super::*;

    async fn validate_query(schema: &GqlSchema, query: &str) -> Result<(), String> {
        let result = schema.execute(query).await;
        if result.errors.is_empty() {
            Ok(())
        } else {
            Err(result
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join(", "))
        }
    }

    /// Test: Project query structure
    #[tokio::test]
    async fn test_project_query_valid() {
        let schema = build_schema();
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
                    validation {
                        isValid
                        errors {
                            file
                            line
                            message
                            severity
                        }
                        warnings {
                            file
                            message
                        }
                    }
                }
            }
        "#;

        let result = validate_query(&schema, query).await;
        assert!(result.is_ok(), "Project query failed: {:?}", result.err());
    }

    /// Test: Scene query with nested structure
    #[tokio::test]
    async fn test_scene_query_valid() {
        let schema = build_schema();
        let query = r#"
            query {
                scene(path: "res://player.tscn") {
                    path
                    root {
                        name
                        type
                        path
                        properties {
                            name
                            value
                            type
                        }
                        children {
                            name
                            type
                            children {
                                name
                                type
                            }
                        }
                        groups
                        signals {
                            fromNode
                            signal
                            toNode
                            method
                        }
                    }
                    allNodes {
                        name
                        type
                    }
                    externalResources {
                        id
                        type
                        path
                    }
                }
            }
        "#;

        let result = validate_query(&schema, query).await;
        assert!(result.is_ok(), "Scene query failed: {:?}", result.err());
    }

    /// Test: Script query
    #[tokio::test]
    async fn test_script_query_valid() {
        let schema = build_schema();
        let query = r#"
            query {
                script(path: "res://player.gd") {
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
                    signals {
                        name
                        arguments
                    }
                    exports {
                        name
                        type
                        defaultValue
                    }
                }
            }
        "#;

        let result = validate_query(&schema, query).await;
        assert!(result.is_ok(), "Script query failed: {:?}", result.err());
    }

    /// Test: Live scene query
    #[tokio::test]
    async fn test_current_scene_query_valid() {
        let schema = build_schema();
        let query = r#"
            query {
                currentScene {
                    path
                    root {
                        name
                        type
                        path
                        globalPosition {
                            x
                            y
                            z
                        }
                        globalPosition2D {
                            x
                            y
                        }
                        properties {
                            name
                            value
                        }
                        children {
                            name
                            type
                        }
                        availableSignals {
                            name
                            arguments
                        }
                        connectedSignals {
                            fromNode
                            signal
                            toNode
                            method
                        }
                    }
                    selectedNodes {
                        name
                        path
                    }
                }
            }
        "#;

        let result = validate_query(&schema, query).await;
        assert!(
            result.is_ok(),
            "CurrentScene query failed: {:?}",
            result.err()
        );
    }

    /// Test: gatherContext query
    #[tokio::test]
    async fn test_gather_context_query_valid() {
        let schema = build_schema();
        let query = r#"
            query {
                gatherContext(input: {
                    entryPoint: "res://scenes/player.tscn"
                    depth: 2
                    include: [SCENE, SCRIPT]
                }) {
                    entryPoint
                    main {
                        path
                        type
                    }
                    dependencies {
                        path
                        type
                    }
                    dependents {
                        path
                        type
                    }
                    resources {
                        path
                        type
                    }
                    summary {
                        totalFiles
                        totalFunctions
                    }
                }
            }
        "#;

        let result = validate_query(&schema, query).await;
        assert!(
            result.is_ok(),
            "GatherContext query failed: {:?}",
            result.err()
        );
    }

    /// Test: dependencyGraph query
    #[tokio::test]
    async fn test_dependency_graph_query_valid() {
        let schema = build_schema();
        let query = r#"
            query {
                dependencyGraph(input: {
                    fileTypes: [SCENE, SCRIPT]
                    format: MERMAID
                }) {
                    nodes(filter: { isUnused: true }, limit: 10, offset: 0) {
                        id
                        label
                        type
                        inDegree
                        outDegree
                        isUnused
                    }
                    edges {
                        from
                        to
                        referenceType
                    }
                    stats {
                        nodeCount
                        edgeCount
                        unusedCount
                        hasCycles
                        cyclePaths
                    }
                    exportedData
                }
            }
        "#;

        let result = validate_query(&schema, query).await;
        assert!(
            result.is_ok(),
            "DependencyGraph query failed: {:?}",
            result.err()
        );
    }
}

/// Test: Representative mutations are syntactically valid
mod mutation_validation {
    use super::*;

    async fn validate_mutation(schema: &GqlSchema, mutation: &str) -> Result<(), String> {
        let result = schema.execute(mutation).await;
        if result.errors.is_empty() {
            Ok(())
        } else {
            Err(result
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join(", "))
        }
    }

    /// Test: addNode mutation
    #[tokio::test]
    async fn test_add_node_mutation_valid() {
        let schema = build_schema();
        let mutation = r#"
            mutation {
                addNode(input: {
                    parent: "/root/Main"
                    name: "Enemy"
                    type: "CharacterBody3D"
                    properties: [
                        { name: "position", value: "Vector3(10, 0, 5)" }
                    ]
                    groups: ["enemies"]
                }) {
                    success
                    node {
                        name
                        type
                        path
                    }
                    message
                }
            }
        "#;

        let result = validate_mutation(&schema, mutation).await;
        assert!(
            result.is_ok(),
            "addNode mutation failed: {:?}",
            result.err()
        );
    }

    /// Test: setProperty mutation
    #[tokio::test]
    async fn test_set_property_mutation_valid() {
        let schema = build_schema();
        let mutation = r#"
            mutation {
                setProperty(input: {
                    nodePath: "/root/Main/Player"
                    property: "position"
                    value: "Vector3(0, 0, 0)"
                }) {
                    success
                    message
                }
            }
        "#;

        let result = validate_mutation(&schema, mutation).await;
        assert!(
            result.is_ok(),
            "setProperty mutation failed: {:?}",
            result.err()
        );
    }

    /// Test: connectSignal mutation
    #[tokio::test]
    async fn test_connect_signal_mutation_valid() {
        let schema = build_schema();
        let mutation = r#"
            mutation {
                connectSignal(input: {
                    fromNode: "/root/Main/Enemy"
                    signal: "body_entered"
                    toNode: "/root/Main/Player"
                    method: "_on_enemy_contact"
                }) {
                    success
                    message
                }
            }
        "#;

        let result = validate_mutation(&schema, mutation).await;
        assert!(
            result.is_ok(),
            "connectSignal mutation failed: {:?}",
            result.err()
        );
    }

    /// Test: validateMutation mutation (safe change flow)
    #[tokio::test]
    async fn test_validate_mutation_valid() {
        let schema = build_schema();
        let mutation = r#"
            mutation {
                validateMutation(input: {
                    operations: [
                        {
                            type: ADD_NODE
                            args: { parent: "/root", name: "Test", type: "Node3D" }
                        },
                        {
                            type: SET_PROPERTY
                            args: { nodePath: "/root/Test", property: "visible", value: "true" }
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
                    warnings {
                        operationIndex
                        message
                    }
                    validationTimeMs
                }
            }
        "#;

        let result = validate_mutation(&schema, mutation).await;
        assert!(
            result.is_ok(),
            "validateMutation mutation failed: {:?}",
            result.err()
        );
    }

    /// Test: applyMutation mutation
    #[tokio::test]
    async fn test_apply_mutation_valid() {
        let schema = build_schema();
        let mutation = r#"
            mutation {
                applyMutation(input: {
                    operations: [
                        {
                            type: ADD_NODE
                            args: { parent: "/root", name: "Test", type: "Node3D" }
                        }
                    ]
                    createBackup: true
                    backupDescription: "Before test changes"
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

        let result = validate_mutation(&schema, mutation).await;
        assert!(
            result.is_ok(),
            "applyMutation mutation failed: {:?}",
            result.err()
        );
    }

    /// Test: previewMutation mutation
    #[tokio::test]
    async fn test_preview_mutation_valid() {
        let schema = build_schema();
        let mutation = r#"
            mutation {
                previewMutation(input: {
                    operations: [
                        {
                            type: ADD_NODE
                            args: { parent: "/root", name: "Test", type: "Node3D" }
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

        let result = validate_mutation(&schema, mutation).await;
        assert!(
            result.is_ok(),
            "previewMutation mutation failed: {:?}",
            result.err()
        );
    }
}
