//! Shader Resolver
//!
//! Handles shader validation (file-based, without Godot runtime).

use super::types::*;

/// Validate shader code syntax (basic validation without Godot)
pub fn resolve_validate_shader(input: &ValidateShaderInput) -> ShaderValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let lines: Vec<&str> = input.shader_code.lines().collect();

    // Check for shader_type declaration
    let shader_type_line = lines
        .iter()
        .enumerate()
        .find(|(_, l)| l.trim().starts_with("shader_type"));

    if shader_type_line.is_none() {
        errors.push(ShaderError {
            line: Some(1),
            column: None,
            message: "Missing shader_type declaration".to_string(),
        });
    }

    // Basic syntax checks
    let mut brace_count: i32 = 0;
    let mut paren_count: i32 = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_num = (i + 1) as i32;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Count braces
        for c in line.chars() {
            match c {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                _ => {}
            }
        }

        // Check for unclosed strings
        let quote_count = line.matches('"').count();
        if quote_count % 2 != 0 {
            errors.push(ShaderError {
                line: Some(line_num),
                column: None,
                message: "Unclosed string literal".to_string(),
            });
        }

        // Check for common mistakes
        if trimmed.ends_with(";;") {
            warnings.push(ShaderWarning {
                line: Some(line_num),
                message: "Double semicolon".to_string(),
            });
        }

        // Check uniform declarations
        if trimmed.starts_with("uniform") && !trimmed.contains(':') && !trimmed.contains('=') {
            warnings.push(ShaderWarning {
                line: Some(line_num),
                message: "Uniform missing type hint".to_string(),
            });
        }
    }

    // Check final brace balance
    if brace_count != 0 {
        errors.push(ShaderError {
            line: None,
            column: None,
            message: format!("Unbalanced braces: {} unclosed", brace_count.abs()),
        });
    }

    if paren_count != 0 {
        errors.push(ShaderError {
            line: None,
            column: None,
            message: format!("Unbalanced parentheses: {} unclosed", paren_count.abs()),
        });
    }

    ShaderValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}
