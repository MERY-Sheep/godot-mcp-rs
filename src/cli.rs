//! CLI モード - ターミナル/スクリプトからの直接操作

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
            map.insert("scene".to_string(), serde_json::Value::String(scene));
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
            map.insert("scene".to_string(), serde_json::Value::String(scene));
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
            map.insert("scene".to_string(), serde_json::Value::String(scene));
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
            map.insert("scene".to_string(), serde_json::Value::String(scene));
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
            return run_live_command(port, "ping", serde_json::json!({})).await;
        }
        ToolCommands::LiveAddNode {
            port,
            parent,
            name,
            node_type,
        } => {
            return run_live_command(
                port,
                "add_node",
                serde_json::json!({
                    "parent": parent,
                    "name": name,
                    "node_type": node_type
                }),
            )
            .await;
        }
        ToolCommands::LiveRemoveNode { port, node_path } => {
            return run_live_command(
                port,
                "remove_node",
                serde_json::json!({
                    "node_path": node_path
                }),
            )
            .await;
        }
        ToolCommands::LiveGetTree { port } => {
            return run_live_command(port, "get_tree", serde_json::json!({})).await;
        }
        ToolCommands::LiveSetProperty {
            port,
            node_path,
            property,
            value,
        } => {
            return run_live_command(
                port,
                "set_property",
                serde_json::json!({
                    "node_path": node_path,
                    "property": property,
                    "value": value
                }),
            )
            .await;
        }
        ToolCommands::LiveSaveScene { port } => {
            return run_live_command(port, "save_scene", serde_json::json!({})).await;
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
                "connect_signal",
                serde_json::json!({
                    "source": source,
                    "signal": signal,
                    "target": target,
                    "method": method
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
                "disconnect_signal",
                serde_json::json!({
                    "source": source,
                    "signal": signal,
                    "target": target,
                    "method": method
                }),
            )
            .await;
        }
        ToolCommands::LiveListSignals { port, node_path } => {
            return run_live_command(
                port,
                "list_signals",
                serde_json::json!({
                    "node_path": node_path
                }),
            )
            .await;
        }
    };

    match result {
        Ok(tool_result) => {
            // 結果を表示
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
async fn run_live_command(
    port: u16,
    command: &str,
    params: serde_json::Value,
) -> anyhow::Result<()> {
    let url = format!("http://localhost:{}", port);
    let body = serde_json::json!({
        "command": command,
        "params": params
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await;

    match response {
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            // パースしてフォーマット出力
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
