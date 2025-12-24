//! GQL MCP Tools - GraphQL interface for Godot operations
//!
//! Provides 3 MCP tools:
//! - `godot_query`: Execute GraphQL queries
//! - `godot_mutate`: Execute GraphQL mutations
//! - `godot_introspect`: Get schema (SDL or introspection)

use rmcp::{model::CallToolResult, ErrorData as McpError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::graphql::{build_schema_with_context, GqlContext};

/// Request for executing a GraphQL query
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GqlQueryRequest {
    /// GraphQL query string
    pub query: String,
    /// Optional variables as JSON object
    pub variables: Option<serde_json::Value>,
}

/// Request for executing a GraphQL mutation
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GqlMutateRequest {
    /// GraphQL mutation string
    pub mutation: String,
    /// Optional variables as JSON object
    pub variables: Option<serde_json::Value>,
}

/// Request for getting the GraphQL schema
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GqlIntrospectRequest {
    /// Format: "SDL" (default) or "INTROSPECTION"
    pub format: Option<String>,
}

/// Execute a GraphQL query
pub async fn handle_godot_query(
    base_path: &Path,
    args: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<CallToolResult, McpError> {
    let request: GqlQueryRequest = match args {
        Some(map) => serde_json::from_value(serde_json::Value::Object(map)).map_err(|e| {
            McpError::invalid_params(format!("Invalid request parameters: {}", e), None)
        })?,
        None => return Err(McpError::invalid_params("Missing request parameters", None)),
    };

    let ctx = GqlContext::new(base_path.to_path_buf());
    let schema = build_schema_with_context(ctx);

    // Build request with optional variables
    let gql_request = if let Some(vars) = request.variables {
        async_graphql::Request::new(&request.query)
            .variables(async_graphql::Variables::from_json(vars))
    } else {
        async_graphql::Request::new(&request.query)
    };

    let response = schema.execute(gql_request).await;

    // Serialize the response
    let response_json = serde_json::to_string_pretty(&response)
        .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize response: {}\"}}", e));

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        response_json,
    )]))
}

/// Execute a GraphQL mutation
pub async fn handle_godot_mutate(
    base_path: &Path,
    args: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<CallToolResult, McpError> {
    let request: GqlMutateRequest = match args {
        Some(map) => serde_json::from_value(serde_json::Value::Object(map)).map_err(|e| {
            McpError::invalid_params(format!("Invalid request parameters: {}", e), None)
        })?,
        None => return Err(McpError::invalid_params("Missing request parameters", None)),
    };

    let ctx = GqlContext::new(base_path.to_path_buf());
    let schema = build_schema_with_context(ctx);

    // Build request with optional variables
    let gql_request = if let Some(vars) = request.variables {
        async_graphql::Request::new(&request.mutation)
            .variables(async_graphql::Variables::from_json(vars))
    } else {
        async_graphql::Request::new(&request.mutation)
    };

    let response = schema.execute(gql_request).await;

    // Serialize the response
    let response_json = serde_json::to_string_pretty(&response)
        .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize response: {}\"}}", e));

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        response_json,
    )]))
}

/// Get the GraphQL schema
pub async fn handle_godot_introspect(
    base_path: &Path,
    args: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<CallToolResult, McpError> {
    let request: GqlIntrospectRequest = match args {
        Some(map) => serde_json::from_value(serde_json::Value::Object(map)).unwrap_or_default(),
        None => GqlIntrospectRequest::default(),
    };

    let ctx = GqlContext::new(base_path.to_path_buf());
    let schema = build_schema_with_context(ctx);

    let format = request.format.unwrap_or_else(|| "SDL".to_string());

    let result = match format.to_uppercase().as_str() {
        "SDL" => {
            // Return the SDL (Schema Definition Language)
            schema.sdl()
        }
        "INTROSPECTION" => {
            // Execute introspection query
            let introspection_query = r#"
                query IntrospectionQuery {
                    __schema {
                        queryType { name }
                        mutationType { name }
                        types {
                            name
                            kind
                            description
                            fields(includeDeprecated: true) {
                                name
                                description
                                args {
                                    name
                                    description
                                    type { name kind ofType { name kind } }
                                    defaultValue
                                }
                                type { name kind ofType { name kind } }
                            }
                        }
                    }
                }
            "#;
            let response = schema.execute(introspection_query).await;
            serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                format!(
                    "{{\"error\": \"Failed to serialize introspection: {}\"}}",
                    e
                )
            })
        }
        _ => {
            return Err(McpError::invalid_params(
                format!("Invalid format '{}'. Use 'SDL' or 'INTROSPECTION'", format),
                None,
            ))
        }
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_handle_godot_query_project() {
        let base_path = PathBuf::from(".");
        let mut args = serde_json::Map::new();
        args.insert(
            "query".to_string(),
            serde_json::json!("{ project { name path stats { sceneCount scriptCount } } }"),
        );

        let result = handle_godot_query(&base_path, Some(args)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_godot_introspect_sdl() {
        let base_path = PathBuf::from(".");
        let mut args = serde_json::Map::new();
        args.insert("format".to_string(), serde_json::json!("SDL"));

        let result = handle_godot_introspect(&base_path, Some(args)).await;
        assert!(result.is_ok());

        // Check that SDL contains expected types
        if let Ok(tool_result) = result {
            let content = format!("{:?}", tool_result);
            assert!(content.contains("Query"));
            assert!(content.contains("Mutation"));
        }
    }

    #[tokio::test]
    async fn test_handle_godot_mutate_validate() {
        let base_path = PathBuf::from(".");
        let mut args = serde_json::Map::new();
        args.insert(
            "mutation".to_string(),
            serde_json::json!(r#"
                mutation {
                    validateMutation(input: {
                        operations: [
                            { type: ADD_NODE, args: { parent: "/root", name: "Test", type: "Node3D" } }
                        ]
                    }) {
                        isValid
                        errors { code message }
                    }
                }
            "#),
        );

        let result = handle_godot_mutate(&base_path, Some(args)).await;
        assert!(result.is_ok());
    }
}
