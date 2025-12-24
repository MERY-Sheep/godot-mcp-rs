//! Dependency Resolver
//!
//! Analyzes dependencies between scenes, scripts, and resources.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use regex::Regex;

use crate::godot::tscn::GodotScene;

use super::context::GqlContext;
use super::resolver::{resolve_scene, resolve_script};
use super::types::*;

// ======================
// Dependency Graph Building
// ======================

/// A dependency edge between two files
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub reference_type: ReferenceType,
}

/// Build the complete dependency graph for the project
pub fn build_dependency_graph(ctx: &GqlContext) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut nodes: HashMap<String, GraphNode> = HashMap::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    // Collect all files
    let (scenes, scripts) = collect_files(&ctx.project_path);

    // Add scene nodes
    for scene_path in &scenes {
        let res_path = to_res_path(&ctx.project_path, scene_path);
        nodes.insert(
            res_path.clone(),
            GraphNode {
                id: res_path.clone(),
                label: scene_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| res_path.clone()),
                node_type: FileType::Scene,
                in_degree: 0,
                out_degree: 0,
                is_unused: true,
            },
        );

        // Parse scene and extract dependencies
        if let Ok(content) = fs::read_to_string(scene_path) {
            if let Ok(scene) = GodotScene::parse(&content) {
                for ext_res in &scene.ext_resources {
                    let ref_type = match ext_res.resource_type.as_str() {
                        "Script" | "GDScript" => ReferenceType::AttachesScript,
                        "PackedScene" => ReferenceType::Instantiates,
                        _ => ReferenceType::UsesResource,
                    };

                    edges.push(GraphEdge {
                        from: res_path.clone(),
                        to: ext_res.path.clone(),
                        reference_type: ref_type,
                    });
                }
            }
        }
    }

    // Add script nodes
    for script_path in &scripts {
        let res_path = to_res_path(&ctx.project_path, script_path);
        nodes.insert(
            res_path.clone(),
            GraphNode {
                id: res_path.clone(),
                label: script_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| res_path.clone()),
                node_type: FileType::Script,
                in_degree: 0,
                out_degree: 0,
                is_unused: true,
            },
        );

        // Parse script and extract preload/load dependencies
        if let Ok(content) = fs::read_to_string(script_path) {
            let deps = extract_script_dependencies(&content);
            for (dep_path, ref_type) in deps {
                edges.push(GraphEdge {
                    from: res_path.clone(),
                    to: dep_path,
                    reference_type: ref_type,
                });
            }
        }
    }

    // Calculate in/out degrees and unused status
    for edge in &edges {
        if let Some(from_node) = nodes.get_mut(&edge.from) {
            from_node.out_degree += 1;
        }
        if let Some(to_node) = nodes.get_mut(&edge.to) {
            to_node.in_degree += 1;
            to_node.is_unused = false;
        }
    }

    (nodes.into_values().collect(), edges)
}

/// Extract dependencies from script content (preload/load calls)
fn extract_script_dependencies(content: &str) -> Vec<(String, ReferenceType)> {
    let mut deps = Vec::new();

    // Match preload("res://...") and load("res://...")
    let preload_re = Regex::new(r#"preload\s*\(\s*"(res://[^"]+)"\s*\)"#).unwrap();
    let load_re = Regex::new(r#"load\s*\(\s*"(res://[^"]+)"\s*\)"#).unwrap();

    for cap in preload_re.captures_iter(content) {
        if let Some(path) = cap.get(1) {
            deps.push((path.as_str().to_string(), ReferenceType::Preloads));
        }
    }

    for cap in load_re.captures_iter(content) {
        if let Some(path) = cap.get(1) {
            deps.push((path.as_str().to_string(), ReferenceType::Loads));
        }
    }

    deps
}

/// Collect scene and script files from project
fn collect_files(project_path: &Path) -> (Vec<std::path::PathBuf>, Vec<std::path::PathBuf>) {
    let mut scenes = Vec::new();
    let mut scripts = Vec::new();
    collect_files_recursive(project_path, &mut scenes, &mut scripts);
    (scenes, scripts)
}

fn collect_files_recursive(
    dir: &Path,
    scenes: &mut Vec<std::path::PathBuf>,
    scripts: &mut Vec<std::path::PathBuf>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip .godot and addons directories
        if path
            .file_name()
            .map(|n| n == ".godot" || n == "addons")
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(&path, scenes, scripts);
        } else if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("tscn") | Some("scn") => scenes.push(path),
                Some("gd") => scripts.push(path),
                _ => {}
            }
        }
    }
}

