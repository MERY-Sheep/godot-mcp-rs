//! MCP Tool Definitions - Godot MCP Server

mod editor;
pub mod gql_tools;
mod live;
mod project;
mod resource;
mod scene;
mod script;

use gql_tools::{GqlIntrospectRequest, GqlMutateRequest, GqlQueryRequest};

use rmcp::{
    model::{CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam, Tool},
    service::{RequestContext, RoleServer},
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::path::{Path, PathBuf};

// ============================================================
// Request Types
// ============================================================

/// Request to list files
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListFilesRequest {
    /// Subdirectory path (optional)
    pub path: Option<String>,
}

/// Request to read a file
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadFileRequest {
    /// File path
    pub path: String,
}

/// Request to create a scene
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateSceneRequest {
    /// Scene file path
    pub path: String,
    /// Root node type (e.g., Node3D, CharacterBody3D)
    pub root_type: String,
    /// Root node name (generated from filename if omitted)
    pub root_name: Option<String>,
}

/// Request to read a scene
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadSceneRequest {
    /// Scene file path
    pub path: String,
}

/// Request to add a node
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AddNodeRequest {
    /// Scene file path
    pub scene_path: String,
    /// Parent node path ("." for root)
    pub parent: String,
    /// Node name
    pub name: String,
    /// Node type (e.g., Node3D, MeshInstance3D)
    pub node_type: String,
}

/// Request to remove a node
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RemoveNodeRequest {
    /// Scene file path
    pub scene_path: String,
    /// Path of the node to remove
    pub node_path: String,
}

/// Request to set a node property
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetNodePropertyRequest {
    /// Scene file path
    pub scene_path: String,
    /// Node path
    pub node_path: String,
    /// Property name
    pub property: String,
    /// Property value (GDScript formatted string)
    pub value: String,
}

/// Request to get the node tree
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetNodeTreeRequest {
    /// Scene file path
    pub path: String,
}

/// Request to create a script
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateScriptRequest {
    /// Script file path
    pub path: String,
    /// Base class to extend (CharacterBody3D, Node3D, etc.)
    pub extends: String,
    /// Initial content (uses template if omitted)
    pub content: Option<String>,
}

/// Request to attach a script
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AttachScriptRequest {
    /// Scene file path
    pub scene_path: String,
    /// Node path ("." for root)
    pub node_path: String,
    /// Script file path
    pub script_path: String,
}

/// Request to validate a scene
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ValidateTscnRequest {
    /// Scene file path
    pub path: String,
}

/// Request to copy a scene
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CopySceneRequest {
    /// Source scene path
    pub source: String,
    /// Destination scene path
    pub destination: String,
}

/// Request to get scene metadata
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetSceneMetadataRequest {
    /// Scene file path
    pub path: String,
}

/// Request to list all scenes (no parameters)
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct ListAllScenesRequest {}

/// Request to compare scenes
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CompareScenesRequest {
    /// Source scene path
    pub path_a: String,
    /// Comparison target scene path
    pub path_b: String,
}

/// Request to export scene as JSON
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExportSceneAsJsonRequest {
    /// Scene file path
    pub path: String,
}

/// Request for batch node addition
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BatchAddNodesRequest {
    /// Scene file path
    pub scene_path: String,
    /// List of nodes to add
    pub nodes: Vec<BatchNodeEntry>,
}

/// Batch node entry
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BatchNodeEntry {
    /// Parent node path ("." for root)
    pub parent: String,
    /// Node name
    pub name: String,
    /// Node type
    #[serde(rename = "type")]
    pub node_type: String,
}

/// Request to search in project
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchInProjectRequest {
    /// Search type: "node_type", "resource", "script"
    pub search_type: String,
    /// Search query
    pub query: String,
}

/// Request to read a script
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadScriptRequest {
    /// Script file path
    pub path: String,
}

/// Request to add a function
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AddFunctionRequest {
    /// Script file path
    pub path: String,
    /// Function name
    pub name: String,
    /// Parameters (optional)
    pub params: Option<Vec<FunctionParamInput>>,
    /// Return type (optional)
    pub return_type: Option<String>,
    /// Function body
    pub body: Option<String>,
}

/// Function parameter input
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct FunctionParamInput {
    pub name: String,
    pub param_type: Option<String>,
}

/// Request to add an export variable
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AddExportVarRequest {
    /// Script file path
    pub path: String,
    /// Variable name
    pub name: String,
    /// Type (optional)
    pub var_type: Option<String>,
    /// Default value (optional)
    pub default_value: Option<String>,
}

/// Request to analyze a script
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AnalyzeScriptRequest {
    /// Script file path
    pub path: String,
}

/// Request to list resources
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct ListResourcesRequest {
    /// Resource type to filter (optional)
    pub filter_type: Option<String>,
}

/// Request to read a resource
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadResourceRequest {
    /// Resource file path
    pub path: String,
}

/// Request to create a resource
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateResourceRequest {
    /// Resource file path (e.g., "resources/my_resource.tres")
    pub path: String,
    /// Resource type (e.g., "Resource", "StandardMaterial3D")
    pub resource_type: String,
}

/// Request to set a resource property
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetResourcePropertyRequest {
    /// Resource file path
    pub path: String,
    /// Property name
    pub property: String,
    /// Property value (Godot formatted string, e.g., 'Color(1, 0, 0, 1)', '0.5', '"text"')
    pub value: String,
}

/// Request to add an external resource reference
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AddExtResourceRequest {
    /// Resource file path
    pub path: String,
    /// External resource ID (e.g., "1", "2")
    pub id: String,
    /// External resource type (e.g., "Script", "Texture2D")
    pub resource_type: String,
    /// Path to the external resource (e.g., "res://scripts/my_script.gd")
    pub resource_path: String,
}

/// Request to add a sub-resource
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AddSubResourceRequest {
    /// Resource file path
    pub path: String,
    /// Sub-resource ID (e.g., "1", "2")
    pub id: String,
    /// Sub-resource type (e.g., "StyleBoxFlat", "Gradient")
    pub resource_type: String,
    /// Initial properties (optional)
    pub properties: Option<Vec<ResourcePropertyEntry>>,
}

/// Resource property entry for batch operations
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ResourcePropertyEntry {
    /// Property name
    pub name: String,
    /// Property value (Godot formatted string)
    pub value: String,
}

/// Request to create a material
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateMaterialRequest {
    /// Material file path (e.g., "materials/metal.tres")
    pub path: String,
    /// Material name (optional)
    pub name: Option<String>,
    /// Albedo color as RGBA array [r, g, b, a] (0.0-1.0)
    pub albedo_color: Option<[f32; 4]>,
    /// Metallic value (0.0-1.0)
    pub metallic: Option<f32>,
    /// Roughness value (0.0-1.0)
    pub roughness: Option<f32>,
}

/// Request to set a material property
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetMaterialPropertyRequest {
    /// Material file path
    pub path: String,
    /// Property name (e.g., "albedo_color", "metallic", "roughness", "emission")
    pub property: String,
    /// Property value (Godot formatted string)
    pub value: String,
}

/// Request to assign a material to a node
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AssignMaterialRequest {
    /// Scene file path
    pub scene_path: String,
    /// Node path (must be a MeshInstance3D or similar)
    pub node_path: String,
    /// Material resource path (e.g., "res://materials/metal.tres")
    pub material_path: String,
    /// Surface index (default 0)
    pub surface_index: Option<u32>,
}

/// Request for node type information
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetNodeTypeInfoRequest {
    /// Node type name (e.g., "CharacterBody3D", "Camera3D")
    pub node_type: String,
}

/// Request to create a scene from a template
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateSceneFromTemplateRequest {
    /// Path for the new scene
    pub path: String,
    /// Template name: "player_3d", "player_2d", "enemy_3d", "level_3d", "ui_menu"
    pub template: String,
    /// Root node name (optional, default is filename)
    pub root_name: Option<String>,
}

/// Request for project statistics
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetProjectStatsRequest {}

/// Request for project validation
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct ValidateProjectRequest {}

/// Request to get Godot version
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetGodotVersionRequest {}

/// Request to run the project
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct RunProjectRequest {
    /// Scene to run (optional, default is main scene)
    pub scene: Option<String>,
}

/// Request to stop the project
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct StopProjectRequest {}

/// Request to get debug output
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetDebugOutputRequest {
    /// Max number of lines to retrieve (optional, default 100)
    pub lines: Option<u32>,
}

/// Request to launch the editor
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LaunchEditorRequest {
    /// Scene to open (optional)
    pub scene: Option<String>,
}

/// Request for running status check
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetRunningStatusRequest {}

// --- Live Commands ---

/// Base for live commands
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LivePingRequest {
    /// Port of the Godot plugin (default 6060)
    pub port: Option<u16>,
}

/// Request to add a node via live editor
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveAddNodeRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Parent node path ("." for root)
    pub parent: String,
    /// New node name
    pub name: String,
    /// Node type
    pub node_type: String,
}

