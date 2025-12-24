//! Refactoring Resolver
//!
//! Handles code understanding and refactoring operations.

use std::fs;
use std::path::Path;

use crate::godot::gdscript::GDScript;

use super::context::GqlContext;
use super::project_resolver::{collect_project_files, to_res_path};
use super::script_resolver::res_path_to_fs_path;
use super::types::*;

/// Get class hierarchy for a script
pub fn resolve_class_hierarchy(ctx: &GqlContext, script_path: &str) -> ClassHierarchy {
    let file_path = res_path_to_fs_path(&ctx.project_path, script_path);

    let mut extends_chain = Vec::new();
    let mut class_name = None;
    let mut current_path = Some(file_path.clone());
    let mut depth = 0;

    // Follow extends chain
    while let Some(ref path) = current_path {
        if let Ok(content) = fs::read_to_string(path) {
            let script = GDScript::parse(&content);

            // Get class_name from first script
            if depth == 0 {
                class_name = script.class_name.clone();
            }

            if let Some(ref extends) = script.extends {
                // Check if it's a file path or a class name
                if extends.starts_with("res://") || extends.ends_with(".gd") {
                    // It's a script path
                    let ext_path = if extends.starts_with("res://") {
                        res_path_to_fs_path(&ctx.project_path, extends)
                    } else {
                        path.parent().unwrap_or(Path::new(".")).join(extends)
                    };

                    extends_chain.push(ClassInfo {
                        name: extends.clone(),
                        script_path: Some(to_res_path(&ctx.project_path, &ext_path)),
                        is_builtin: false,
                    });

                    current_path = Some(ext_path);
                } else {
                    // It's a built-in class name
                    extends_chain.push(ClassInfo {
                        name: extends.clone(),
                        script_path: None,
                        is_builtin: true,
                    });
                    current_path = None;
                }
            } else {
                current_path = None;
            }
        } else {
            current_path = None;
        }

        depth += 1;
        // Safety limit
        if depth > 50 {
            break;
        }
    }

    ClassHierarchy {
        script_path: script_path.to_string(),
        class_name,
        extends_chain,
        depth,
    }
}

/// Find references to a symbol across the project
pub fn resolve_find_references(
    ctx: &GqlContext,
    symbol: &str,
    scope: Option<&str>,
) -> SymbolReferences {
    let mut references = Vec::new();
    let mut definition = None;

    // Collect all scripts
    let (_, scripts) = collect_project_files(&ctx.project_path);

    // Search pattern - word boundary match
    let pattern = format!(r"\b{}\b", regex::escape(symbol));
    let regex = regex::Regex::new(&pattern).unwrap_or_else(|_| regex::Regex::new(symbol).unwrap());

    for script_file in &scripts {
        // Apply scope filter if provided
        if let Some(scope_path) = scope {
            if !script_file.path.starts_with(scope_path) {
                continue;
            }
        }

        let file_path = res_path_to_fs_path(&ctx.project_path, &script_file.path);
        if let Ok(content) = fs::read_to_string(&file_path) {
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    let line_num = (line_num + 1) as i32;

                    // Check if this is a definition (var, func, signal, class_name)
                    let trimmed = line.trim();
                    let is_definition = trimmed.starts_with("func ")
                        || trimmed.starts_with("var ")
                        || trimmed.starts_with("signal ")
                        || trimmed.starts_with("class_name ")
                        || trimmed.starts_with("@export var ");

                    let location = SymbolLocation {
                        file: script_file.path.clone(),
                        line: line_num,
                        column: line.find(symbol).map(|c| c as i32),
                        context: Some(trimmed.to_string()),
                    };

                    if is_definition && definition.is_none() {
                        definition = Some(location);
                    } else {
                        references.push(location);
                    }
                }
            }
        }
    }

    let total_count = references.len() as i32 + if definition.is_some() { 1 } else { 0 };

    SymbolReferences {
        symbol: symbol.to_string(),
        definition,
        references,
        total_count,
    }
}

/// Get autoloads from project.godot
pub fn resolve_autoloads(ctx: &GqlContext) -> AutoloadsResult {
    let project_godot = ctx.project_path.join("project.godot");
    let mut autoloads = Vec::new();

    if let Ok(content) = fs::read_to_string(&project_godot) {
        let mut in_autoload_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for section headers
            if trimmed.starts_with('[') {
                in_autoload_section = trimmed == "[autoload]";
                continue;
            }

            // Parse autoload entries
            if in_autoload_section && trimmed.contains('=') {
                if let Some((name, value)) = trimmed.split_once('=') {
                    let name = name.trim().to_string();
                    let value = value.trim().trim_matches('"');

                    // Check for singleton marker (*)
                    let (is_singleton, path) = if value.starts_with('*') {
                        (true, value[1..].to_string())
                    } else {
                        (false, value.to_string())
                    };

                    autoloads.push(AutoloadEntry {
                        name,
                        path,
                        is_singleton,
                    });
                }
            }
        }
    }

    let count = autoloads.len() as i32;
    AutoloadsResult { autoloads, count }
}

