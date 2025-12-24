//! GraphQL schema root definitions
//!
//! Implements Query and Mutation roots as defined in `docs/gql/schema.graphql`.

use async_graphql::{Context, EmptySubscription, Object, Schema};

use super::context::GqlContext;
use super::resolver;
use super::types::*;

/// GraphQL Query Root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get project information
    async fn project(&self, ctx: &Context<'_>) -> Project {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::resolve_project(gql_ctx)
    }

    /// Get scene file contents
    async fn scene(&self, ctx: &Context<'_>, path: String) -> Option<Scene> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::resolve_scene(gql_ctx, &path)
    }

    /// Get script file contents
    async fn script(&self, ctx: &Context<'_>, path: String) -> Option<Script> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::resolve_script(gql_ctx, &path)
    }

    /// Get current scene in editor (live)
    async fn current_scene(&self) -> Option<LiveScene> {
        // TODO: Implement live resolver (Phase 4)
        None
    }

    /// Get node details (live)
    async fn node(&self, _path: String) -> Option<LiveNode> {
        // TODO: Implement live resolver (Phase 4)
        None
    }

    /// Get Godot node type information
    async fn node_type_info(&self, _type_name: String) -> Option<NodeTypeInfo> {
        // TODO: Implement resolver
        None
    }

    /// Gather context from entry point (index-chan inspired)
    async fn gather_context(&self, input: GatherContextInput) -> GatheredContext {
        // TODO: Implement resolver (Phase 5)
        GatheredContext {
            entry_point: input.entry_point,
            main: ContextItem {
                path: String::new(),
                file_type: FileType::Scene,
                scene: None,
                script: None,
            },
            dependencies: vec![],
            dependents: vec![],
            resources: vec![],
            summary: ContextSummary {
                total_files: 0,
                total_functions: 0,
            },
        }
    }

    /// Get project dependency graph
    async fn dependency_graph(&self, _input: Option<DependencyGraphInput>) -> DependencyGraph {
        // TODO: Implement resolver (Phase 5)
        DependencyGraph {
            nodes: vec![],
            edges: vec![],
            stats: GraphStats {
                node_count: 0,
                edge_count: 0,
                unused_count: 0,
                has_cycles: false,
                cycle_paths: None,
            },
            exported_data: None,
        }
    }
}

/// GraphQL Mutation Root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // ========== File-based operations ==========

    async fn create_scene(&self, _input: CreateSceneInput) -> SceneResult {
        // TODO: Implement resolver
        SceneResult {
            success: false,
            scene: None,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn create_scene_from_template(&self, _input: TemplateSceneInput) -> SceneResult {
        // TODO: Implement resolver
        SceneResult {
            success: false,
            scene: None,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn create_script(&self, _input: CreateScriptInput) -> ScriptResult {
        // TODO: Implement resolver
        ScriptResult {
            success: false,
            script: None,
            message: Some("Not implemented".to_string()),
        }
    }

    // ========== Live operations ==========

    async fn add_node(&self, _input: AddNodeInput) -> NodeResult {
        // TODO: Implement resolver (Phase 4)
        NodeResult {
            success: false,
            node: None,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn remove_node(&self, _path: String) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn duplicate_node(&self, _path: String) -> NodeResult {
        // TODO: Implement resolver (Phase 4)
        NodeResult {
            success: false,
            node: None,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn reparent_node(&self, _path: String, _new_parent: String) -> NodeResult {
        // TODO: Implement resolver (Phase 4)
        NodeResult {
            success: false,
            node: None,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn set_property(&self, _input: SetPropertyInput) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn set_properties(
        &self,
        _node_path: String,
        _properties: Vec<PropertyInput>,
    ) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn connect_signal(&self, _input: ConnectSignalInput) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn disconnect_signal(&self, _input: DisconnectSignalInput) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn add_to_group(&self, _node_path: String, _group: String) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn remove_from_group(&self, _node_path: String, _group: String) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn save_scene(&self) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn open_scene(&self, _path: String) -> OperationResult {
        // TODO: Implement resolver (Phase 4)
        OperationResult {
            success: false,
            message: Some("Not implemented".to_string()),
        }
    }

    // ========== Safe change flow ==========

    async fn validate_mutation(
        &self,
        ctx: &Context<'_>,
        input: MutationPlanInput,
    ) -> MutationValidationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::validate_mutation(gql_ctx, &input)
    }

    async fn preview_mutation(
        &self,
        ctx: &Context<'_>,
        input: MutationPlanInput,
    ) -> PreviewResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::preview_mutation(gql_ctx, &input)
    }

    async fn apply_mutation(
        &self,
        ctx: &Context<'_>,
        input: ApplyMutationInput,
    ) -> ApplyResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::apply_mutation(gql_ctx, &input)
    }
}

/// GQL Schema type alias
pub type GqlSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Build the GraphQL schema with context
pub fn build_schema_with_context(ctx: GqlContext) -> GqlSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        .finish()
}

/// Build the GraphQL schema (for tests without context)
pub fn build_schema() -> GqlSchema {
    use std::path::PathBuf;
    let ctx = GqlContext::new(PathBuf::from("."));
    build_schema_with_context(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builds() {
        let _schema = build_schema();
    }
}
