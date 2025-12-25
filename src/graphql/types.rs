//! GraphQL type definitions
//!
//! These types correspond to `docs/gql/schema.graphql`.
//! Keep in sync with the SDL.

use async_graphql::{Enum, InputObject, Object, SimpleObject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ======================
// Scalar types
// ======================
// JSON scalar is handled by async-graphql's Json<T>

// ======================
// Structured Error Types (Phase 1)
// ======================

/// Error category for AI-friendly error handling
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum, Serialize, Deserialize)]
pub enum GqlErrorCategory {
    /// Connection errors (Godot plugin communication)
    Connection,
    /// Validation errors (input validation)
    Validation,
    /// Godot runtime errors
    Godot,
    /// File system errors
    FileSystem,
    /// Schema/Query errors
    Schema,
}

/// Error location information
#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize, Default)]
pub struct GqlErrorLocation {
    /// File path (res:// or file://)
    pub file: Option<String>,
    /// Line number (1-indexed)
    pub line: Option<i32>,
    /// Column number (1-indexed)
    pub column: Option<i32>,
}

/// Stack frame for error traces
#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct GqlErrorStackFrame {
    /// Function or method name
    pub function: String,
    /// File path
    pub file: Option<String>,
    /// Line number
    pub line: Option<i32>,
}

/// AI-friendly structured error with context for self-repair
#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct GqlStructuredError {
    /// Error code (e.g., "CONN_TIMEOUT", "VALIDATION_NODE_NOT_FOUND")
    pub code: String,
    /// Error category for grouping
    pub category: GqlErrorCategory,
    /// Human-readable error message
    pub message: String,
    /// Location where error occurred
    pub location: Option<GqlErrorLocation>,
    /// Stack trace for debugging
    pub stack_trace: Vec<GqlErrorStackFrame>,
    /// Suggested fix for AI agent
    pub suggestion: Option<String>,
    /// Related documentation URL
    pub help_url: Option<String>,
    /// Additional context as JSON
    pub context: Option<async_graphql::Json<HashMap<String, String>>>,
}

impl GqlStructuredError {
    /// Create a new structured error
    pub fn new(
        code: impl Into<String>,
        category: GqlErrorCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            category,
            message: message.into(),
            location: None,
            stack_trace: vec![],
            suggestion: None,
            help_url: None,
            context: None,
        }
    }

    /// Add a suggestion for fixing the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add context key-value pairs
    pub fn with_context(mut self, ctx: HashMap<String, String>) -> Self {
        self.context = Some(async_graphql::Json(ctx));
        self
    }
}

// ======================
// Core Types
// ======================

/// Project information
#[derive(Debug, Clone, SimpleObject)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub scenes: Vec<SceneFile>,
    pub scripts: Vec<ScriptFile>,
    pub stats: ProjectStats,
    pub validation: ProjectValidationResult,
}

/// Scene file reference
#[derive(Debug, Clone, SimpleObject)]
pub struct SceneFile {
    pub path: String,
}

/// Script file reference
#[derive(Debug, Clone, SimpleObject)]
pub struct ScriptFile {
    pub path: String,
}

/// Project statistics
#[derive(Debug, Clone, SimpleObject)]
pub struct ProjectStats {
    pub scene_count: i32,
    pub script_count: i32,
    pub resource_count: i32,
}

/// Scene structure from file
#[derive(Debug, Clone)]
pub struct Scene {
    pub path: String,
    pub root: SceneNode,
    pub all_nodes: Vec<SceneNode>,
    pub external_resources: Vec<ExternalResource>,
}

#[Object]
impl Scene {
    async fn path(&self) -> &str {
        &self.path
    }

    async fn root(&self) -> &SceneNode {
        &self.root
    }

    async fn all_nodes(&self) -> &[SceneNode] {
        &self.all_nodes
    }

    async fn external_resources(&self) -> &[ExternalResource] {
        &self.external_resources
    }
}

/// Scene node from file analysis
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub name: String,
    pub r#type: String,
    pub path: String,
    pub properties: Vec<Property>,
    pub children: Vec<SceneNode>,
    pub script: Option<Script>,
    pub groups: Vec<String>,
    pub signals: Vec<SignalConnection>,
}

#[Object]
impl SceneNode {
    async fn name(&self) -> &str {
        &self.name
    }

    #[graphql(name = "type")]
    async fn node_type(&self) -> &str {
        &self.r#type
    }

