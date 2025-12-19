//! MCP Tool Definitions - Godot MCP Server

mod editor;
mod project;
mod resource;
mod scene;
mod script;

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

/// List Files Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListFilesRequest {
    /// Subdirectory path (optional)
    pub path: Option<String>,
}

/// Read File Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadFileRequest {
    /// File path
    pub path: String,
}

/// Create Scene Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateSceneRequest {
    /// Scene file path
    pub path: String,
    /// Root node type (e.g. Node3D, CharacterBody3D)
    pub root_type: String,
    /// Root node name (generated from filename if omitted)
    pub root_name: Option<String>,
}

/// Read Scene Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadSceneRequest {
    /// Scene file path
    pub path: String,
}

/// Add Node Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AddNodeRequest {
    /// Scene file path
    pub scene_path: String,
    /// Parent node path ("." for root)
    pub parent: String,
    /// Node name
    pub name: String,
    /// Node type (e.g. Node3D, MeshInstance3D)
    pub node_type: String,
}

/// Remove Node Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RemoveNodeRequest {
    /// Scene file path
    pub scene_path: String,
    /// Path of the node to remove
    pub node_path: String,
}

/// Set Property Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetNodePropertyRequest {
    /// Scene file path
    pub scene_path: String,
    /// Node path
    pub node_path: String,
    /// Property name
    pub property: String,
    /// Property value (GDScript format string)
    pub value: String,
}

/// Get Node Tree Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetNodeTreeRequest {
    /// Scene file path
    pub path: String,
}

/// Create Script Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateScriptRequest {
    /// Script file path
    pub path: String,
    /// Class to inherit (CharacterBody3D, Node3D, etc.)
    pub extends: String,
    /// Initial content (uses template if omitted)
    pub content: Option<String>,
}

/// Attach Script Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AttachScriptRequest {
    /// Scene file path
    pub scene_path: String,
    /// Node path ("." for root)
    pub node_path: String,
    /// Script file path
    pub script_path: String,
}

/// Validate Scene Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ValidateTscnRequest {
    /// Scene file path
    pub path: String,
}

/// Copy Scene Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CopySceneRequest {
    /// Source scene path
    pub source: String,
    /// Destination scene path
    pub destination: String,
}

/// Get Scene Metadata Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetSceneMetadataRequest {
    /// Scene file path
    pub path: String,
}

/// List All Scenes Request (no params)
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct ListAllScenesRequest {}

/// Compare Scenes Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CompareScenesRequest {
    /// Source scene path
    pub path_a: String,
    /// Target scene path
    pub path_b: String,
}

/// Export Scene as JSON Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExportSceneAsJsonRequest {
    /// Scene file path
    pub path: String,
}

/// Batch Add Nodes Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BatchAddNodesRequest {
    /// Scene file path
    pub scene_path: String,
    /// List of nodes to add
    pub nodes: Vec<BatchNodeEntry>,
}

/// Batch Node Entry
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

/// Search in Project Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchInProjectRequest {
    /// Search type: "node_type", "resource", "script"
    pub search_type: String,
    /// Search query
    pub query: String,
}

/// Read Script Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadScriptRequest {
    /// Script file path
    pub path: String,
}

/// Add Function Request
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

/// Function Parameter Input
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct FunctionParamInput {
    pub name: String,
    pub param_type: Option<String>,
}

/// Add Export Variable Request
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

/// Analyze Script Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AnalyzeScriptRequest {
    /// Script file path
    pub path: String,
}

/// List Resources Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct ListResourcesRequest {
    /// Resource type to filter (optional)
    pub filter_type: Option<String>,
}

/// Read Resource Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadResourceRequest {
    /// Resource file path
    pub path: String,
}

/// Get Node Type Info Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetNodeTypeInfoRequest {
    /// Node type name (e.g. "CharacterBody3D", "Camera3D")
    pub node_type: String,
}

/// Create Scene from Template Request
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateSceneFromTemplateRequest {
    /// Path of the scene to create
    pub path: String,
    /// Template name: "player_3d", "player_2d", "enemy_3d", "level_3d", "ui_menu"
    pub template: String,
    /// Root node name (optional, defaults to filename)
    pub root_name: Option<String>,
}

/// Get Project Stats Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetProjectStatsRequest {}

/// Validate Project Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct ValidateProjectRequest {}

/// Get Godot Version Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetGodotVersionRequest {}

/// Run Project Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct RunProjectRequest {
    /// Scene to run (optional, defaults to main scene)
    pub scene: Option<String>,
}

/// Stop Project Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct StopProjectRequest {}

