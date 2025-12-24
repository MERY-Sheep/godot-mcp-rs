//! Mutation Resolver
//!
//! Handles mutation validation, preview, and application.

use std::time::Instant;

use super::context::GqlContext;
use super::types::*;

/// Validate a mutation plan
pub fn validate_mutation(ctx: &GqlContext, input: &MutationPlanInput) -> MutationValidationResult {
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
pub fn preview_mutation(_ctx: &GqlContext, input: &MutationPlanInput) -> PreviewResult {
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
                    diff_lines.push(format!(
                        "+ [node name=\"{}\" type=\"{}\" parent=\"{}\"]",
                        name, node_type, parent
                    ));
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
pub fn apply_mutation(_ctx: &GqlContext, input: &ApplyMutationInput) -> ApplyResult {
    let mut applied_count = 0;
    let errors: Vec<ApplyError> = Vec::new();

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
    for (_index, _op) in input.operations.iter().enumerate() {
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