    async fn path(&self) -> &str {
        &self.path
    }

    async fn properties(&self) -> &[Property] {
        &self.properties
    }

    async fn property(&self, name: String) -> Option<&Property> {
        self.properties.iter().find(|p| p.name == name)
    }

    async fn children(&self) -> &[SceneNode] {
        &self.children
    }

    async fn script(&self) -> Option<&Script> {
        self.script.as_ref()
    }

    async fn groups(&self) -> &[String] {
        &self.groups
    }

    async fn signals(&self) -> &[SignalConnection] {
        &self.signals
    }
}

/// Live scene from editor
#[derive(Debug, Clone)]
pub struct LiveScene {
    pub path: Option<String>,
    pub root: LiveNode,
    pub selected_nodes: Vec<LiveNode>,
}

#[Object]
impl LiveScene {
    async fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    async fn root(&self) -> &LiveNode {
        &self.root
    }

    async fn selected_nodes(&self) -> &[LiveNode] {
        &self.selected_nodes
    }
}

/// Live node from editor
#[derive(Debug, Clone)]
pub struct LiveNode {
    pub name: String,
    pub r#type: String,
    pub path: String,
    pub global_position: Option<Vector3>,
    pub global_position_2d: Option<Vector2>,
    pub properties: Vec<Property>,
    pub children: Vec<LiveNode>,
    pub available_signals: Vec<SignalInfo>,
    pub connected_signals: Vec<SignalConnection>,
}

#[Object]
impl LiveNode {
    async fn name(&self) -> &str {
        &self.name
    }

    #[graphql(name = "type")]
    async fn node_type(&self) -> &str {
        &self.r#type
    }

    async fn path(&self) -> &str {
        &self.path
    }

    async fn global_position(&self) -> Option<&Vector3> {
        self.global_position.as_ref()
    }

    #[graphql(name = "globalPosition2D")]
    async fn global_position_2d(&self) -> Option<&Vector2> {
        self.global_position_2d.as_ref()
    }

    async fn properties(&self) -> &[Property] {
        &self.properties
    }

    async fn children(&self) -> &[LiveNode] {
        &self.children
    }

    async fn available_signals(&self) -> &[SignalInfo] {
        &self.available_signals
    }

    async fn connected_signals(&self) -> &[SignalConnection] {
        &self.connected_signals
    }
}

/// Script analysis result
#[derive(Debug, Clone)]
pub struct Script {
    pub path: String,
    pub extends: String,
    pub class_name: Option<String>,
    pub functions: Vec<Function>,
    pub variables: Vec<Variable>,
    pub signals: Vec<SignalDefinition>,
    pub exports: Vec<Variable>,
}

#[Object]
impl Script {
    async fn path(&self) -> &str {
        &self.path
    }

    async fn extends(&self) -> &str {
        &self.extends
    }

    async fn class_name(&self) -> Option<&str> {
        self.class_name.as_deref()
    }

    async fn functions(&self) -> &[Function] {
        &self.functions
    }

    async fn variables(&self) -> &[Variable] {
        &self.variables
    }

    async fn signals(&self) -> &[SignalDefinition] {
        &self.signals
    }

    async fn exports(&self) -> &[Variable] {
        &self.exports
    }
}

// ======================
// Property / Values
// ======================

#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
    #[graphql(name = "type")]
    pub property_type: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct PropertyInput {
    pub name: String,
    pub value: String,
}

// ======================
// Vector helpers
// ======================

#[derive(Debug, Clone, SimpleObject)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// ======================
// Signals / Scripts
// ======================

