//! GraphQL type definitions
//!
//! These types correspond to `docs/gql/schema.graphql`.
//! Keep in sync with the SDL.

use async_graphql::{Enum, InputObject, Object, SimpleObject};
use serde::{Deserialize, Serialize};

// ======================
// Scalar types
// ======================
// JSON scalar is handled by async-graphql's Json<T>

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

#[derive(Debug, Clone, SimpleObject)]
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
}

#[derive(Debug, Clone, SimpleObject)]
pub struct NodeResult {
    pub success: bool,
    pub node: Option<LiveNode>,
    pub message: Option<String>,
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