/// Get Debug Output Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetDebugOutputRequest {
    /// Maximum number of lines to retrieve (optional, default 100)
    pub lines: Option<u32>,
}

/// Launch Editor Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct LaunchEditorRequest {
    /// Scene to open (optional)
    pub scene: Option<String>,
}

/// Get Running Status Request
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GetRunningStatusRequest {}

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
                // Project tools
                Tool::new(
                    "list_project_files",
                    "List files in the Godot project directory",
                    schema_to_json_object::<ListFilesRequest>(),
                ),
                Tool::new(
                    "read_file",
                    "Read a file from the Godot project",
                    schema_to_json_object::<ReadFileRequest>(),
                ),
                Tool::new(
                    "list_all_scenes",
                    "List all scene files (.tscn) in the project recursively",
                    schema_to_json_object::<ListAllScenesRequest>(),
                ),
                Tool::new(
                    "search_in_project",
                    "Search for node types, resources, or scripts across all scenes",
                    schema_to_json_object::<SearchInProjectRequest>(),
                ),
                // Scene tools
                Tool::new(
                    "create_scene",
                    "Create a new Godot scene file (.tscn)",
                    schema_to_json_object::<CreateSceneRequest>(),
                ),
                Tool::new(
                    "read_scene",
                    "Read and parse a Godot scene file, returning structured JSON",
                    schema_to_json_object::<ReadSceneRequest>(),
                ),
                Tool::new(
                    "add_node",
                    "Add a new node to an existing Godot scene",
                    schema_to_json_object::<AddNodeRequest>(),
                ),
                Tool::new(
                    "remove_node",
                    "Remove a node from a Godot scene",
                    schema_to_json_object::<RemoveNodeRequest>(),
                ),
                Tool::new(
                    "set_node_property",
                    "Set a property on a node in a Godot scene",
                    schema_to_json_object::<SetNodePropertyRequest>(),
                ),
                Tool::new(
                    "get_node_tree",
                    "Get the node tree structure of a Godot scene",
                    schema_to_json_object::<GetNodeTreeRequest>(),
                ),
                Tool::new(
                    "validate_tscn",
                    "Validate a Godot scene file and check for issues",
                    schema_to_json_object::<ValidateTscnRequest>(),
                ),
                Tool::new(
                    "copy_scene",
                    "Copy a scene file to a new location",
                    schema_to_json_object::<CopySceneRequest>(),
                ),
                Tool::new(
                    "get_scene_metadata",
                    "Get metadata about a Godot scene (node types, resource counts, etc.)",
                    schema_to_json_object::<GetSceneMetadataRequest>(),
                ),
                Tool::new(
                    "compare_scenes",
                    "Compare two scene files and show differences (added/removed nodes, modified properties)",
                    schema_to_json_object::<CompareScenesRequest>(),
                ),
                Tool::new(
                    "export_scene_as_json",
                    "Export a Godot scene as hierarchical JSON with nested children",
                    schema_to_json_object::<ExportSceneAsJsonRequest>(),
                ),
                Tool::new(
                    "batch_add_nodes",
                    "Add multiple nodes to a scene in a single operation",
                    schema_to_json_object::<BatchAddNodesRequest>(),
                ),
                // Script tools
                Tool::new(
                    "create_script",
                    "Create a new GDScript file with optional template",
                    schema_to_json_object::<CreateScriptRequest>(),
                ),
                Tool::new(
                    "attach_script",
                    "Attach a script to a node in a Godot scene",
                    schema_to_json_object::<AttachScriptRequest>(),
                ),
                Tool::new(
                    "read_script",
                    "Read and parse a GDScript file, returning its structure as JSON",
                    schema_to_json_object::<ReadScriptRequest>(),
                ),
                Tool::new(
                    "add_function",
                    "Add a function to an existing GDScript file",
                    schema_to_json_object::<AddFunctionRequest>(),
                ),
                Tool::new(
                    "add_export_var",
                    "Add an @export variable to an existing GDScript file",
                    schema_to_json_object::<AddExportVarRequest>(),
                ),
                Tool::new(
                    "analyze_script",
                    "Analyze a GDScript file and return summary information",
                    schema_to_json_object::<AnalyzeScriptRequest>(),
                ),
                // Resource tools
                Tool::new(
                    "list_resources",
                    "List all .tres resource files in the project",
                    schema_to_json_object::<ListResourcesRequest>(),
                ),
                Tool::new(
                    "read_resource",
                    "Read and parse a .tres resource file",
                    schema_to_json_object::<ReadResourceRequest>(),
                ),
                Tool::new(
                    "get_node_type_info",
                    "Get detailed information about a Godot node type (properties, methods, common children)",
                    schema_to_json_object::<GetNodeTypeInfoRequest>(),
                ),
                Tool::new(
                    "create_scene_from_template",
                    "Create a scene from a predefined template (player_3d, player_2d, enemy_3d, level_3d, ui_menu)",
                    schema_to_json_object::<CreateSceneFromTemplateRequest>(),
                ),
                Tool::new(
                    "get_project_stats",
                    "Get project statistics (file counts, node counts by type)",
                    schema_to_json_object::<GetProjectStatsRequest>(),
                ),
                Tool::new(
                    "validate_project",
                    "Validate the entire project (detect parse errors, empty scenes, unused scripts)",
                    schema_to_json_object::<ValidateProjectRequest>(),
                ),
                // Editor tools
                Tool::new(
                    "get_godot_version",
                    "Get the installed Godot version and executable path",
                    schema_to_json_object::<GetGodotVersionRequest>(),
                ),
                Tool::new(
                    "run_project",
                    "Run the Godot project in debug mode",
                    schema_to_json_object::<RunProjectRequest>(),
                ),
                Tool::new(
                    "stop_project",
                    "Stop the running Godot project",
                    schema_to_json_object::<StopProjectRequest>(),
                ),
                Tool::new(
                    "get_debug_output",
                    "Get the debug console output from the running project",
                    schema_to_json_object::<GetDebugOutputRequest>(),
                ),
                Tool::new(
                    "launch_editor",
                    "Launch the Godot editor for this project",
                    schema_to_json_object::<LaunchEditorRequest>(),
                ),
                Tool::new(
                    "get_running_status",
                    "Check if a Godot project is currently running",
                    schema_to_json_object::<GetRunningStatusRequest>(),
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
                // Project tools
                "list_project_files" => self.handle_list_project_files(request.arguments).await,
                "read_file" => self.handle_read_file(request.arguments).await,
                "list_all_scenes" => self.handle_list_all_scenes(request.arguments).await,
                "search_in_project" => self.handle_search_in_project(request.arguments).await,
                "get_node_type_info" => self.handle_get_node_type_info(request.arguments).await,
                // Scene tools
                "create_scene" => self.handle_create_scene(request.arguments).await,
                "read_scene" => self.handle_read_scene(request.arguments).await,
                "add_node" => self.handle_add_node(request.arguments).await,
                "remove_node" => self.handle_remove_node(request.arguments).await,
                "set_node_property" => self.handle_set_node_property(request.arguments).await,
                "get_node_tree" => self.handle_get_node_tree(request.arguments).await,
                "validate_tscn" => self.handle_validate_tscn(request.arguments).await,
                "copy_scene" => self.handle_copy_scene(request.arguments).await,
                "get_scene_metadata" => self.handle_get_scene_metadata(request.arguments).await,
                "compare_scenes" => self.handle_compare_scenes(request.arguments).await,
                "export_scene_as_json" => self.handle_export_scene_as_json(request.arguments).await,
                "batch_add_nodes" => self.handle_batch_add_nodes(request.arguments).await,
                // Script tools
                "create_script" => self.handle_create_script(request.arguments).await,
                "attach_script" => self.handle_attach_script(request.arguments).await,
                "read_script" => self.handle_read_script(request.arguments).await,
                "add_function" => self.handle_add_function(request.arguments).await,
                "add_export_var" => self.handle_add_export_var(request.arguments).await,
                "analyze_script" => self.handle_analyze_script(request.arguments).await,
                // Resource tools
                "list_resources" => self.handle_list_resources(request.arguments).await,
                "read_resource" => self.handle_read_resource(request.arguments).await,
                "create_scene_from_template" => {
                    self.handle_create_scene_from_template(request.arguments)
                        .await
                }
                "get_project_stats" => self.handle_get_project_stats(request.arguments).await,
                "validate_project" => self.handle_validate_project(request.arguments).await,
                // Editor tools
                "get_godot_version" => self.handle_get_godot_version(request.arguments).await,
                "run_project" => self.handle_run_project(request.arguments).await,
                "stop_project" => self.handle_stop_project(request.arguments).await,
                "get_debug_output" => self.handle_get_debug_output(request.arguments).await,
                "launch_editor" => self.handle_launch_editor(request.arguments).await,
                "get_running_status" => self.handle_get_running_status(request.arguments).await,
                _ => Err(McpError::invalid_request(
                    format!("Unknown tool: {}", request.name),
                    None,
                )),
            }
        }
    }
}