/// Convert filesystem path to res:// path
fn to_res_path(project_root: &Path, file_path: &Path) -> String {
    let relative = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");
    format!("res://{}", relative)
}

/// Convert res:// path to filesystem path
fn res_path_to_fs_path(project_root: &Path, res_path: &str) -> std::path::PathBuf {
    let relative = res_path.strip_prefix("res://").unwrap_or(res_path);
    project_root.join(relative)
}

// ======================
// Cycle Detection
// ======================

/// Detect cycles in the dependency graph
fn detect_cycles(nodes: &[GraphNode], edges: &[GraphEdge]) -> (bool, Vec<Vec<String>>) {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for node in nodes {
        adj.insert(&node.id, Vec::new());
    }

    for edge in edges {
        if let Some(neighbors) = adj.get_mut(edge.from.as_str()) {
            neighbors.push(&edge.to);
        }
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut rec_stack: HashSet<&str> = HashSet::new();
    let mut cycles: Vec<Vec<String>> = Vec::new();

    for node in nodes {
        if !visited.contains(node.id.as_str()) {
            let mut path = Vec::new();
            if dfs_cycle(&node.id, &adj, &mut visited, &mut rec_stack, &mut path) {
                cycles.push(path);
            }
        }
    }

    (!cycles.is_empty(), cycles)
}

fn dfs_cycle<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    rec_stack: &mut HashSet<&'a str>,
    path: &mut Vec<String>,
) -> bool {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(*neighbor) {
                if dfs_cycle(neighbor, adj, visited, rec_stack, path) {
                    return true;
                }
            } else if rec_stack.contains(*neighbor) {
                path.push(neighbor.to_string());
                return true;
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
    false
}

// ======================
// Export Formats
// ======================

/// Export graph to MERMAID format
fn export_to_mermaid(nodes: &[GraphNode], edges: &[GraphEdge]) -> String {
    let mut output = String::from("graph LR\n");

    for node in nodes {
        let shape = match node.node_type {
            FileType::Scene => format!("{}[{}]", sanitize_id(&node.id), node.label),
            FileType::Script => format!("{}(({}))", sanitize_id(&node.id), node.label),
            _ => format!("{}>{}>]", sanitize_id(&node.id), node.label),
        };
        output.push_str(&format!("    {}\n", shape));
    }

    for edge in edges {
        let arrow = match edge.reference_type {
            ReferenceType::Instantiates => "-->",
            ReferenceType::AttachesScript => "-.->",
            ReferenceType::Preloads => "==>",
            ReferenceType::Loads => "-->",
            ReferenceType::UsesResource => "-.->",
        };
        output.push_str(&format!(
            "    {} {} {}\n",
            sanitize_id(&edge.from),
            arrow,
            sanitize_id(&edge.to)
        ));
    }

    output
}

/// Export graph to DOT format
fn export_to_dot(nodes: &[GraphNode], edges: &[GraphEdge]) -> String {
    let mut output = String::from("digraph Dependencies {\n");
    output.push_str("    rankdir=LR;\n");

    for node in nodes {
        let shape = match node.node_type {
            FileType::Scene => "box",
            FileType::Script => "ellipse",
            _ => "diamond",
        };
        output.push_str(&format!(
            "    \"{}\" [label=\"{}\" shape={}];\n",
            node.id, node.label, shape
        ));
    }

    for edge in edges {
        let style = match edge.reference_type {
            ReferenceType::Instantiates => "solid",
            ReferenceType::AttachesScript => "dashed",
            ReferenceType::Preloads => "bold",
            ReferenceType::Loads => "solid",
            ReferenceType::UsesResource => "dotted",
        };
        output.push_str(&format!(
            "    \"{}\" -> \"{}\" [style={}];\n",
            edge.from, edge.to, style
        ));
    }

    output.push_str("}\n");
    output
}

/// Sanitize node ID for MERMAID (replace special chars)
fn sanitize_id(id: &str) -> String {
    id.replace("res://", "")
        .replace('/', "_")
        .replace('.', "_")
        .replace('-', "_")
}

// ======================
// Query Resolvers
// ======================

/// Resolve gatherContext query
pub fn resolve_gather_context(ctx: &GqlContext, input: GatherContextInput) -> GatheredContext {
    let depth = input.depth.unwrap_or(2);
    let include_types = input
        .include
        .unwrap_or_else(|| vec![FileType::Scene, FileType::Script]);

    // Determine file type of entry point
    let entry_type = if input.entry_point.ends_with(".tscn") || input.entry_point.ends_with(".scn")
    {
        FileType::Scene
    } else if input.entry_point.ends_with(".gd") {
        FileType::Script
    } else {
        FileType::Resource
    };

    // Load main item
    let main = ContextItem {
        path: input.entry_point.clone(),
        file_type: entry_type,
        scene: if entry_type == FileType::Scene {
            resolve_scene(ctx, &input.entry_point)
        } else {
            None
        },
        script: if entry_type == FileType::Script {
            resolve_script(ctx, &input.entry_point)
        } else {
            None
        },
    };

    // Collect dependencies (outgoing references)
    let mut dependencies = Vec::new();
    let mut visited = HashSet::new();
    visited.insert(input.entry_point.clone());
    collect_dependencies(
        ctx,
        &input.entry_point,
        depth,
        &include_types,
        &mut dependencies,
        &mut visited,
    );

    // Collect dependents (incoming references)
    let dependents = find_dependents(ctx, &input.entry_point);

    // Collect resources
    let resources = collect_resources(ctx, &input.entry_point);

    // Calculate summary
    let total_functions = main
        .script
        .as_ref()
        .map(|s| s.functions.len() as i32)
        .unwrap_or(0)
        + dependencies
            .iter()
            .filter_map(|d| d.script.as_ref())
            .map(|s| s.functions.len() as i32)
            .sum::<i32>();

    let summary = ContextSummary {
        total_files: 1 + dependencies.len() as i32 + dependents.len() as i32,
        total_functions,
    };

    GatheredContext {
        entry_point: input.entry_point,
        main,
        dependencies,
        dependents,
        resources,
        summary,
    }
}

/// Recursively collect dependencies
fn collect_dependencies(
    ctx: &GqlContext,
    path: &str,
    depth: i32,
    include_types: &[FileType],
    result: &mut Vec<ContextItem>,
    visited: &mut HashSet<String>,
) {
    if depth <= 0 {
        return;
    }

    let fs_path = res_path_to_fs_path(&ctx.project_path, path);
    let deps = if path.ends_with(".tscn") || path.ends_with(".scn") {
        extract_scene_dependencies(&fs_path)
    } else if path.ends_with(".gd") {
        extract_script_deps(&fs_path)
    } else {
        Vec::new()
    };

    for dep_path in deps {
        if visited.contains(&dep_path) {
            continue;
        }
        visited.insert(dep_path.clone());

        let file_type = if dep_path.ends_with(".tscn") || dep_path.ends_with(".scn") {
            FileType::Scene
        } else if dep_path.ends_with(".gd") {
            FileType::Script
        } else {
            FileType::Resource
        };

        // Check if this type should be included
        if !include_types.contains(&file_type) {
            continue;
        }

        let item = ContextItem {
            path: dep_path.clone(),
            file_type,
            scene: if file_type == FileType::Scene {
                resolve_scene(ctx, &dep_path)
            } else {
                None
            },
            script: if file_type == FileType::Script {
                resolve_script(ctx, &dep_path)
            } else {
                None
            },
        };

        result.push(item);

        // Recursive call
        collect_dependencies(ctx, &dep_path, depth - 1, include_types, result, visited);
    }
}

/// Extract dependencies from a scene file
fn extract_scene_dependencies(path: &Path) -> Vec<String> {
    let mut deps = Vec::new();

    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(scene) = GodotScene::parse(&content) {
            for ext_res in scene.ext_resources {
                deps.push(ext_res.path);
            }
        }
    }

    deps
}

