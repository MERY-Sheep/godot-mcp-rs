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