/// Rename a symbol across the project
pub fn resolve_rename_symbol(ctx: &GqlContext, input: &RenameSymbolInput) -> RenameSymbolResult {
    let mut files_changed = Vec::new();
    let mut total_occurrences = 0;

    // Collect all scripts
    let (_, scripts) = collect_project_files(&ctx.project_path);

    // Search pattern - word boundary match
    let pattern = format!(r"\b{}\b", regex::escape(&input.symbol));
    let regex = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(e) => {
            return RenameSymbolResult {
                success: false,
                old_name: input.symbol.clone(),
                new_name: input.new_name.clone(),
                files_changed: vec![],
                occurrences_replaced: 0,
                message: Some(format!("Invalid symbol pattern: {}", e)),
            };
        }
    };

    for script_file in &scripts {
        // Apply scope filter if provided
        if let Some(ref scope) = input.scope {
            if !script_file.path.starts_with(scope) {
                continue;
            }
        }

        let file_path = res_path_to_fs_path(&ctx.project_path, &script_file.path);
        if let Ok(content) = fs::read_to_string(&file_path) {
            let new_content = regex.replace_all(&content, input.new_name.as_str());

            if new_content != content {
                let count = regex.find_iter(&content).count();
                total_occurrences += count as i32;

                // Write the modified content
                if let Err(e) = fs::write(&file_path, new_content.as_ref()) {
                    return RenameSymbolResult {
                        success: false,
                        old_name: input.symbol.clone(),
                        new_name: input.new_name.clone(),
                        files_changed,
                        occurrences_replaced: total_occurrences,
                        message: Some(format!("Failed to write {}: {}", script_file.path, e)),
                    };
                }

                files_changed.push(FileChange {
                    path: script_file.path.clone(),
                    changes_count: count as i32,
                });
            }
        }
    }

    RenameSymbolResult {
        success: true,
        old_name: input.symbol.clone(),
        new_name: input.new_name.clone(),
        files_changed,
        occurrences_replaced: total_occurrences,
        message: None,
    }
}

/// Extract code block into a new function
pub fn resolve_extract_function(
    ctx: &GqlContext,
    input: &ExtractFunctionInput,
) -> ExtractFunctionResult {
    let file_path = res_path_to_fs_path(&ctx.project_path, &input.script_path);

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            return ExtractFunctionResult {
                success: false,
                function_name: input.function_name.clone(),
                script_path: input.script_path.clone(),
                message: Some(format!("Failed to read script: {}", e)),
            };
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    // Validate line range
    let start = (input.start_line - 1) as usize;
    let end = input.end_line as usize;

    if start >= lines.len() || end > lines.len() || start >= end {
        return ExtractFunctionResult {
            success: false,
            function_name: input.function_name.clone(),
            script_path: input.script_path.clone(),
            message: Some("Invalid line range".to_string()),
        };
    }

    // Extract the code block
    let extracted_lines: Vec<&str> = lines[start..end].to_vec();

    // Determine base indentation
    let base_indent = extracted_lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    // Build function body
    let function_body: Vec<String> = extracted_lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                "\t".to_string()
            } else if l.len() >= base_indent {
                format!("\t{}", &l[base_indent..])
            } else {
                format!("\t{}", l.trim())
            }
        })
        .collect();

    // Build parameters
    let params = input.parameters.clone().unwrap_or_default().join(", ");

    // Build new function
    let new_function = format!(
        "\n\nfunc {}({}) -> void:\n{}",
        input.function_name,
        params,
        function_body.join("\n")
    );

    // Build function call
    let call_indent = " ".repeat(base_indent);
    let call_params = input.parameters.clone().unwrap_or_default().join(", ");
    let function_call = format!("{}{}({})", call_indent, input.function_name, call_params);

    // Build new content
    let mut new_lines: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == start {
            new_lines.push(function_call.clone());
        } else if i < start || i >= end {
            new_lines.push(line.to_string());
        }
    }

    // Append new function at end
    let mut new_content = new_lines.join("\n");
    new_content.push_str(&new_function);

    // Write back
    if let Err(e) = fs::write(&file_path, &new_content) {
        return ExtractFunctionResult {
            success: false,
            function_name: input.function_name.clone(),
            script_path: input.script_path.clone(),
            message: Some(format!("Failed to write script: {}", e)),
        };
    }

    ExtractFunctionResult {
        success: true,
        function_name: input.function_name.clone(),
        script_path: input.script_path.clone(),
        message: Some(format!(
            "Extracted lines {}-{} to function {}",
            input.start_line, input.end_line, input.function_name
        )),
    }
}