/// Request to remove a node via live editor
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveRemoveNodeRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Path of the node to remove
    pub node_path: String,
}

/// Request to set a property via live editor
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveSetPropertyRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Node path
    pub node_path: String,
    /// Property name
    pub property: String,
    /// Property value (GDScript literal string or JSON)
    pub value: String,
}

/// Request to get the live tree structure
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LiveGetTreeRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
}

/// Request to save the current scene in editor
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LiveSaveSceneRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
}

/// Request to open a scene in editor
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveOpenSceneRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Scene path (res://...)
    pub scene_path: String,
}

/// Request to connect a signal
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveConnectSignalRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Source node path
    pub source: String,
    /// Signal name
    pub signal: String,
    /// Target node path
    pub target: String,
    /// Target method name
    pub method: String,
}

/// Request to disconnect a signal
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveDisconnectSignalRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Source node path
    pub source: String,
    /// Signal name
    pub signal: String,
    /// Target node path
    pub target: String,
    /// Target method name
    pub method: String,
}

/// Request to list signals
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveListSignalsRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Node path
    pub node_path: String,
}

/// Request to create an animation
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveCreateAnimationRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// AnimationPlayer node path
    pub player: String,
    /// Animation name
    pub name: String,
    /// Animation length in seconds
    pub length: f32,
}

