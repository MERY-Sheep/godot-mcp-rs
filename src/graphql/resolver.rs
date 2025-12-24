//! GraphQL Resolvers
//!
//! Connects GraphQL queries to Godot file parsers.

use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::godot::gdscript::GDScript;
use crate::godot::tscn::GodotScene;

use super::context::GqlContext;
use super::types::*;

/// Resolve project information
pub fn resolve_project(ctx: &GqlContext) -> Project {
    let project_path = &ctx.project_path;

    // Read project.godot to get project name
    let project_godot_path = project_path.join("project.godot");
    let name = if project_godot_path.exists() {
        parse_project_name(&project_godot_path).unwrap_or_else(|| "Unknown".to_string())
    } else {
        project_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    };

    // Collect scenes and scripts
    let (scenes, scripts) = collect_project_files(project_path);

    // Count resources (*.tres, *.res files)
    let resource_count = count_resources(project_path);

    let stats = ProjectStats {
        scene_count: scenes.len() as i32,
        script_count: scripts.len() as i32,
        resource_count,
    };

    // Basic validation
    let validation = validate_project(project_path, &scenes, &scripts);

    Project {
        name,
        path: project_path.to_string_lossy().to_string(),
        scenes,
        scripts,
        stats,
        validation,
    }
}

/// Parse project name from project.godot
fn parse_project_name(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if line.starts_with("config/name=") {
            let value = line.strip_prefix("config/name=")?;
            // Remove quotes
            let trimmed = value.trim_matches('"');
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Collect scene and script files from project
fn collect_project_files(project_path: &Path) -> (Vec<SceneFile>, Vec<ScriptFile>) {
    let mut scenes = Vec::new();
    let mut scripts = Vec::new();

    collect_files_recursive(project_path, project_path, &mut scenes, &mut scripts);

    // Sort by path for consistent output
    scenes.sort_by(|a, b| a.path.cmp(&b.path));
    scripts.sort_by(|a, b| a.path.cmp(&b.path));

    (scenes, scripts)
}

/// Recursively collect scene and script files
fn collect_files_recursive(
    root: &Path,
    dir: &Path,
    scenes: &mut Vec<SceneFile>,
    scripts: &mut Vec<ScriptFile>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip .godot directory
        if path
            .file_name()
            .map(|n| n == ".godot" || n == "addons")
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(root, &path, scenes, scripts);
        } else if let Some(ext) = path.extension() {
            let res_path = to_res_path(root, &path);
            match ext.to_str() {
                Some("tscn") | Some("scn") => {
                    scenes.push(SceneFile { path: res_path });
                }
                Some("gd") => {
                    scripts.push(ScriptFile { path: res_path });
                }
                _ => {}
            }
        }
    }
}

/// Count resource files
fn count_resources(project_path: &Path) -> i32 {
    let mut count = 0;
    count_resources_recursive(project_path, &mut count);
    count
}

fn count_resources_recursive(dir: &Path, count: &mut i32) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path
            .file_name()
            .map(|n| n == ".godot" || n == "addons")
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            count_resources_recursive(&path, count);
        } else if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("tres") | Some("res") => {
                    *count += 1;
                }
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

/// Basic project validation
fn validate_project(
    _project_path: &Path,
    _scenes: &[SceneFile],
    _scripts: &[ScriptFile],
) -> ProjectValidationResult {
    // For now, just return valid
    // TODO: Add actual validation (missing references, etc.)
    ProjectValidationResult {
        is_valid: true,
        errors: vec![],
        warnings: vec![],
    }
}

/// Resolve scene from file path
pub fn resolve_scene(ctx: &GqlContext, res_path: &str) -> Option<Scene> {
    let file_path = res_path_to_fs_path(&ctx.project_path, res_path);
    let content = fs::read_to_string(&file_path).ok()?;
    let godot_scene = GodotScene::parse(&content).ok()?;

    Some(convert_godot_scene_to_gql(&godot_scene, res_path))
}

