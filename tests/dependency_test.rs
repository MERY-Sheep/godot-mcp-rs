//! Dependency Graph Tests (Phase 5)
//!
//! Tests for gatherContext and dependencyGraph queries.

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
// gatherContext Tests
// ======================

/// Test: gatherContext returns valid result for scene entry point
#[tokio::test]
async fn test_gather_context_scene() {
    let schema = build_test_schema();
    let query = r#"
        query {
            gatherContext(input: {
                entryPoint: "res://scenes/player.tscn"
            }) {
                entryPoint
                main {
                    path
                    type
                    scene {
                        path
                        root {
                            name
                            type
                        }
                    }
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

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let ctx = &data["gatherContext"];

    assert_eq!(ctx["entryPoint"], "res://scenes/player.tscn");
    assert_eq!(ctx["main"]["path"], "res://scenes/player.tscn");
    assert_eq!(ctx["main"]["type"], "SCENE");
}

/// Test: gatherContext with depth parameter
#[tokio::test]
async fn test_gather_context_depth() {
    let schema = build_test_schema();
    let query = r#"
        query {
            gatherContext(input: {
                entryPoint: "res://scenes/player.tscn",
                depth: 1
            }) {
                dependencies {
                    path
                }
                summary {
                    totalFiles
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
}

/// Test: gatherContext for script entry point
#[tokio::test]
async fn test_gather_context_script() {
    let schema = build_test_schema();
    let query = r#"
        query {
            gatherContext(input: {
                entryPoint: "res://scripts/player.gd"
            }) {
                entryPoint
                main {
                    path
                    type
                    script {
                        path
                        extends
                    }
                }
                summary {
                    totalFunctions
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let ctx = &data["gatherContext"];

    assert_eq!(ctx["main"]["type"], "SCRIPT");
}

// ======================
// dependencyGraph Tests
// ======================

/// Test: dependencyGraph returns valid structure
#[tokio::test]
async fn test_dependency_graph_basic() {
    let schema = build_test_schema();
    let query = r#"
        query {
            dependencyGraph {
                nodes {
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
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let graph = &data["dependencyGraph"];

    // Should have at least some nodes (player.tscn, player.gd)
    assert!(
        graph["stats"]["nodeCount"].as_i64().unwrap() >= 1,
        "Should have at least 1 node"
    );
}

/// Test: dependencyGraph with MERMAID format
#[tokio::test]
async fn test_dependency_graph_mermaid() {
    let schema = build_test_schema();
    let query = r#"
        query {
            dependencyGraph(input: { format: MERMAID }) {
                exportedData
                stats {
                    nodeCount
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let graph = &data["dependencyGraph"];

    let exported = graph["exportedData"].as_str().unwrap_or("");
    assert!(
        exported.contains("graph LR"),
        "MERMAID output should contain 'graph LR'"
    );
}

/// Test: dependencyGraph with DOT format
#[tokio::test]
async fn test_dependency_graph_dot() {
    let schema = build_test_schema();
    let query = r#"
        query {
            dependencyGraph(input: { format: DOT }) {
                exportedData
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let graph = &data["dependencyGraph"];

    let exported = graph["exportedData"].as_str().unwrap_or("");
    assert!(
        exported.contains("digraph"),
        "DOT output should contain 'digraph'"
    );
}

/// Test: dependencyGraph with JSON format
#[tokio::test]
async fn test_dependency_graph_json() {
    let schema = build_test_schema();
    let query = r#"
        query {
            dependencyGraph(input: { format: JSON }) {
                exportedData
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let graph = &data["dependencyGraph"];

    let exported = graph["exportedData"].as_str().unwrap_or("");
    assert!(
        exported.contains("\"nodes\"") && exported.contains("\"edges\""),
        "JSON output should contain nodes and edges"
    );
}

/// Test: dependencyGraph node filtering
#[tokio::test]
async fn test_dependency_graph_filter_unused() {
    let schema = build_test_schema();
    let query = r#"
        query {
            dependencyGraph {
                nodes(filter: { isUnused: true }) {
                    id
                    isUnused
                }
                stats {
                    unusedCount
                }
            }
        }
    "#;

    let result = schema.execute(query).await;
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let nodes = data["dependencyGraph"]["nodes"].as_array().unwrap();

    // All filtered nodes should be unused
    for node in nodes {
        assert_eq!(node["isUnused"], true);
    }
}

// ======================
// Dependency Resolver Unit Tests
// ======================

#[cfg(test)]
mod dependency_resolver_tests {
    use godot_mcp_rs::graphql::dependency_resolver::*;

    #[test]
    fn test_build_dependency_graph_returns_data() {
        use godot_mcp_rs::graphql::GqlContext;
        use std::path::PathBuf;

        let ctx = GqlContext::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_project"));

        let (nodes, edges) = build_dependency_graph(&ctx);

        // Should have at least some nodes
        assert!(!nodes.is_empty(), "Should have at least one node");
    }
}