/// Extract dependencies from a script file
fn extract_script_deps(path: &Path) -> Vec<String> {
    let mut deps = Vec::new();

    if let Ok(content) = fs::read_to_string(path) {
        for (dep_path, _) in extract_script_dependencies(&content) {
            deps.push(dep_path);
        }
    }

    deps
}

/// Find files that depend on the given path
fn find_dependents(ctx: &GqlContext, target_path: &str) -> Vec<ContextItem> {
    let mut dependents = Vec::new();
    let (scenes, scripts) = collect_files(&ctx.project_path);

    // Check scenes
    for scene_path in scenes {
        let res_path = to_res_path(&ctx.project_path, &scene_path);
        if res_path == target_path {
            continue;
        }

        let deps = extract_scene_dependencies(&scene_path);
        if deps.contains(&target_path.to_string()) {
            dependents.push(ContextItem {
                path: res_path.clone(),
                file_type: FileType::Scene,
                scene: resolve_scene(ctx, &res_path),
                script: None,
            });
        }
    }

    // Check scripts
    for script_path in scripts {
        let res_path = to_res_path(&ctx.project_path, &script_path);
        if res_path == target_path {
            continue;
        }

        let deps = extract_script_deps(&script_path);
        if deps.contains(&target_path.to_string()) {
            dependents.push(ContextItem {
                path: res_path.clone(),
                file_type: FileType::Script,
                scene: None,
                script: resolve_script(ctx, &res_path),
            });
        }
    }

    dependents
}

