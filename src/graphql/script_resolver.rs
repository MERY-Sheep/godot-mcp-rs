//! Script Resolver
//!
//! Handles script parsing, conversion, and creation.

use std::fs;

use crate::godot::gdscript::GDScript;
use crate::path_utils;

use super::context::GqlContext;
use super::types::*;

/// Resolve script from file path
pub fn resolve_script(ctx: &GqlContext, res_path: &str) -> Option<Script> {
    let file_path = path_utils::to_fs_path_unchecked(&ctx.project_path, res_path);
    let content = fs::read_to_string(&file_path).ok()?;
    let gdscript = GDScript::parse(&content);

    Some(convert_gdscript_to_gql(&gdscript, res_path))
}

/// Convert GDScript to GraphQL Script
pub fn convert_gdscript_to_gql(script: &GDScript, path: &str) -> Script {
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
pub fn parse_signal_definition(signal_str: &str) -> (String, Vec<String>) {
    if let Some(paren_start) = signal_str.find('(') {
        let name = signal_str[..paren_start].trim().to_string();
        let paren_end = signal_str.rfind(')').unwrap_or(signal_str.len());
        let args_str = &signal_str[paren_start + 1..paren_end];

        if args_str.trim().is_empty() {
            (name, vec![])
        } else {
            let args: Vec<String> = args_str.split(',').map(|a| a.trim().to_string()).collect();
            (name, args)
        }
    } else {
        (signal_str.trim().to_string(), vec![])
    }
}

/// Create a new GDScript file
pub fn create_script(ctx: &GqlContext, input: &CreateScriptInput) -> ScriptResult {
    let project_path = &ctx.project_path;

    // Convert res:// path to filesystem path with validation
    let file_path = path_utils::to_fs_path_unchecked(project_path, &input.path);

    // Check if file already exists
    if file_path.exists() {
        return ScriptResult {
            success: false,
            script: None,
            message: Some(format!("Script already exists: {}", input.path)),
        };
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return ScriptResult {
                success: false,
                script: None,
                message: Some(format!("Failed to create directory: {}", e)),
            };
        }
    }

    // Generate GDScript content
    let class_name_line = input
        .class_name
        .as_ref()
        .map(|name| format!("class_name {}\n", name))
        .unwrap_or_default();

    let script_content = format!(
        r#"{}extends {}


func _ready() -> void:
	pass


func _process(delta: float) -> void:
	pass
"#,
        class_name_line, input.extends
    );

    // Write file
    if let Err(e) = fs::write(&file_path, script_content) {
        return ScriptResult {
            success: false,
            script: None,
            message: Some(format!("Failed to write script: {}", e)),
        };
    }

    ScriptResult {
        success: true,
        script: None, // Could load and return the script
        message: Some(format!("Created script: {}", input.path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_signal_definition() {
        let (name, args) = parse_signal_definition("health_changed(new_value: int)");
        assert_eq!(name, "health_changed");
        assert_eq!(args, vec!["new_value: int"]);

        let (name, args) = parse_signal_definition("simple_signal");
        assert_eq!(name, "simple_signal");
        assert!(args.is_empty());
    }
}
