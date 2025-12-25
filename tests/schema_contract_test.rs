//! Schema Contract Tests (Phase 1)
//!
//! These tests ensure the GraphQL schema remains stable and consistent.
//! - SDL snapshot test: Detects unintended schema changes
//! - Query validation tests: Ensures representative queries are type-valid
//! - Schema sync test: Ensures Rust implementation matches schema.graphql

use godot_mcp_rs::graphql::{build_schema, GqlSchema};
use std::collections::HashSet;

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

/// Extract type names from SDL (types, inputs, enums)
fn extract_type_names(sdl: &str) -> HashSet<String> {
    let mut types = HashSet::new();
    let type_pattern = regex::Regex::new(r"(?m)^(?:type|input|enum)\s+(\w+)").unwrap();

    for cap in type_pattern.captures_iter(sdl) {
        if let Some(name) = cap.get(1) {
            types.insert(name.as_str().to_string());
        }
    }
    types
}

/// Extract query/mutation field names from SDL
fn extract_operations(sdl: &str) -> (HashSet<String>, HashSet<String>) {
    let mut queries = HashSet::new();
    let mut mutations = HashSet::new();

    // Parser state
    let mut in_query = false;
    let mut in_mutation = false;
    let mut paren_depth: i32 = 0; // Track parenthesis nesting for multiline args

    // Field definition pattern: starts with field name followed by ( or :
    // Field names must start with lowercase letter (GraphQL convention)
    // Examples:
    //   project: Project!
    //   scene(path: String!): Scene
    //   setProperties(
    let field_start_pattern = regex::Regex::new(r"^\t+([a-z]\w*)\s*[\(:]").unwrap();

    for line in sdl.lines() {
        let trimmed = line.trim();

        // Match Query/QueryRoot type block (exact match with optional trailing space/brace)
        if trimmed == "type Query {"
            || trimmed.starts_with("type Query {")
            || trimmed == "type QueryRoot {"
            || trimmed.starts_with("type QueryRoot {")
        {
            in_query = true;
            in_mutation = false;
            paren_depth = 0;
            continue;
        }
        // Match Mutation/MutationRoot type block (exact match to avoid MutationValidationError etc.)
        if trimmed == "type Mutation {"
            || trimmed.starts_with("type Mutation {")
            || trimmed == "type MutationRoot {"
            || trimmed.starts_with("type MutationRoot {")
        {
            in_mutation = true;
            in_query = false;
            paren_depth = 0;
            continue;
        }
        // Any other type/input/enum definition closes current block
        if trimmed.starts_with("type ")
            || trimmed.starts_with("input ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("scalar ")
        {
            in_query = false;
            in_mutation = false;
            paren_depth = 0;
            continue;
        }
        // Close type block on }
        if trimmed == "}" && paren_depth == 0 {
            in_query = false;
            in_mutation = false;
            continue;
        }

        // Track parenthesis depth for multiline argument lists
        for ch in trimmed.chars() {
            match ch {
                '(' => paren_depth += 1,
                ')' => paren_depth = paren_depth.saturating_sub(1),
                _ => {}
            }
        }

        // Only extract field names when we're at the top level (not inside argument list)
        // and we're in a Query or Mutation block
        if (in_query || in_mutation) && paren_depth == 0 {
            // Skip lines that look like argument definitions (argName: Type)
            // These don't have leading whitespace pattern matching a field
            if let Some(cap) = field_start_pattern.captures(line) {
                if let Some(name) = cap.get(1) {
                    let field_name = name.as_str().to_string();
                    if in_query {
                        queries.insert(field_name);
                    } else if in_mutation {
                        mutations.insert(field_name);
                    }
                }
            }
        }
        // Also extract field when line contains '(' and ')' on same line (single-line field with args)
        else if (in_query || in_mutation) && trimmed.contains('(') && paren_depth == 0 {
            if let Some(cap) = field_start_pattern.captures(line) {
                if let Some(name) = cap.get(1) {
                    let field_name = name.as_str().to_string();
                    if in_query {
                        queries.insert(field_name);
                    } else if in_mutation {
                        mutations.insert(field_name);
                    }
                }
            }
        }
    }

    (queries, mutations)
}

/// Test: Schema matches source of truth (schema.graphql)
///
/// This test ensures the Rust implementation matches the schema.graphql file.
/// If this test fails, it means types or operations defined in schema.graphql
/// are missing from the Rust implementation.
#[test]
fn test_schema_matches_source_of_truth() {
    // Load schema.graphql (source of truth)
    let schema_file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("gql")
        .join("schema.graphql");

    let source_sdl = std::fs::read_to_string(&schema_file_path)
        .expect("Failed to read schema.graphql - this file should exist as the source of truth");

    // Get Rust implementation SDL
    let rust_sdl = get_schema_sdl();

    // Extract types from both
    let source_types = extract_type_names(&source_sdl);
    let rust_types = extract_type_names(&rust_sdl);

    // Known type mappings (schema.graphql name -> Rust name)
    // async-graphql generates different names for root types
    let type_mappings: HashSet<_> = [
        "Query",    // -> QueryRoot in async-graphql
        "Mutation", // -> MutationRoot in async-graphql
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    // Find missing types in Rust implementation (excluding known mappings)
    let missing_types: Vec<_> = source_types
        .difference(&rust_types)
        .filter(|t| !type_mappings.contains(*t))
        .cloned()
        .collect();

    // Extract operations
    let (source_queries, source_mutations) = extract_operations(&source_sdl);
    let (rust_queries, rust_mutations) = extract_operations(&rust_sdl);

    // Find missing queries
    let missing_queries: Vec<_> = source_queries.difference(&rust_queries).cloned().collect();

    // Find missing mutations
    let missing_mutations: Vec<_> = source_mutations
        .difference(&rust_mutations)
        .cloned()
        .collect();

    // Build error message
    let mut errors = Vec::new();

    if !missing_types.is_empty() {
        errors.push(format!(
            "Missing types in Rust implementation ({} types):\n  - {}",
            missing_types.len(),
            missing_types.join("\n  - ")
        ));
    }

    if !missing_queries.is_empty() {
        errors.push(format!(
            "Missing queries in Rust implementation ({} queries):\n  - {}",
            missing_queries.len(),
            missing_queries.join("\n  - ")
        ));
    }

    if !missing_mutations.is_empty() {
        errors.push(format!(
            "Missing mutations in Rust implementation ({} mutations):\n  - {}",
            missing_mutations.len(),
            missing_mutations.join("\n  - ")
        ));
    }

    if !errors.is_empty() {
        panic!(
            "\n\n=== Schema Sync Error ===\n\
            The Rust implementation does not match schema.graphql (source of truth).\n\n\
            {}\n\n\
            To fix this:\n\
            1. Add missing types to src/graphql/types.rs\n\
            2. Add missing resolvers to src/graphql/schema.rs\n\
            3. Run `cargo test test_schema_matches_source_of_truth` to verify\n\
            ===========================\n",
            errors.join("\n\n")
        );
    }
}

/// Test: Representative queries are syntactically valid
///
/// These queries should parse and validate against the schema.
mod query_validation {
    use super::*;
    use godot_mcp_rs::graphql::GqlContext;
    use std::path::PathBuf;

    fn test_context() -> GqlContext {
        GqlContext::new(PathBuf::from("."))
    }

    async fn validate_query(schema: &GqlSchema, query: &str) -> Result<(), String> {
        let ctx = test_context();
        let request = async_graphql::Request::new(query).data(ctx);
        let result = schema.execute(request).await;
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
    use godot_mcp_rs::graphql::GqlContext;
    use std::path::PathBuf;

    fn test_context() -> GqlContext {
        GqlContext::new(PathBuf::from("."))
    }

    async fn validate_mutation(schema: &GqlSchema, mutation: &str) -> Result<(), String> {
        let ctx = test_context();
        let request = async_graphql::Request::new(mutation).data(ctx);
        let result = schema.execute(request).await;
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