/// Collect resource references from a file
fn collect_resources(ctx: &GqlContext, path: &str) -> Vec<ResourceInfo> {
    let mut resources = Vec::new();
    let fs_path = res_path_to_fs_path(&ctx.project_path, path);

    if path.ends_with(".tscn") || path.ends_with(".scn") {
        if let Ok(content) = fs::read_to_string(&fs_path) {
            if let Ok(scene) = GodotScene::parse(&content) {
                for ext_res in scene.ext_resources {
                    // Only include non-scene, non-script resources
                    if ext_res.resource_type != "Script"
                        && ext_res.resource_type != "GDScript"
                        && ext_res.resource_type != "PackedScene"
                    {
                        resources.push(ResourceInfo {
                            path: ext_res.path,
                            resource_type: Some(ext_res.resource_type),
                        });
                    }
                }
            }
        }
    }

    resources
}

/// Resolve dependencyGraph query
pub fn resolve_dependency_graph(
    ctx: &GqlContext,
    input: Option<DependencyGraphInput>,
) -> DependencyGraph {
    let (nodes, edges) = build_dependency_graph(ctx);

    // Detect cycles
    let (has_cycles, cycle_paths) = detect_cycles(&nodes, &edges);

    // Calculate stats
    let unused_count = nodes.iter().filter(|n| n.is_unused).count() as i32;

    let stats = GraphStats {
        node_count: nodes.len() as i32,
        edge_count: edges.len() as i32,
        unused_count,
        has_cycles,
        cycle_paths: if has_cycles { Some(cycle_paths) } else { None },
    };

    // Export data if format requested
    let exported_data = input.as_ref().and_then(|i| i.format).map(|format| {
        match format {
            GraphFormat::Mermaid => export_to_mermaid(&nodes, &edges),
            GraphFormat::Dot => export_to_dot(&nodes, &edges),
            GraphFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                "nodes": nodes.iter().map(|n| serde_json::json!({
                    "id": n.id,
                    "label": n.label,
                    "type": format!("{:?}", n.node_type),
                    "inDegree": n.in_degree,
                    "outDegree": n.out_degree,
                    "isUnused": n.is_unused,
                })).collect::<Vec<_>>(),
                "edges": edges.iter().map(|e| serde_json::json!({
                    "from": e.from,
                    "to": e.to,
                    "type": format!("{:?}", e.reference_type),
                })).collect::<Vec<_>>(),
            }))
            .unwrap_or_default(),
            GraphFormat::Graphml => String::new(), // TODO: Implement GraphML export
        }
    });

    DependencyGraph {
        nodes,
        edges,
        stats,
        exported_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_script_dependencies() {
        let content = r#"extends Node

const Weapon = preload("res://scenes/weapon.tscn")
var item = load("res://resources/item.tres")

func _ready():
    pass
"#;
        let deps = extract_script_dependencies(content);
        // Should find preload and load
        assert!(
            deps.iter()
                .any(|(p, t)| p == "res://scenes/weapon.tscn" && *t == ReferenceType::Preloads),
            "Should find preload for weapon.tscn"
        );
        assert!(
            deps.iter()
                .any(|(p, t)| p == "res://resources/item.tres" && *t == ReferenceType::Loads),
            "Should find load for item.tres"
        );
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(
            sanitize_id("res://scenes/player.tscn"),
            "scenes_player_tscn"
        );
    }

    #[test]
    fn test_export_to_mermaid() {
        let nodes = vec![GraphNode {
            id: "res://scenes/player.tscn".to_string(),
            label: "player.tscn".to_string(),
            node_type: FileType::Scene,
            in_degree: 0,
            out_degree: 1,
            is_unused: false,
        }];

        let edges = vec![];

        let mermaid = export_to_mermaid(&nodes, &edges);
        assert!(mermaid.contains("graph LR"));
        assert!(mermaid.contains("player.tscn"));
    }
}