/// Resolve script from file path
pub fn resolve_script(ctx: &GqlContext, res_path: &str) -> Option<Script> {
    let file_path = res_path_to_fs_path(&ctx.project_path, res_path);
    let content = fs::read_to_string(&file_path).ok()?;
    let gdscript = GDScript::parse(&content);

    Some(convert_gdscript_to_gql(&gdscript, res_path))
}

/// Convert res:// path to filesystem path
fn res_path_to_fs_path(project_root: &Path, res_path: &str) -> std::path::PathBuf {
    let relative = res_path.strip_prefix("res://").unwrap_or(res_path);
    project_root.join(relative)
}

/// Convert GodotScene to GraphQL Scene
fn convert_godot_scene_to_gql(scene: &GodotScene, path: &str) -> Scene {
    // Build node tree
    let all_nodes: Vec<SceneNode> = scene
        .nodes
        .iter()
        .map(|n| {
            let node_path = if n.parent.is_none() {
                ".".to_string()
            } else if n.parent.as_deref() == Some(".") {
                n.name.clone()
            } else {
                format!("{}/{}", n.parent.as_deref().unwrap_or("."), n.name)
            };

            SceneNode {
                name: n.name.clone(),
                r#type: n.node_type.clone(),
                path: node_path,
                properties: n
                    .properties
                    .iter()
                    .map(|(k, v)| Property {
                        name: k.clone(),
                        value: v.clone(),
                        property_type: None,
                    })
                    .collect(),
                children: vec![], // Filled later if needed
                script: None,     // TODO: Parse script reference
                groups: vec![],   // TODO: Parse groups
                signals: vec![],  // TODO: Parse signal connections
            }
        })
        .collect();

    // Root is the first node (no parent)
    let root = all_nodes
        .first()
        .cloned()
        .unwrap_or_else(|| SceneNode {
            name: "Root".to_string(),
            r#type: "Node".to_string(),
            path: ".".to_string(),
            properties: vec![],
            children: vec![],
            script: None,
            groups: vec![],
            signals: vec![],
        });

    // External resources
    let external_resources: Vec<ExternalResource> = scene
        .ext_resources
        .iter()
        .map(|r| ExternalResource {
            id: r.id.parse().unwrap_or(0),
            resource_type: r.resource_type.clone(),
            path: r.path.clone(),
        })
        .collect();

    Scene {
        path: path.to_string(),
        root,
        all_nodes,
        external_resources,
    }
}

/// Convert GDScript to GraphQL Script
fn convert_gdscript_to_gql(script: &GDScript, path: &str) -> Script {
    Script {
        path: path.to_string(),
        extends: script.extends.clone().unwrap_or_else(|| "Node".to_string()),
        class_name: script.class_name.clone(),
        functions: script
            .functions
            .iter()
            .map(|f| Function {
                name: f.name.clone(),
                arguments: f.params.iter().map(|p| p.name.clone()).collect(),
            })
            .collect(),
        variables: script
            .variables
            .iter()
            .map(|v| Variable {
                name: v.name.clone(),
                var_type: v.var_type.clone().unwrap_or_else(|| "Variant".to_string()),
                default_value: v.default_value.clone(),
            })
            .collect(),
        signals: script
            .signals
            .iter()
            .map(|s| {
                // Parse signal arguments from signal definition
                // e.g., "health_changed(new_value: int)"
                let (name, args) = parse_signal_definition(s);
                SignalDefinition {
                    name,
                    arguments: args,
                }
            })
            .collect(),
        exports: script
            .exports
            .iter()
            .map(|e| Variable {
                name: e.name.clone(),
                var_type: e.var_type.clone().unwrap_or_else(|| "Variant".to_string()),
                default_value: e.default_value.clone(),
            })
            .collect(),
    }
}

