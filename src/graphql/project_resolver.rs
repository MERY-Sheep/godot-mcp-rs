//! Project Resolver
//!
//! Handles project information, file collection, and validation.

use std::fs;
use std::path::Path;

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
pub fn parse_project_name(path: &Path) -> Option<String> {
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
pub fn collect_project_files(project_path: &Path) -> (Vec<SceneFile>, Vec<ScriptFile>) {
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
pub fn count_resources(project_path: &Path) -> i32 {
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
pub fn to_res_path(project_root: &Path, file_path: &Path) -> String {
    let relative = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");
    format!("res://{}", relative)
}

/// Basic project validation
pub fn validate_project(
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

// ======================
// Phase 2.2: Input Action & Project Settings
// ======================

/// Add an input action to the project's InputMap
/// This modifies the project.godot file's [input] section
pub fn resolve_add_input_action(ctx: &GqlContext, input: &AddInputActionInput) -> OperationResult {
    // Validate action name
    if input.action_name.is_empty() {
        return OperationResult::err_msg("Action name cannot be empty");
    }

    let project_godot = ctx.project_path.join("project.godot");
    if !project_godot.exists() {
        return OperationResult::err_msg("project.godot not found");
    }

    // Read current project.godot
    let content = match fs::read_to_string(&project_godot) {
        Ok(c) => c,
        Err(e) => return OperationResult::err_msg(format!("Failed to read project.godot: {}", e)),
    };

    // Build input action entry
    let action_key = format!("input/{}", input.action_name);

    // Check if action already exists
    if content
        .lines()
        .any(|line| line.starts_with(&format!("{}=", action_key)))
    {
        return OperationResult::err_msg(format!(
            "Input action '{}' already exists",
            input.action_name
        ));
    }

    // Build events string for Godot 4.x format
    let events: Vec<String> = input.events.iter().map(|e| format_input_event(e)).collect();

    let action_value = format!(
        r#"{{
"deadzone": 0.5,
"events": [{}]
}}"#,
        events.join(", ")
    );

    // Find [input] section or create it
    let mut new_content = String::new();
    let mut found_input_section = false;
    let mut in_input_section = false;
    let mut inserted = false;

    for line in content.lines() {
        if line.trim() == "[input]" {
            found_input_section = true;
            in_input_section = true;
            new_content.push_str(line);
            new_content.push('\n');
            continue;
        }

        if in_input_section && line.starts_with('[') && line != "[input]" {
            // End of input section, insert new action before next section
            if !inserted {
                new_content.push_str(&format!("{}={}\n", action_key, action_value));
                inserted = true;
            }
            in_input_section = false;
        }

        new_content.push_str(line);
        new_content.push('\n');
    }

    // If no [input] section found, add it at the end
    if !found_input_section {
        new_content.push_str("\n[input]\n");
        new_content.push_str(&format!("{}={}\n", action_key, action_value));
    } else if !inserted {
        // [input] was the last section
        new_content.push_str(&format!("{}={}\n", action_key, action_value));
    }

    // Write back
    if let Err(e) = fs::write(&project_godot, new_content) {
        return OperationResult::err_msg(format!("Failed to write project.godot: {}", e));
    }

    OperationResult::ok()
}

/// Format a single input event for project.godot
fn format_input_event(event: &InputEventInput) -> String {
    match event.event_type {
        InputEventType::Key => {
            let key = event.key.as_deref().unwrap_or("Unknown");
            format!(
                r#"Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":{},"key_label":0,"unicode":0,"echo":false)"#,
                key_name_to_godot_keycode(key)
            )
        }
        InputEventType::MouseButton => {
            let button = event.button.unwrap_or(1);
            format!(
                r#"Object(InputEventMouseButton,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"button_mask":0,"position":Vector2(0, 0),"global_position":Vector2(0, 0),"factor":1.0,"button_index":{},"canceled":false,"pressed":false,"double_click":false)"#,
                button
            )
        }
        InputEventType::JoyButton => {
            let button = event.button.unwrap_or(0);
            let device = event.device.unwrap_or(0);
            format!(
                r#"Object(InputEventJoypadButton,"resource_local_to_scene":false,"resource_name":"","device":{},"button_index":{},"pressure":0.0,"pressed":false)"#,
                device, button
            )
        }
        InputEventType::JoyAxis => {
            let button = event.button.unwrap_or(0);
            let device = event.device.unwrap_or(0);
            format!(
                r#"Object(InputEventJoypadMotion,"resource_local_to_scene":false,"resource_name":"","device":{},"axis":{},"axis_value":0.0)"#,
                device, button
            )
        }
    }
}

/// Convert key name to Godot keycode
fn key_name_to_godot_keycode(key: &str) -> i32 {
    match key.to_uppercase().as_str() {
        "SPACE" => 32,
        "A" => 65,
        "B" => 66,
        "C" => 67,
        "D" => 68,
        "E" => 69,
        "F" => 70,
        "W" => 87,
        "S" => 83,
        "Q" => 81,
        "LEFT" => 4194319,
        "RIGHT" => 4194321,
        "UP" => 4194320,
        "DOWN" => 4194322,
        "ENTER" | "RETURN" => 4194309,
        "ESCAPE" | "ESC" => 4194305,
        "TAB" => 4194306,
        "SHIFT" => 4194325,
        "CTRL" | "CONTROL" => 4194326,
        "ALT" => 4194328,
        _ => 0, // Unknown key
    }
}

/// Set a project setting in project.godot
pub fn resolve_set_project_setting(
    ctx: &GqlContext,
    input: &SetProjectSettingInput,
) -> OperationResult {
    // Validate path
    if input.path.is_empty() {
        return OperationResult::err_msg("Setting path cannot be empty");
    }

    let project_godot = ctx.project_path.join("project.godot");
    if !project_godot.exists() {
        return OperationResult::err_msg("project.godot not found");
    }

    // Read current project.godot
    let content = match fs::read_to_string(&project_godot) {
        Ok(c) => c,
        Err(e) => return OperationResult::err_msg(format!("Failed to read project.godot: {}", e)),
    };

    // Parse the path to find section and key
    let parts: Vec<&str> = input.path.split('/').collect();
    if parts.len() < 2 {
        return OperationResult::err_msg("Invalid setting path format (expected: section/key)");
    }

    let section = format!("[{}]", parts[0]);
    let key = parts[1..].join("/");

    let mut new_content = String::new();
    let mut in_target_section = false;
    let mut found_key = false;

    for line in content.lines() {
        // Check for section header
        if line.trim().starts_with('[') && line.trim().ends_with(']') {
            // If leaving target section without finding key, insert it
            if in_target_section && !found_key {
                new_content.push_str(&format!("{}={}\n", key, input.value));
                found_key = true;
            }
            in_target_section = line.trim() == section;
        }

        // Check for existing key in target section
        if in_target_section && line.starts_with(&format!("{}=", key)) {
            new_content.push_str(&format!("{}={}\n", key, input.value));
            found_key = true;
            continue;
        }

        new_content.push_str(line);
        new_content.push('\n');
    }

    // If section not found, add it
    if !found_key {
        if !content.contains(&section) {
            new_content.push_str(&format!("\n{}\n", section));
        }
        new_content.push_str(&format!("{}={}\n", key, input.value));
    }

    // Write back
    if let Err(e) = fs::write(&project_godot, new_content) {
        return OperationResult::err_msg(format!("Failed to write project.godot: {}", e));
    }

    OperationResult::ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_to_res_path() {
        let root = PathBuf::from("/project");
        let file = PathBuf::from("/project/scenes/main.tscn");
        assert_eq!(to_res_path(&root, &file), "res://scenes/main.tscn");
    }
}
