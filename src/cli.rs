//! CLI Mode - Direct operation from terminal or scripts.

use crate::godot::commands::{
    AddAnimationKeyParams, AddAnimationTrackParams, AddNodeParams, CreateAnimationParams,
    GetLogParams, GodotCommand, GroupNameParams, GroupParams, InstantiateSceneParams,
    NodePathParams, OpenSceneParams, PlayAnimationParams, Position3D, RemoveNodeParams,
    SetPropertyParams, SignalParams, StopAnimationParams,
};
use crate::tools::GodotTools;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Godot MCP Server CLI
#[derive(Parser, Debug)]
#[command(name = "godot-mcp-rs")]
#[command(about = "MCP server for Godot - CLI mode for direct tool execution")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start MCP server (default mode)
    Serve,

    /// Execute a tool directly via CLI
    #[command(subcommand)]
    Tool(ToolCommands),
}

/// Available tools
#[derive(Subcommand, Debug)]
pub enum ToolCommands {
    // === Project Tools ===
    /// List all scenes in the project
    ListAllScenes {
        #[arg(short, long)]
        project: PathBuf,
    },

    /// List files in a directory
    ListProjectFiles {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: Option<String>,
    },

    /// Read a file's content
    ReadFile {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
    },

    /// Search in project
    SearchInProject {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "node_type")]
        search_type: String,
    },

    /// Get node type information
    GetNodeTypeInfo {
        #[arg(long)]
        node_type: String,
    },

    /// Get project statistics
    GetProjectStats {
        #[arg(short, long)]
        project: PathBuf,
    },

    /// Validate the project
    ValidateProject {
        #[arg(short, long)]
        project: PathBuf,
    },

    /// Read Godot's log file for a project
    ReadGodotLog {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long, default_value = "50")]
        lines: usize,
    },

    // === Scene Tools ===
    /// Create a new scene
    CreateScene {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        root_type: String,
        #[arg(long)]
        root_name: Option<String>,
    },

    /// Create scene from template
    CreateSceneFromTemplate {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        template: String,
        #[arg(long)]
        root_name: Option<String>,
    },

    /// Read a scene
    ReadScene {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
    },

    /// Add a node to a scene
    AddNode {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: String,
        #[arg(long)]
        parent: String,
        #[arg(long)]
        name: String,
        #[arg(long, name = "type")]
        node_type: String,
    },

    /// Remove a node from a scene
    RemoveNode {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: String,
        #[arg(long)]
        node_path: String,
    },

    /// Set a node property
    SetNodeProperty {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: String,
        #[arg(long)]
        node_path: String,
        #[arg(long)]
        property: String,
        #[arg(long)]
        value: String,
    },

    /// Get node tree
    GetNodeTree {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: String,
    },

    /// Validate a scene file
    ValidateTscn {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
    },

    /// Compare two scenes
    CompareScenes {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene_a: String,
        #[arg(long)]
        scene_b: String,
    },

    /// Export scene as JSON
    ExportSceneAsJson {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
    },

    // === Script Tools ===
    /// Create a script
    CreateScript {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        extends: String,
        #[arg(long)]
        class_name: Option<String>,
    },

    /// Read and analyze a script
    ReadScript {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
    },

    /// Attach script to a node
    AttachScript {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: String,
        #[arg(long)]
        node_path: String,
        #[arg(long)]
        script_path: String,
    },

    /// Add a function to a script
    AddFunction {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        args: Option<String>,
        #[arg(long)]
        body: Option<String>,
    },

    /// Add an export variable to a script
    AddExportVar {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        var_type: String,
        #[arg(long)]
        default: Option<String>,
    },

    /// Analyze a script
    AnalyzeScript {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
    },

    // === Resource Tools ===
    /// List resources
    ListResources {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        filter_type: Option<String>,
    },

    /// Read a resource file
    ReadResource {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
    },

    /// Create a new resource file
    CreateResource {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long, name = "type")]
        resource_type: String,
    },

    /// Set a property on a resource file
    SetResourceProperty {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        property: String,
        #[arg(long)]
        value: String,
    },

    /// Create a new StandardMaterial3D
    CreateMaterial {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        metallic: Option<f32>,
        #[arg(long)]
        roughness: Option<f32>,
    },

    /// Set a property on a material
    SetMaterialProperty {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        property: String,
        #[arg(long)]
        value: String,
    },

    /// Assign a material to a node in a scene
    AssignMaterial {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: String,
        #[arg(long)]
        node_path: String,
        #[arg(long)]
        material: String,
        #[arg(long)]
        surface_index: Option<u32>,
    },

    // === Editor Tools ===
    /// Get Godot version
    GetGodotVersion {
        #[arg(short, long)]
        project: Option<PathBuf>,
    },

    /// Run the project
    RunProject {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: Option<String>,
    },

    /// Stop the running project
    StopProject {
        #[arg(short, long)]
        project: PathBuf,
    },

    /// Get debug output
    GetDebugOutput {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        lines: Option<u32>,
    },

    /// Launch the Godot editor
    LaunchEditor {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long)]
        scene: Option<String>,
    },

    /// Get running status
    GetRunningStatus {
        #[arg(short, long)]
        project: PathBuf,
    },

    // === Live Commands (via Godot Plugin HTTP) ===
    /// Ping the Godot plugin
    LivePing {
        #[arg(long, default_value = "6060")]
        port: u16,
    },

    /// Add a node via the Godot plugin (real-time, Undo-enabled)
    LiveAddNode {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        parent: String,
        #[arg(long)]
        name: String,
        #[arg(long, name = "type")]
        node_type: String,
    },

    /// Remove a node via the Godot plugin
    LiveRemoveNode {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long)]
        node_path: String,
    },

    /// Get node tree via the Godot plugin
    LiveGetTree {
        #[arg(long, default_value = "6060")]
        port: u16,
    },

    /// Set node property via the Godot plugin
    LiveSetProperty {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        node_path: String,
        #[arg(long)]
        property: String,
        #[arg(long)]
        value: String,
    },

    /// Save the current scene via the Godot plugin
    LiveSaveScene {
        #[arg(long, default_value = "6060")]
        port: u16,
    },

    /// Open a scene in the Godot editor via the Godot plugin
    LiveOpenScene {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long)]
        scene_path: String,
    },

    /// Connect a signal between nodes via the Godot plugin
    LiveConnectSignal {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        source: String,
        #[arg(long)]
        signal: String,
        #[arg(long, default_value = ".")]
        target: String,
        #[arg(long)]
        method: String,
    },

    /// Disconnect a signal between nodes via the Godot plugin
    LiveDisconnectSignal {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        source: String,
        #[arg(long)]
        signal: String,
        #[arg(long, default_value = ".")]
        target: String,
        #[arg(long)]
        method: String,
    },

    /// List signals of a node via the Godot plugin
    LiveListSignals {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        node_path: String,
    },

    // === Live Animation Commands ===
    /// Create a new animation
    LiveCreateAnimation {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = "AnimationPlayer")]
        player: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "1.0")]
        length: f32,
    },

    /// Add a track to an animation
    LiveAddAnimationTrack {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = "AnimationPlayer")]
        player: String,
        #[arg(long)]
        animation: String,
        #[arg(long)]
        track_path: String,
        #[arg(long, default_value = "value")]
        track_type: String,
    },

    /// Add a keyframe to an animation track
    LiveAddAnimationKey {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = "AnimationPlayer")]
        player: String,
        #[arg(long)]
        animation: String,
        #[arg(long, default_value = "0")]
        track: u32,
        #[arg(long)]
        time: f32,
        #[arg(long)]
        value: String,
    },

    /// Play an animation
    LivePlayAnimation {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = "AnimationPlayer")]
        player: String,
        #[arg(long)]
        animation: String,
    },

    /// Stop an animation
    LiveStopAnimation {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = "AnimationPlayer")]
        player: String,
    },

    /// List all animations
    LiveListAnimations {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = "AnimationPlayer")]
        player: String,
    },

    // === Live Debug Log Commands ===
    /// Get editor debug log
    LiveGetEditorLog {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = "50")]
        lines: u32,
    },

    /// Clear editor debug log
    LiveClearEditorLog {
        #[arg(long, default_value = "6060")]
        port: u16,
    },

    // === Live Scene Instance Commands ===
    /// Instantiate a scene (like placing a prefab)
    LiveInstantiateScene {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long)]
        scene_path: String,
        #[arg(long, default_value = ".")]
        parent: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        x: Option<f32>,
        #[arg(long)]
        y: Option<f32>,
        #[arg(long)]
        z: Option<f32>,
    },

    /// Reload the Godot MCP plugin (picks up code changes)
    LiveReloadPlugin {
        #[arg(long, default_value = "6060")]
        port: u16,
    },

    // === Live Group Management Commands ===
    /// Add a node to a group
    LiveAddToGroup {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        node_path: String,
        #[arg(long)]
        group: String,
    },

    /// Remove a node from a group
    LiveRemoveFromGroup {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        node_path: String,
        #[arg(long)]
        group: String,
    },

    /// List groups of a node
    LiveListGroups {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long, default_value = ".")]
        node_path: String,
    },

    /// Get all nodes in a group
    LiveGetGroupNodes {
        #[arg(long, default_value = "6060")]
        port: u16,
        #[arg(long)]
        group: String,
    },
}