#[derive(Debug, Clone, SimpleObject)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Variable {
    pub name: String,
    #[graphql(name = "type")]
    pub var_type: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SignalDefinition {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SignalInfo {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SignalConnection {
    pub from_node: String,
    pub signal: String,
    pub to_node: String,
    pub method: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ExternalResource {
    pub id: i32,
    #[graphql(name = "type")]
    pub resource_type: String,
    pub path: String,
}

// ======================
// Node Type Metadata
// ======================

#[derive(Debug, Clone, SimpleObject)]
pub struct NodeTypeInfo {
    pub type_name: String,
    pub properties: Vec<NodePropertyInfo>,
    pub signals: Vec<SignalInfo>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct NodePropertyInfo {
    pub name: String,
    #[graphql(name = "type")]
    pub property_type: String,
    pub hint: Option<String>,
}

// ======================
// Mutations: Inputs/Out
// ======================

#[derive(Debug, Clone, InputObject)]
pub struct AddNodeInput {
    pub parent: String,
    pub name: String,
    #[graphql(name = "type")]
    pub node_type: String,
    pub properties: Option<Vec<PropertyInput>>,
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, InputObject)]
pub struct SetPropertyInput {
    pub node_path: String,
    pub property: String,
    pub value: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct ConnectSignalInput {
    pub from_node: String,
    pub signal: String,
    pub to_node: String,
    pub method: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct DisconnectSignalInput {
    pub from_node: String,
    pub signal: String,
    pub to_node: String,
    pub method: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct OperationResult {
    pub success: bool,
    pub message: Option<String>,
    /// Structured error for AI-friendly error handling
    pub error: Option<GqlStructuredError>,
}

impl OperationResult {
    /// Create a success result
    pub fn ok() -> Self {
        Self {
            success: true,
            message: None,
            error: None,
        }
    }

    /// Create a failure result with structured error
    pub fn err(error: GqlStructuredError) -> Self {
        Self {
            success: false,
            message: Some(error.message.clone()),
            error: Some(error),
        }
    }

    /// Create a failure result with just a message (legacy)
    pub fn err_msg(message: impl Into<String>) -> Self {
        let msg = message.into();
        Self {
            success: false,
            message: Some(msg.clone()),
            error: Some(GqlStructuredError::new(
                "UNKNOWN_ERROR",
                GqlErrorCategory::Godot,
                msg,
            )),
        }
    }
}

/// Result of a transaction operation (begin, commit, rollback)
#[derive(Debug, Clone, SimpleObject)]
pub struct TransactionResult {
    pub success: bool,
    /// Unique identifier for the transaction
    pub transaction_id: Option<String>,
    pub message: Option<String>,
}

impl TransactionResult {
    /// Create a success result with transaction ID
    pub fn ok(transaction_id: impl Into<String>) -> Self {
        Self {
            success: true,
            transaction_id: Some(transaction_id.into()),
            message: None,
        }
    }

    /// Create a success result without transaction ID (for commit/rollback)
    pub fn ok_msg(message: impl Into<String>) -> Self {
        Self {
            success: true,
            transaction_id: None,
            message: Some(message.into()),
        }
    }

    /// Create a failure result
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            transaction_id: None,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct NodeResult {
    pub success: bool,
    pub node: Option<LiveNode>,
    pub message: Option<String>,
    /// Structured error for AI-friendly error handling
    pub error: Option<GqlStructuredError>,
}

impl NodeResult {
    /// Create a success result with node
    pub fn ok(node: LiveNode) -> Self {
        Self {
            success: true,
            node: Some(node),
            message: None,
            error: None,
        }
    }

    /// Create a failure result with structured error
    pub fn err(error: GqlStructuredError) -> Self {
        Self {
            success: false,
            node: None,
            message: Some(error.message.clone()),
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SceneResult {
    pub success: bool,
    pub scene: Option<Scene>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ScriptResult {
    pub success: bool,
    pub script: Option<Script>,
    pub message: Option<String>,
}

// ======================
// File-based inputs
// ======================

#[derive(Debug, Clone, InputObject)]
pub struct CreateSceneInput {
    pub path: String,
    pub root_name: String,
    pub root_type: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct TemplateSceneInput {
    pub template: String,
    pub path: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct CreateScriptInput {
    pub path: String,
    pub extends: String,
    pub class_name: Option<String>,
}

// ======================
// Phase 2.2: Input Action & Project Settings
// ======================

/// Input for adding an input action to the InputMap
#[derive(Debug, Clone, InputObject)]
pub struct AddInputActionInput {
    /// Action name (e.g., "jump", "move_left")
    pub action_name: String,
    /// Events to associate with this action
    pub events: Vec<InputEventInput>,
}

/// Input event definition
#[derive(Debug, Clone, InputObject)]
pub struct InputEventInput {
    /// Event type
    #[graphql(name = "type")]
    pub event_type: InputEventType,
    /// Key name (for KEY type)
    pub key: Option<String>,
    /// Button number (for MOUSE_BUTTON / JOY_BUTTON)
    pub button: Option<i32>,
    /// Device number (for gamepad)
    pub device: Option<i32>,
}

/// Input event type enumeration
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum InputEventType {
    Key,
    MouseButton,
    JoyButton,
    JoyAxis,
}

/// Input for setting a project setting
#[derive(Debug, Clone, InputObject)]
pub struct SetProjectSettingInput {
    /// Setting path (e.g., "application/config/name", "display/window/size/width")
    pub path: String,
    /// Setting value (GDScript format string)
    pub value: String,
    /// Optional type hint
    #[graphql(name = "type")]
    pub value_type: Option<String>,
}

// ======================
// Safe change flow
// ======================

#[derive(Debug, Clone, InputObject)]
pub struct MutationPlanInput {
    pub operations: Vec<PlannedOperation>,
}

#[derive(Debug, Clone, InputObject)]
pub struct PlannedOperation {
    #[graphql(name = "type")]
    pub operation_type: OperationType,
    pub args: async_graphql::Json<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum OperationType {
    AddNode,
    RemoveNode,
    SetProperty,
    SetProperties,
    ConnectSignal,
    DisconnectSignal,
    AddToGroup,
    RemoveFromGroup,
    ReparentNode,
    DuplicateNode,
    CreateScript,
    AttachScript,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct MutationValidationResult {
    pub is_valid: bool,
    pub errors: Vec<MutationValidationError>,
    pub warnings: Vec<MutationValidationWarning>,
    pub validation_time_ms: i32,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct MutationValidationError {
    pub operation_index: i32,
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct MutationValidationWarning {
    pub operation_index: i32,
    pub message: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct PreviewResult {
    pub success: bool,
    pub diff: String,
    pub affected_files: Vec<AffectedFile>,
    pub summary: ChangeSummary,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct AffectedFile {
    pub path: String,
    pub change_type: FileChangeType,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ChangeSummary {
    pub nodes_added: i32,
    pub nodes_removed: i32,
    pub properties_changed: i32,
    pub signals_connected: i32,
}

#[derive(Debug, Clone, InputObject)]
pub struct ApplyMutationInput {
    pub operations: Vec<PlannedOperation>,
    pub create_backup: Option<bool>,
    pub backup_description: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ApplyResult {
    pub success: bool,
    pub applied_count: i32,
    pub backup_path: Option<String>,
    pub errors: Vec<ApplyError>,
    pub undo_action_id: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ApplyError {
    pub operation_index: i32,
    pub message: String,
}

// ======================
// Project validation (static)
// ======================

#[derive(Debug, Clone, SimpleObject)]
pub struct ProjectValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ProjectValidationError>,
    pub warnings: Vec<ProjectValidationWarning>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ProjectValidationError {
    pub file: String,
    pub line: Option<i32>,
    pub message: String,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ProjectValidationWarning {
    pub file: Option<String>,
    pub message: String,
}

// ======================
// gatherContext
// ======================

#[derive(Debug, Clone, InputObject)]
pub struct GatherContextInput {
    pub entry_point: String,
    pub depth: Option<i32>,
    pub include: Option<Vec<FileType>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum FileType {
    Scene,
    Script,
    Resource,
    Shader,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GatheredContext {
    pub entry_point: String,
    pub main: ContextItem,
    pub dependencies: Vec<ContextItem>,
    pub dependents: Vec<ContextItem>,
    pub resources: Vec<ResourceInfo>,
    pub summary: ContextSummary,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ContextItem {
    pub path: String,
    #[graphql(name = "type")]
    pub file_type: FileType,
    pub scene: Option<Scene>,
    pub script: Option<Script>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ResourceInfo {
    pub path: String,
    #[graphql(name = "type")]
    pub resource_type: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ContextSummary {
    pub total_files: i32,
    pub total_functions: i32,
}

// ======================
// dependencyGraph
// ======================

#[derive(Debug, Clone, InputObject)]
pub struct DependencyGraphInput {
    pub directory: Option<String>,
    pub file_types: Option<Vec<FileType>>,
    pub format: Option<GraphFormat>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum GraphFormat {
    Json,
    Graphml,
    Dot,
    Mermaid,
}

#[derive(Debug, Clone, InputObject)]
pub struct GraphNodeFilter {
    pub is_unused: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub stats: GraphStats,
    pub exported_data: Option<String>,
}

#[Object]
impl DependencyGraph {
    async fn nodes(
        &self,
        filter: Option<GraphNodeFilter>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Vec<&GraphNode> {
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.map(|l| l as usize);

        let filtered: Vec<&GraphNode> = self
            .nodes
            .iter()
            .filter(|n| {
                if let Some(ref f) = filter {
                    if let Some(is_unused) = f.is_unused {
                        return n.is_unused == is_unused;
                    }
                }
                true
            })
            .skip(offset)
            .take(limit.unwrap_or(usize::MAX))
            .collect();

        filtered
    }

    async fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    async fn stats(&self) -> &GraphStats {
        &self.stats
    }

    async fn exported_data(&self) -> Option<&str> {
        self.exported_data.as_deref()
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[graphql(name = "type")]
    pub node_type: FileType,
    pub in_degree: i32,
    pub out_degree: i32,
    pub is_unused: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub reference_type: ReferenceType,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum ReferenceType {
    Instantiates,
    AttachesScript,
    UsesResource,
    Preloads,
    Loads,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GraphStats {
    pub node_count: i32,
    pub edge_count: i32,
    pub unused_count: i32,
    pub has_cycles: bool,
    pub cycle_paths: Option<Vec<Vec<String>>>,
}

// ======================
// runTests Types
// ======================

#[derive(Debug, Clone, InputObject)]
pub struct RunTestsInput {
    pub test_path: Option<String>,
    pub retries: Option<i32>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TestExecutionResult {
    pub success: bool,
    pub total_count: i32,
    pub passed_count: i32,
    pub failed_count: i32,
    pub error_count: i32,
    pub skipped_count: i32,
    pub duration_ms: i32,
    pub suites: Vec<TestSuiteResult>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TestSuiteResult {
    pub name: String,
    pub path: String,
    pub success: bool,
    pub passed_count: i32,
    pub failed_count: i32,
    pub skipped_count: i32,
    pub cases: Vec<TestCaseResult>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TestCaseResult {
    pub name: String,
    pub success: bool,
    pub line: Option<i32>,
    pub message: Option<String>,
    pub stack_overflow: Option<bool>,
}

// ======================
// Debugging Types (Phase 2)
// ======================

#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct DebuggerError {
    pub message: String,
    pub stack_info: Vec<StackFrame>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct StackFrame {
    pub file: String,
    pub line: i32,
    pub function: String,
}

#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct LogEntry {
    pub message: String,
    pub severity: String,
    pub timestamp: String,
    pub file: Option<String>,
    pub line: Option<i32>,
}

#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize)]
pub struct GodotObject {
    pub id: String,
    pub class: String,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, InputObject)]
pub struct BreakpointInput {
    pub path: String,
    pub line: i32,
    #[graphql(default = true)]
    pub enabled: bool,
}

// ======================
// Phase 3: Debug Enhanced Types
// ======================

/// Parse error from GDScript compilation
#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize, Default)]
pub struct ParseError {
    pub line: i32,
    pub column: i32,
    pub message: String,
    pub severity: ErrorSeverity,
}

/// Error severity level
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum, Serialize, Deserialize, Default)]
pub enum ErrorSeverity {
    #[default]
    Error,
    Warning,
}

/// Stack variable during debugging
#[derive(Debug, Clone, SimpleObject, Serialize, Deserialize, Default)]
pub struct StackVariable {
    pub name: String,
    pub value: String,
    #[graphql(name = "type")]
    #[serde(rename = "type")]
    pub var_type: String,
}

// ======================
// Phase 3: Code Understanding Types
// ======================

/// Class hierarchy information
#[derive(Debug, Clone, SimpleObject)]
pub struct ClassHierarchy {
    pub script_path: String,
    pub class_name: Option<String>,
    pub extends_chain: Vec<ClassInfo>,
    pub depth: i32,
}

/// Information about a class in the hierarchy
#[derive(Debug, Clone, SimpleObject)]
pub struct ClassInfo {
    pub name: String,
    pub script_path: Option<String>,
    pub is_builtin: bool,
}

/// Symbol reference search result
#[derive(Debug, Clone, SimpleObject)]
pub struct SymbolReferences {
    pub symbol: String,
    pub definition: Option<SymbolLocation>,
    pub references: Vec<SymbolLocation>,
    pub total_count: i32,
}

/// Location of a symbol
#[derive(Debug, Clone, SimpleObject)]
pub struct SymbolLocation {
    pub file: String,
    pub line: i32,
    pub column: Option<i32>,
    pub context: Option<String>,
}

/// Autoload entry
#[derive(Debug, Clone, SimpleObject)]
pub struct AutoloadEntry {
    pub name: String,
    pub path: String,
    pub is_singleton: bool,
}

/// Autoloads list result
#[derive(Debug, Clone, SimpleObject)]
pub struct AutoloadsResult {
    pub autoloads: Vec<AutoloadEntry>,
    pub count: i32,
}

// ======================
// Phase 3: Refactoring Types
// ======================

/// Rename symbol input
#[derive(Debug, Clone, InputObject)]
pub struct RenameSymbolInput {
    pub symbol: String,
    pub new_name: String,
    pub scope: Option<String>,
}

/// Rename symbol result
#[derive(Debug, Clone, SimpleObject)]
pub struct RenameSymbolResult {
    pub success: bool,
    pub old_name: String,
    pub new_name: String,
    pub files_changed: Vec<FileChange>,
    pub occurrences_replaced: i32,
    pub message: Option<String>,
}

/// File change detail
#[derive(Debug, Clone, SimpleObject)]
pub struct FileChange {
    pub path: String,
    pub changes_count: i32,
}

/// Extract function input
#[derive(Debug, Clone, InputObject)]
pub struct ExtractFunctionInput {
    pub script_path: String,
    pub start_line: i32,
    pub end_line: i32,
    pub function_name: String,
    pub parameters: Option<Vec<String>>,
}

/// Extract function result
#[derive(Debug, Clone, SimpleObject)]
pub struct ExtractFunctionResult {
    pub success: bool,
    pub function_name: String,
    pub script_path: String,
    pub message: Option<String>,
}

/// Move node to scene input
#[derive(Debug, Clone, InputObject)]
pub struct MoveNodeToSceneInput {
    pub node_path: String,
    pub new_scene_path: String,
    pub keep_instance: Option<bool>,
}

/// Move node to scene result
#[derive(Debug, Clone, SimpleObject)]
pub struct MoveNodeToSceneResult {
    pub success: bool,
    pub new_scene_path: String,
    pub instance_path: Option<String>,
    pub message: Option<String>,
}

// ======================
// Phase 3: Code Generation Types
// ======================

/// Generate input handler input
#[derive(Debug, Clone, InputObject)]
pub struct GenerateInputHandlerInput {
    pub script_path: String,
    pub actions: Vec<String>,
    pub handler_type: Option<InputHandlerType>,
}

/// Input handler type
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum InputHandlerType {
    Process,
    PhysicsProcess,
    UnhandledInput,
    Input,
}

/// Generate state machine input
#[derive(Debug, Clone, InputObject)]
pub struct GenerateStateMachineInput {
    pub script_path: String,
    pub states: Vec<String>,
    pub initial_state: Option<String>,
    pub use_enum: Option<bool>,
}

/// Generate test script input
#[derive(Debug, Clone, InputObject)]
pub struct GenerateTestScriptInput {
    pub target_script: String,
    pub output_path: Option<String>,
    pub test_framework: Option<TestFramework>,
}

/// Test framework
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum TestFramework {
    GdUnit4,
    Gut,
    Custom,
}

/// Code generation result
#[derive(Debug, Clone, SimpleObject)]
pub struct CodeGenerationResult {
    pub success: bool,
    pub path: String,
    pub message: Option<String>,
}

// ======================
// Phase 3: Shader Types
// ======================

/// Validate shader input
#[derive(Debug, Clone, InputObject)]
pub struct ValidateShaderInput {
    pub shader_code: String,
    pub shader_type: Option<ShaderType>,
}

/// Shader type
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum ShaderType {
    Spatial,
    CanvasItem,
    Particles,
    Sky,
    Fog,
}

/// Shader validation result
#[derive(Debug, Clone, SimpleObject)]
pub struct ShaderValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ShaderError>,
    pub warnings: Vec<ShaderWarning>,
}

/// Shader error
#[derive(Debug, Clone, SimpleObject)]
pub struct ShaderError {
    pub line: Option<i32>,
    pub column: Option<i32>,
    pub message: String,
}

/// Shader warning
#[derive(Debug, Clone, SimpleObject)]
pub struct ShaderWarning {
    pub line: Option<i32>,
    pub message: String,
}

/// Visual shader node input
#[derive(Debug, Clone, InputObject)]
pub struct CreateVisualShaderNodeInput {
    pub shader_path: String,
    pub node_type: String,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}