/// Request to add an animation track
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveAddAnimationTrackRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// AnimationPlayer node path
    pub player: String,
    /// Animation name
    pub animation: String,
    /// Track path (e.g. "NodePath:property")
    pub track_path: String,
    /// Track type (default "value")
    pub track_type: Option<String>,
}

/// Request to add an animation keyframe
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveAddAnimationKeyRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// AnimationPlayer node path
    pub player: String,
    /// Animation name
    pub animation: String,
    /// Track index
    pub track: u32,
    /// Time in seconds
    pub time: f32,
    /// Keyframe value
    pub value: String,
}

/// Request to play an animation
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LivePlayAnimationRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// AnimationPlayer node path
    pub player: String,
    /// Animation name
    pub animation: String,
}

/// Request to stop an animation
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveStopAnimationRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// AnimationPlayer node path
    pub player: String,
}

/// Request to list animations
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveListAnimationsRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// AnimationPlayer node path
    pub player: String,
}

/// Request to get the editor log
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LiveGetEditorLogRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Number of lines to retrieve
    pub lines: Option<u32>,
}

/// Request to clear the editor log
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LiveClearEditorLogRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
}

/// Request to reload the plugin
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LiveReloadPluginRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
}

/// Request to add a node to a group
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveAddToGroupRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Node path
    pub node_path: String,
    /// Group name
    pub group: String,
}

/// Request to remove a node from a group
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveRemoveFromGroupRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Node path
    pub node_path: String,
    /// Group name
    pub group: String,
}

/// Request to list groups of a node
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveListGroupsRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Node path
    pub node_path: String,
}

/// Request to get nodes in a group
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveGetGroupNodesRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Group name
    pub group: String,
}

/// Request to instantiate a scene via live editor
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LiveInstantiateSceneRequest {
    /// Port (default 6060)
    pub port: Option<u16>,
    /// Scene path (res://...)
    pub scene_path: String,
    /// Parent node path ("." for root)
    pub parent: String,
    /// New node name (optional)
    pub name: Option<String>,
    /// X position (optional)
    pub x: Option<f32>,
    /// Y position (optional)
    pub y: Option<f32>,
    /// Z position (optional)
    pub z: Option<f32>,
}

// ============================================================
// GodotTools
// ============================================================

/// Godot MCP Server
#[derive(Clone, Default)]
pub struct GodotTools {
    pub project_root: Option<PathBuf>,
    pub godot_path: Option<PathBuf>,
}

fn schema_to_json_object<T: JsonSchema>() -> serde_json::Map<String, serde_json::Value> {
    let schema = schemars::schema_for!(T);
    match serde_json::to_value(schema).unwrap_or_default() {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    }
}

impl GodotTools {
    pub fn new() -> Self {
        Self {
            project_root: None,
            godot_path: None,
        }
    }

    /// Create with a specific project root
    pub fn with_project(project_root: std::path::PathBuf) -> Self {
        Self {
            project_root: Some(project_root),
            godot_path: None,
        }
    }

    pub(crate) fn get_base_path(&self) -> &Path {
        self.project_root
            .as_ref()
            .map(|p| p.as_path())
            .unwrap_or(Path::new("."))
    }
}

// ============================================================
// ServerHandler Implementation
// ============================================================

impl ServerHandler for GodotTools {
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            let tools = vec![
                // === GQL Tools (GraphQL API) ===
                Tool::new(
                    "godot_query",
                    "Execute a GraphQL query against the Godot project. Use this to read project structure, scenes, scripts, and dependencies. Use godot_introspect to discover available queries.",
                    schema_to_json_object::<GqlQueryRequest>(),
                ),
                Tool::new(
                    "godot_mutate",
                    "Execute a GraphQL mutation to modify the Godot project. Use this for adding nodes, setting properties, creating files, and other changes. Use godot_introspect to discover available mutations.",
                    schema_to_json_object::<GqlMutateRequest>(),
                ),
                Tool::new(
                    "godot_introspect",
                    "Get the GraphQL schema (SDL or introspection). Use this to discover available queries and mutations before using godot_query or godot_mutate.",
                    schema_to_json_object::<GqlIntrospectRequest>(),
                ),
            ];

            Ok(ListToolsResult {
                tools,
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            match request.name.as_ref() {
                // GraphQL Tools
                "godot_query" => {
                    gql_tools::handle_godot_query(self.get_base_path(), request.arguments).await
                }
                "godot_mutate" => {
                    gql_tools::handle_godot_mutate(self.get_base_path(), request.arguments).await
                }
                "godot_introspect" => {
                    gql_tools::handle_godot_introspect(self.get_base_path(), request.arguments)
                        .await
                }

                // Note: Legacy tools are still implemented in GodotTools for CLI usage,
                // but are no longer exposed as MCP tools.
                _ => Err(McpError::invalid_request(
                    format!("Unknown tool: {}", request.name),
                    None,
                )),
            }
        }
    }
}
