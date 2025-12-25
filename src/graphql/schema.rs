//! GraphQL schema root definitions
//!
//! Implements Query and Mutation roots as defined in `docs/gql/schema.graphql`.

use async_graphql::{Context, EmptySubscription, Object, Schema};

use super::codegen_resolver;
use super::context::GqlContext;
use super::dependency_resolver;
use super::live_resolver;
use super::refactoring_resolver;
use super::resolver;
use super::shader_resolver;
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

    // ========== Phase 3: Debug Enhanced ==========

    /// Get parse errors from a script (live)
    async fn parse_errors(&self, ctx: &Context<'_>, script_path: String) -> Vec<ParseError> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_parse_errors(gql_ctx, script_path).await
    }

    /// Get stack frame variables during debugging (live)
    async fn stack_frame_vars(
        &self,
        ctx: &Context<'_>,
        frame_index: Option<i32>,
    ) -> Vec<StackVariable> {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_stack_frame_vars(gql_ctx, frame_index).await
    }

    // ========== Phase 3: Code Understanding ==========

    /// Get class hierarchy for a script
    async fn class_hierarchy(&self, ctx: &Context<'_>, script_path: String) -> ClassHierarchy {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        refactoring_resolver::resolve_class_hierarchy(gql_ctx, &script_path)
    }

    /// Find references to a symbol
    async fn find_references(
        &self,
        ctx: &Context<'_>,
        symbol: String,
        scope: Option<String>,
    ) -> SymbolReferences {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        refactoring_resolver::resolve_find_references(gql_ctx, &symbol, scope.as_deref())
    }

    /// Get autoloads list
    async fn autoloads(&self, ctx: &Context<'_>) -> AutoloadsResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        refactoring_resolver::resolve_autoloads(gql_ctx)
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

    // ========== Transaction operations ==========

    /// Begin a transaction - groups subsequent operations into a single Undo action
    async fn begin_transaction(&self, ctx: &Context<'_>, name: String) -> TransactionResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_begin_transaction(gql_ctx, name).await
    }

    /// Commit the current transaction
    async fn commit_transaction(&self, ctx: &Context<'_>) -> TransactionResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_commit_transaction(gql_ctx).await
    }

    /// Rollback the current transaction - discards all changes
    async fn rollback_transaction(&self, ctx: &Context<'_>) -> TransactionResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_rollback_transaction(gql_ctx).await
    }

    // ========== Phase 3: Refactoring ==========

    /// Rename a symbol across the project
    async fn rename_symbol(
        &self,
        ctx: &Context<'_>,
        input: RenameSymbolInput,
    ) -> RenameSymbolResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        refactoring_resolver::resolve_rename_symbol(gql_ctx, &input)
    }

    /// Extract code block to a new function
    async fn extract_function(
        &self,
        ctx: &Context<'_>,
        input: ExtractFunctionInput,
    ) -> ExtractFunctionResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        refactoring_resolver::resolve_extract_function(gql_ctx, &input)
    }

    /// Move node to a new scene
    async fn move_node_to_scene(
        &self,
        ctx: &Context<'_>,
        input: MoveNodeToSceneInput,
    ) -> MoveNodeToSceneResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_move_node_to_scene(gql_ctx, input).await
    }

    // ========== Phase 3: Code Generation ==========

    /// Generate input handler code
    async fn generate_input_handler(
        &self,
        ctx: &Context<'_>,
        input: GenerateInputHandlerInput,
    ) -> CodeGenerationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        codegen_resolver::resolve_generate_input_handler(gql_ctx, &input)
    }

    /// Generate state machine boilerplate
    async fn generate_state_machine(
        &self,
        ctx: &Context<'_>,
        input: GenerateStateMachineInput,
    ) -> CodeGenerationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        codegen_resolver::resolve_generate_state_machine(gql_ctx, &input)
    }

    /// Generate test script
    async fn generate_test_script(
        &self,
        ctx: &Context<'_>,
        input: GenerateTestScriptInput,
    ) -> CodeGenerationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        codegen_resolver::resolve_generate_test_script(gql_ctx, &input)
    }

    // ========== Phase 3: Shader ==========

    /// Validate shader code (file-based)
    async fn validate_shader(&self, input: ValidateShaderInput) -> ShaderValidationResult {
        shader_resolver::resolve_validate_shader(&input)
    }

    /// Validate shader code (live)
    async fn validate_shader_live(
        &self,
        ctx: &Context<'_>,
        input: ValidateShaderInput,
    ) -> ShaderValidationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_validate_shader_live(gql_ctx, input).await
    }

    /// Create visual shader node
    async fn create_visual_shader_node(
        &self,
        ctx: &Context<'_>,
        input: CreateVisualShaderNodeInput,
    ) -> OperationResult {
        let gql_ctx = ctx.data::<GqlContext>().expect("GqlContext not found");
        live_resolver::resolve_create_visual_shader_node(gql_ctx, input).await
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