/// Parse signal definition string
fn parse_signal_definition(signal_str: &str) -> (String, Vec<String>) {
    if let Some(paren_start) = signal_str.find('(') {
        let name = signal_str[..paren_start].trim().to_string();
        let paren_end = signal_str.rfind(')').unwrap_or(signal_str.len());
        let args_str = &signal_str[paren_start + 1..paren_end];

        if args_str.trim().is_empty() {
            (name, vec![])
        } else {
            let args: Vec<String> = args_str
                .split(',')
                .map(|a| a.trim().to_string())
                .collect();
            (name, args)
        }
    } else {
        (signal_str.trim().to_string(), vec![])
    }
}

// ======================
// Mutation Resolvers
// ======================

/// Validate a mutation plan
pub fn validate_mutation(
    ctx: &GqlContext,
    input: &MutationPlanInput,
) -> MutationValidationResult {
    let start = Instant::now();
    let mut errors = Vec::new();
    let warnings = Vec::new();

    for (index, op) in input.operations.iter().enumerate() {
        let op_errors = validate_operation(ctx, index as i32, op);
        errors.extend(op_errors);
    }

    let validation_time_ms = start.elapsed().as_millis() as i32;

    MutationValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        validation_time_ms,
    }
}

/// Validate a single operation
fn validate_operation(
    _ctx: &GqlContext,
    index: i32,
    op: &PlannedOperation,
) -> Vec<MutationValidationError> {
    let mut errors = Vec::new();
    let args = &op.args.0;

    match op.operation_type {
        OperationType::SetProperty => {
            // Check required args
            if args.get("nodePath").is_none() {
                errors.push(MutationValidationError {
                    operation_index: index,
                    code: "MISSING_REQUIRED_ARG".to_string(),
                    message: "Missing required argument: nodePath".to_string(),
                    suggestion: None,
                });
            }
            if args.get("property").is_none() {
                errors.push(MutationValidationError {
                    operation_index: index,
                    code: "MISSING_REQUIRED_ARG".to_string(),
                    message: "Missing required argument: property".to_string(),
                    suggestion: None,
                });
            }
            if args.get("value").is_none() {
                errors.push(MutationValidationError {
                    operation_index: index,
                    code: "MISSING_REQUIRED_ARG".to_string(),
                    message: "Missing required argument: value".to_string(),
                    suggestion: None,
                });
            }

            // Check if node path exists (skip for "." which always exists)
            if let Some(node_path) = args.get("nodePath").and_then(|v| v.as_str()) {
                if node_path != "." {
                    // For now, assume non-root nodes might not exist
                    // In real implementation, we would check against the scene tree
                    errors.push(MutationValidationError {
                        operation_index: index,
                        code: "NODE_NOT_FOUND".to_string(),
                        message: format!("Node not found: {}", node_path),
                        suggestion: Some("Check the node path is correct".to_string()),
                    });
                }
            }
        }
        OperationType::AddNode => {
            // Check required args
            if args.get("parent").is_none() {
                errors.push(MutationValidationError {
                    operation_index: index,
                    code: "MISSING_REQUIRED_ARG".to_string(),
                    message: "Missing required argument: parent".to_string(),
                    suggestion: None,
                });
            }
            if args.get("name").is_none() {
                errors.push(MutationValidationError {
                    operation_index: index,
                    code: "MISSING_REQUIRED_ARG".to_string(),
                    message: "Missing required argument: name".to_string(),
                    suggestion: None,
                });
            }
            if args.get("type").is_none() {
                errors.push(MutationValidationError {
                    operation_index: index,
                    code: "MISSING_REQUIRED_ARG".to_string(),
                    message: "Missing required argument: type".to_string(),
                    suggestion: None,
                });
            }
        }
        OperationType::RemoveNode => {
            // Check required args
            if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                if path == "." {
                    errors.push(MutationValidationError {
                        operation_index: index,
                        code: "CANNOT_REMOVE_ROOT".to_string(),
                        message: "Cannot remove root node".to_string(),
                        suggestion: None,
                    });
                }
            } else {
                errors.push(MutationValidationError {
                    operation_index: index,
                    code: "MISSING_REQUIRED_ARG".to_string(),
                    message: "Missing required argument: path".to_string(),
                    suggestion: None,
                });
            }
        }
        _ => {
            // Other operations - basic validation only
        }
    }

    errors
}