/// Execute CLI command
pub async fn run_cli(cmd: ToolCommands) -> anyhow::Result<()> {
    let result = match cmd {
        // === Project Tools ===
        ToolCommands::ListAllScenes { project } => {
            let tools = GodotTools::with_project(project);
            tools.handle_list_all_scenes(None).await
        }
        ToolCommands::ListProjectFiles { project, path } => {
            let tools = GodotTools::with_project(project);
            let args = path.map(|p| {
                let mut map = serde_json::Map::new();
                map.insert("path".to_string(), serde_json::Value::String(p));
                map
            });
            tools.handle_list_project_files(args).await
        }
        ToolCommands::ReadFile { project, path } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            tools.handle_read_file(Some(map)).await
        }
        ToolCommands::SearchInProject {
            project,
            query,
            search_type,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("query".to_string(), serde_json::Value::String(query));
            map.insert(
                "search_type".to_string(),
                serde_json::Value::String(search_type),
            );
            tools.handle_search_in_project(Some(map)).await
        }
        ToolCommands::GetNodeTypeInfo { node_type } => {
            let tools = GodotTools::new();
            let mut map = serde_json::Map::new();
            map.insert(
                "node_type".to_string(),
                serde_json::Value::String(node_type),
            );
            tools.handle_get_node_type_info(Some(map)).await
        }
        ToolCommands::GetProjectStats { project } => {
            let tools = GodotTools::with_project(project);
            tools.handle_get_project_stats(None).await
        }
        ToolCommands::ValidateProject { project } => {
            let tools = GodotTools::with_project(project);
            tools.handle_validate_project(None).await
        }
        ToolCommands::ReadGodotLog { project, lines } => {
            // Get project name from project.godot
            let project_godot = project.join("project.godot");
            let project_name = if project_godot.exists() {
                let content = std::fs::read_to_string(&project_godot).unwrap_or_default();
                content
                    .lines()
                    .find(|line| line.starts_with("config/name="))
                    .map(|line| {
                        line.trim_start_matches("config/name=")
                            .trim_matches('"')
                            .to_string()
                    })
                    .unwrap_or_else(|| project.file_name().unwrap().to_string_lossy().to_string())
            } else {
                project.file_name().unwrap().to_string_lossy().to_string()
            };

            // Build log file path
            let appdata = std::env::var("APPDATA").unwrap_or_default();
            let log_path = std::path::PathBuf::from(&appdata)
                .join("Godot")
                .join("app_userdata")
                .join(&project_name)
                .join("logs")
                .join("godot.log");

            let result = if log_path.exists() {
                let content = std::fs::read_to_string(&log_path).unwrap_or_default();
                let log_lines: Vec<&str> = content.lines().collect();
                let start = if log_lines.len() > lines {
                    log_lines.len() - lines
                } else {
                    0
                };
                let recent_lines: Vec<String> =
                    log_lines[start..].iter().map(|s| s.to_string()).collect();
                serde_json::json!({
                    "success": true,
                    "project": project_name,
                    "log_path": log_path.to_string_lossy(),
                    "total_lines": log_lines.len(),
                    "lines": recent_lines
                })
            } else {
                serde_json::json!({
                    "error": "Log file not found",
                    "expected_path": log_path.to_string_lossy(),
                    "project_name": project_name
                })
            };

            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            return Ok(());
        }

        // === Scene Tools ===
        ToolCommands::CreateScene {
            project,
            path,
            root_type,
            root_name,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert(
                "root_type".to_string(),
                serde_json::Value::String(root_type),
            );
            if let Some(name) = root_name {
                map.insert("root_name".to_string(), serde_json::Value::String(name));
            }
            tools.handle_create_scene(Some(map)).await
        }
        ToolCommands::CreateSceneFromTemplate {
            project,
            path,
            template,
            root_name,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert("template".to_string(), serde_json::Value::String(template));
            if let Some(name) = root_name {
                map.insert("root_name".to_string(), serde_json::Value::String(name));
            }
            tools.handle_create_scene_from_template(Some(map)).await
        }
        ToolCommands::ReadScene { project, path } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            tools.handle_read_scene(Some(map)).await
        }
        ToolCommands::AddNode {
            project,
            scene,
            parent,
            name,
            node_type,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("scene_path".to_string(), serde_json::Value::String(scene));
            map.insert("parent".to_string(), serde_json::Value::String(parent));
            map.insert("name".to_string(), serde_json::Value::String(name));
            map.insert(
                "node_type".to_string(),
                serde_json::Value::String(node_type),
            );
            tools.handle_add_node(Some(map)).await
        }
        ToolCommands::RemoveNode {
            project,
            scene,
            node_path,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("scene_path".to_string(), serde_json::Value::String(scene));
            map.insert(
                "node_path".to_string(),
                serde_json::Value::String(node_path),
            );
            tools.handle_remove_node(Some(map)).await
        }
        ToolCommands::SetNodeProperty {
            project,
            scene,
            node_path,
            property,
            value,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("scene_path".to_string(), serde_json::Value::String(scene));
            map.insert(
                "node_path".to_string(),
                serde_json::Value::String(node_path),
            );
            map.insert("property".to_string(), serde_json::Value::String(property));
            map.insert("value".to_string(), serde_json::Value::String(value));
            tools.handle_set_node_property(Some(map)).await
        }
        ToolCommands::GetNodeTree { project, scene } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(scene));
            tools.handle_get_node_tree(Some(map)).await
        }
        ToolCommands::ValidateTscn { project, path } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            tools.handle_validate_tscn(Some(map)).await
        }
        ToolCommands::CompareScenes {
            project,
            scene_a,
            scene_b,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("scene_a".to_string(), serde_json::Value::String(scene_a));
            map.insert("scene_b".to_string(), serde_json::Value::String(scene_b));
            tools.handle_compare_scenes(Some(map)).await
        }
        ToolCommands::ExportSceneAsJson { project, path } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            tools.handle_export_scene_as_json(Some(map)).await
        }

        // === Script Tools ===
        ToolCommands::CreateScript {
            project,
            path,
            extends,
            class_name,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert("extends".to_string(), serde_json::Value::String(extends));
            if let Some(name) = class_name {
                map.insert("class_name".to_string(), serde_json::Value::String(name));
            }
            tools.handle_create_script(Some(map)).await
        }
        ToolCommands::ReadScript { project, path } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            tools.handle_read_script(Some(map)).await
        }
        ToolCommands::AttachScript {
            project,
            scene,
            node_path,
            script_path,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("scene".to_string(), serde_json::Value::String(scene));
            map.insert(
                "node_path".to_string(),
                serde_json::Value::String(node_path),
            );
            map.insert(
                "script_path".to_string(),
                serde_json::Value::String(script_path),
            );
            tools.handle_attach_script(Some(map)).await
        }
        ToolCommands::AddFunction {
            project,
            path,
            name,
            args,
            body,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert("name".to_string(), serde_json::Value::String(name));
            if let Some(a) = args {
                map.insert("args".to_string(), serde_json::Value::String(a));
            }
            if let Some(b) = body {
                map.insert("body".to_string(), serde_json::Value::String(b));
            }
            tools.handle_add_function(Some(map)).await
        }
        ToolCommands::AddExportVar {
            project,
            path,
            name,
            var_type,
            default,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert("name".to_string(), serde_json::Value::String(name));
            map.insert("var_type".to_string(), serde_json::Value::String(var_type));
            if let Some(d) = default {
                map.insert("default".to_string(), serde_json::Value::String(d));
            }
            tools.handle_add_export_var(Some(map)).await
        }
        ToolCommands::AnalyzeScript { project, path } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            tools.handle_analyze_script(Some(map)).await
        }

        // === Resource Tools ===
        ToolCommands::ListResources {
            project,
            filter_type,
        } => {
            let tools = GodotTools::with_project(project);
            let args = filter_type.map(|ft| {
                let mut map = serde_json::Map::new();
                map.insert("filter_type".to_string(), serde_json::Value::String(ft));
                map
            });
            tools.handle_list_resources(args).await
        }
        ToolCommands::ReadResource { project, path } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            tools.handle_read_resource(Some(map)).await
        }
        ToolCommands::CreateResource {
            project,
            path,
            resource_type,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert(
                "resource_type".to_string(),
                serde_json::Value::String(resource_type),
            );
            tools.handle_create_resource(Some(map)).await
        }
        ToolCommands::SetResourceProperty {
            project,
            path,
            property,
            value,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert("property".to_string(), serde_json::Value::String(property));
            map.insert("value".to_string(), serde_json::Value::String(value));
            tools.handle_set_resource_property(Some(map)).await
        }
        ToolCommands::CreateMaterial {
            project,
            path,
            name,
            metallic,
            roughness,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            if let Some(n) = name {
                map.insert("name".to_string(), serde_json::Value::String(n));
            }
            if let Some(m) = metallic {
                map.insert(
                    "metallic".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(m as f64).unwrap()),
                );
            }
            if let Some(r) = roughness {
                map.insert(
                    "roughness".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(r as f64).unwrap()),
                );
            }
            tools.handle_create_material(Some(map)).await
        }
        ToolCommands::SetMaterialProperty {
            project,
            path,
            property,
            value,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), serde_json::Value::String(path));
            map.insert("property".to_string(), serde_json::Value::String(property));
            map.insert("value".to_string(), serde_json::Value::String(value));
            tools.handle_set_material_property(Some(map)).await
        }
        ToolCommands::AssignMaterial {
            project,
            scene,
            node_path,
            material,
            surface_index,
        } => {
            let tools = GodotTools::with_project(project);
            let mut map = serde_json::Map::new();
            map.insert("scene_path".to_string(), serde_json::Value::String(scene));
            map.insert(
                "node_path".to_string(),
                serde_json::Value::String(node_path),
            );
            map.insert(
                "material_path".to_string(),
                serde_json::Value::String(material),
            );
            if let Some(idx) = surface_index {
                map.insert(
                    "surface_index".to_string(),
                    serde_json::Value::Number(idx.into()),
                );
            }
            tools.handle_assign_material(Some(map)).await
        }

        // === Editor Tools ===
        ToolCommands::GetGodotVersion { project } => {
            let tools = if let Some(p) = project {
                GodotTools::with_project(p)
            } else {
                GodotTools::new()
            };
            tools.handle_get_godot_version(None).await
        }
        ToolCommands::RunProject { project, scene } => {
            let tools = GodotTools::with_project(project);
            let args = scene.map(|s| {
                let mut map = serde_json::Map::new();
                map.insert("scene".to_string(), serde_json::Value::String(s));
                map
            });
            tools.handle_run_project(args).await
        }
        ToolCommands::StopProject { project } => {
            let tools = GodotTools::with_project(project);
            tools.handle_stop_project(None).await
        }
        ToolCommands::GetDebugOutput { project, lines } => {
            let tools = GodotTools::with_project(project);
            let args = lines.map(|l| {
                let mut map = serde_json::Map::new();
                map.insert("lines".to_string(), serde_json::Value::Number(l.into()));
                map
            });
            tools.handle_get_debug_output(args).await
        }
        ToolCommands::LaunchEditor { project, scene } => {
            let tools = GodotTools::with_project(project);
            let args = scene.map(|s| {
                let mut map = serde_json::Map::new();
                map.insert("scene".to_string(), serde_json::Value::String(s));
                map
            });
            tools.handle_launch_editor(args).await
        }
        ToolCommands::GetRunningStatus { project } => {
            let tools = GodotTools::with_project(project);
            tools.handle_get_running_status(None).await
        }

        // === Live Commands (via Godot Plugin HTTP) ===
        ToolCommands::LivePing { port } => {
            return run_live_command(port, GodotCommand::Ping).await;
        }
        ToolCommands::LiveAddNode {
            port,
            parent,
            name,
            node_type,
        } => {
            return run_live_command(
                port,
                GodotCommand::AddNode(AddNodeParams {
                    parent,
                    name,
                    node_type,
                }),
            )
            .await;
        }
        ToolCommands::LiveRemoveNode { port, node_path } => {
            return run_live_command(
                port,
                GodotCommand::RemoveNode(RemoveNodeParams { node_path }),
            )
            .await;
        }
        ToolCommands::LiveGetTree { port } => {
            return run_live_command(port, GodotCommand::GetTree).await;
        }
        ToolCommands::LiveSetProperty {
            port,
            node_path,
            property,
            value,
        } => {
            return run_live_command(
                port,
                GodotCommand::SetProperty(SetPropertyParams {
                    node_path,
                    property,
                    value: serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value)),
                }),
            )
            .await;
        }
        ToolCommands::LiveSaveScene { port } => {
            return run_live_command(port, GodotCommand::SaveScene).await;
        }
        ToolCommands::LiveOpenScene { port, scene_path } => {
            return run_live_command(
                port,
                GodotCommand::OpenScene(OpenSceneParams { scene_path }),
            )
            .await;
        }
        ToolCommands::LiveConnectSignal {
            port,
            source,
            signal,
            target,
            method,
        } => {
            return run_live_command(
                port,
                GodotCommand::ConnectSignal(SignalParams {
                    source,
                    signal,
                    target,
                    method,
                }),
            )
            .await;
        }
        ToolCommands::LiveDisconnectSignal {
            port,
            source,
            signal,
            target,
            method,
        } => {
            return run_live_command(
                port,
                GodotCommand::DisconnectSignal(SignalParams {
                    source,
                    signal,
                    target,
                    method,
                }),
            )
            .await;
        }
        ToolCommands::LiveListSignals { port, node_path } => {
            return run_live_command(
                port,
                GodotCommand::ListSignals(NodePathParams { node_path }),
            )
            .await;
        }

        // === Live Animation Commands ===
        ToolCommands::LiveCreateAnimation {
            port,
            player,
            name,
            length,
        } => {
            return run_live_command(
                port,
                GodotCommand::CreateAnimation(CreateAnimationParams {
                    player,
                    name,
                    length,
                }),
            )
            .await;
        }
        ToolCommands::LiveAddAnimationTrack {
            port,
            player,
            animation,
            track_path,
            track_type,
        } => {
            return run_live_command(
                port,
                GodotCommand::AddAnimationTrack(AddAnimationTrackParams {
                    player,
                    animation,
                    track_path,
                    track_type,
                }),
            )
            .await;
        }
        ToolCommands::LiveAddAnimationKey {
            port,
            player,
            animation,
            track,
            time,
            value,
        } => {
            return run_live_command(
                port,
                GodotCommand::AddAnimationKey(AddAnimationKeyParams {
                    player,
                    animation,
                    track: track as usize,
                    time,
                    value: serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value)),
                }),
            )
            .await;
        }
        ToolCommands::LivePlayAnimation {
            port,
            player,
            animation,
        } => {
            return run_live_command(
                port,
                GodotCommand::PlayAnimation(PlayAnimationParams { player, animation }),
            )
            .await;
        }
        ToolCommands::LiveStopAnimation { port, player } => {
            return run_live_command(
                port,
                GodotCommand::StopAnimation(StopAnimationParams { player }),
            )
            .await;
        }
        ToolCommands::LiveListAnimations { port, player } => {
            return run_live_command(
                port,
                GodotCommand::ListAnimations(StopAnimationParams { player }),
            )
            .await;
        }

        // === Live Debug Log Commands ===
        ToolCommands::LiveGetEditorLog { port, lines } => {
            return run_live_command(port, GodotCommand::GetEditorLog(GetLogParams { lines }))
                .await;
        }
        ToolCommands::LiveClearEditorLog { port } => {
            return run_live_command(port, GodotCommand::ClearEditorLog).await;
        }

        // === Live Scene Instance Commands ===
        ToolCommands::LiveInstantiateScene {
            port,
            scene_path,
            parent,
            name,
            x,
            y,
            z,
        } => {
            return run_live_command(
                port,
                GodotCommand::InstantiateScene(InstantiateSceneParams {
                    scene_path,
                    parent,
                    name,
                    position: if x.is_some() || y.is_some() || z.is_some() {
                        Some(Position3D {
                            x: x.unwrap_or(0.0),
                            y: y.unwrap_or(0.0),
                            z: z.unwrap_or(0.0),
                        })
                    } else {
                        None
                    },
                }),
            )
            .await;
        }
        ToolCommands::LiveReloadPlugin { port } => {
            return run_live_command(port, GodotCommand::ReloadPlugin).await;
        }

        // === Live Group Management Commands ===
        ToolCommands::LiveAddToGroup {
            port,
            node_path,
            group,
        } => {
            return run_live_command(
                port,
                GodotCommand::AddToGroup(GroupParams { node_path, group }),
            )
            .await;
        }
        ToolCommands::LiveRemoveFromGroup {
            port,
            node_path,
            group,
        } => {
            return run_live_command(
                port,
                GodotCommand::RemoveFromGroup(GroupParams { node_path, group }),
            )
            .await;
        }
        ToolCommands::LiveListGroups { port, node_path } => {
            return run_live_command(port, GodotCommand::ListGroups(NodePathParams { node_path }))
                .await;
        }
        ToolCommands::LiveGetGroupNodes { port, group } => {
            return run_live_command(port, GodotCommand::GetGroupNodes(GroupNameParams { group }))
                .await;
        }
    };

    match result {
        Ok(tool_result) => {
            // Display results
            for content in &tool_result.content {
                if let Some(text) = content.raw.as_text() {
                    println!("{}", text.text);
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}

/// Execute a live command via HTTP to the Godot plugin
async fn run_live_command(port: u16, command: GodotCommand) -> anyhow::Result<()> {
    let url = format!("http://localhost:{}", port);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&command)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            // Parse and format output
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("{}", text);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to connect to Godot plugin: {}", e);
            eprintln!("Make sure the Godot editor is running with the MCP plugin enabled.");
            std::process::exit(1);
        }
    }
}
