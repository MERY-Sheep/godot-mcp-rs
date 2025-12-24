//! GraphQL schema root definitions
//!
//! Implements Query and Mutation roots as defined in `docs/gql/schema.graphql`.

use async_graphql::{Context, EmptySubscription, Object, Schema};

use super::context::GqlContext;
use super::dependency_resolver;
use super::live_resolver;
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
    async fn current_scene(&self, ctx: &Context<'_>) -> Option<LiveScene> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_current_scene(gql_ctx).await
    }

    /// Get node details (live)
    async fn node(&self, ctx: &Context<'_>, path: String) -> Option<LiveNode> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_node(gql_ctx, path).await
    }

    /// Get Godot node type information
    async fn node_type_info(&self, type_name: String) -> Option<NodeTypeInfo> {
        resolver::resolve_node_type_info(&type_name)
    }

    /// Gather context from entry point (index-chan inspired)
    async fn gather_context(
        &self,
        ctx: &Context<'_>,
        input: GatherContextInput,
    ) -> GatheredContext {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        dependency_resolver::resolve_gather_context(gql_ctx, input)
    }

    /// Get project dependency graph
    async fn dependency_graph(
        &self,
        ctx: &Context<'_>,
        input: Option<DependencyGraphInput>,
    ) -> DependencyGraph {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        dependency_resolver::resolve_dependency_graph(gql_ctx, input)
    }

    // ========== Debugging (Phase 2) ==========

    /// Get debugger errors
    async fn debugger_errors(&self, ctx: &Context<'_>) -> Vec<DebuggerError> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_debugger_errors(gql_ctx).await
    }

    /// Get logs
    async fn logs(&self, ctx: &Context<'_>, limit: Option<i32>) -> Vec<LogEntry> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_logs(gql_ctx, limit).await
    }

    /// Get object by ID
    async fn object_by_id(&self, ctx: &Context<'_>, object_id: String) -> Option<GodotObject> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_object_by_id(gql_ctx, object_id).await
    }
}

/// GraphQL Mutation Root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // ========== File-based operations ==========

    async fn create_scene(&self, ctx: &Context<'_>, input: CreateSceneInput) -> SceneResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::create_scene(gql_ctx, &input)
    }

    async fn create_scene_from_template(&self, _input: TemplateSceneInput) -> SceneResult {
        // TODO: Implement resolver
        SceneResult {
            success: false,
            scene: None,
            message: Some("Not implemented".to_string()),
        }
    }

    async fn create_script(&self, ctx: &Context<'_>, input: CreateScriptInput) -> ScriptResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::create_script(gql_ctx, &input)
    }

    // ========== Live operations ==========

    async fn add_node(&self, ctx: &Context<'_>, input: AddNodeInput) -> NodeResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_add_node(gql_ctx, input).await
    }

    async fn remove_node(&self, ctx: &Context<'_>, path: String) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_remove_node(gql_ctx, path).await
    }

    async fn duplicate_node(&self, _path: String) -> NodeResult {
        // TODO: Implement resolver (Phase 4)
        NodeResult::err(
            GqlStructuredError::new(
                "NOT_IMPLEMENTED",
                GqlErrorCategory::Schema,
                "Not implemented",
            )
            .with_suggestion("この機能は Phase 4 で実装予定です"),
        )
    }

    async fn reparent_node(&self, _path: String, _new_parent: String) -> NodeResult {
        // TODO: Implement resolver (Phase 4)
        NodeResult::err(
            GqlStructuredError::new(
                "NOT_IMPLEMENTED",
                GqlErrorCategory::Schema,
                "Not implemented",
            )
            .with_suggestion("この機能は Phase 4 で実装予定です"),
        )
    }

    async fn set_property(&self, ctx: &Context<'_>, input: SetPropertyInput) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_set_property(gql_ctx, input).await
    }

    async fn set_properties(
        &self,
        ctx: &Context<'_>,
        node_path: String,
        properties: Vec<PropertyInput>,
    ) -> OperationResult {
        // Execute each property set in sequence
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        for prop in properties {
            let input = SetPropertyInput {
                node_path: node_path.clone(),
                property: prop.name,
                value: prop.value,
            };
            let result = live_resolver::resolve_set_property(gql_ctx, input).await;
            if !result.success {
                return result;
            }
        }
        OperationResult::ok()
    }

    async fn connect_signal(
        &self,
        ctx: &Context<'_>,
        input: ConnectSignalInput,
    ) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_connect_signal(gql_ctx, input).await
    }

    async fn disconnect_signal(
        &self,
        ctx: &Context<'_>,
        input: DisconnectSignalInput,
    ) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_disconnect_signal(gql_ctx, input).await
    }

    async fn add_to_group(
        &self,
        ctx: &Context<'_>,
        node_path: String,
        group: String,
    ) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_add_to_group(gql_ctx, node_path, group).await
    }

    async fn remove_from_group(
        &self,
        ctx: &Context<'_>,
        node_path: String,
        group: String,
    ) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_remove_from_group(gql_ctx, node_path, group).await
    }

    async fn save_scene(&self, ctx: &Context<'_>) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_save_scene(gql_ctx).await
    }

    async fn open_scene(&self, ctx: &Context<'_>, path: String) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_open_scene(gql_ctx, path).await
    }

    // ========== Development / Testing ==========

    async fn run_tests(&self, ctx: &Context<'_>, input: RunTestsInput) -> TestExecutionResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::resolve_run_tests(gql_ctx, &input).await
    }

    // ========== Debugging Operations (Phase 2) ==========

    async fn pause(&self, ctx: &Context<'_>) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_pause(gql_ctx).await
    }

    async fn resume(&self, ctx: &Context<'_>) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_resume(gql_ctx).await
    }

    async fn step(&self, ctx: &Context<'_>) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_step(gql_ctx).await
    }

    async fn set_breakpoint(&self, ctx: &Context<'_>, input: BreakpointInput) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_set_breakpoint(gql_ctx, input).await
    }

    async fn remove_breakpoint(
        &self,
        ctx: &Context<'_>,
        input: BreakpointInput,
    ) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_remove_breakpoint(gql_ctx, input).await
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

    async fn preview_mutation(&self, ctx: &Context<'_>, input: MutationPlanInput) -> PreviewResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        resolver::preview_mutation(gql_ctx, &input)
    }

    async fn apply_mutation(&self, ctx: &Context<'_>, input: ApplyMutationInput) -> ApplyResult {
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