/// Preview a mutation (generate diff)
pub fn preview_mutation(
    _ctx: &GqlContext,
    input: &MutationPlanInput,
) -> PreviewResult {
    let mut nodes_added = 0;
    let mut nodes_removed = 0;
    let mut properties_changed = 0;
    let mut signals_connected = 0;
    let mut diff_lines = Vec::new();
    let affected_files = Vec::new();

    for op in &input.operations {
        let args = &op.args.0;

        match op.operation_type {
            OperationType::SetProperty => {
                properties_changed += 1;
                if let (Some(node_path), Some(property), Some(value)) = (
                    args.get("nodePath").and_then(|v| v.as_str()),
                    args.get("property").and_then(|v| v.as_str()),
                    args.get("value").and_then(|v| v.as_str()),
                ) {
                    diff_lines.push(format!("+ {}:{} = {}", node_path, property, value));
                }
            }
            OperationType::AddNode => {
                nodes_added += 1;
                if let (Some(parent), Some(name), Some(node_type)) = (
                    args.get("parent").and_then(|v| v.as_str()),
                    args.get("name").and_then(|v| v.as_str()),
                    args.get("type").and_then(|v| v.as_str()),
                ) {
                    diff_lines.push(format!("+ [node name=\"{}\" type=\"{}\" parent=\"{}\"]", name, node_type, parent));
                }
            }
            OperationType::RemoveNode => {
                nodes_removed += 1;
                if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                    diff_lines.push(format!("- [node at \"{}\"]", path));
                }
            }
            OperationType::ConnectSignal => {
                signals_connected += 1;
            }
            _ => {}
        }
    }

    // For now, we don't know the actual affected files without context
    // In a real implementation, we'd analyze which scenes are affected

    PreviewResult {
        success: true,
        diff: diff_lines.join("\n"),
        affected_files,
        summary: ChangeSummary {
            nodes_added,
            nodes_removed,
            properties_changed,
            signals_connected,
        },
    }
}

/// Apply a mutation
pub fn apply_mutation(
    _ctx: &GqlContext,
    input: &ApplyMutationInput,
) -> ApplyResult {
    let mut applied_count = 0;
    let mut errors = Vec::new();

    // Generate undo action ID using system time
    let undo_action_id = if !input.operations.is_empty() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Some(format!("undo_{}", timestamp))
    } else {
        None
    };

    // Process each operation
    for (index, op) in input.operations.iter().enumerate() {
        // In a real implementation, we would:
        // 1. Load the affected scene file
        // 2. Apply the modification
        // 3. Save the file (or accumulate changes for atomic write)

        // For now, just count as applied
        applied_count += 1;
    }

    ApplyResult {
        success: errors.is_empty(),
        applied_count,
        backup_path: if input.create_backup.unwrap_or(false) {
            Some("backup/".to_string())
        } else {
            None
        },
        errors,
        undo_action_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_to_res_path() {
        let root = PathBuf::from("/project");
        let file = PathBuf::from("/project/scenes/player.tscn");
        assert_eq!(to_res_path(&root, &file), "res://scenes/player.tscn");
    }

    #[test]
    fn test_parse_signal_definition() {
        let (name, args) = parse_signal_definition("health_changed(new_value: int)");
        assert_eq!(name, "health_changed");
        assert_eq!(args, vec!["new_value: int"]);

        let (name2, args2) = parse_signal_definition("simple_signal");
        assert_eq!(name2, "simple_signal");
        assert!(args2.is_empty());
    }
}
