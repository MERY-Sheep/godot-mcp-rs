//! Test Resolver
//!
//! Handles GdUnit4 test execution and result parsing.

use super::context::GqlContext;
use super::types::*;

/// Run GdUnit4 tests and return structured results
pub async fn resolve_run_tests(ctx: &GqlContext, input: &RunTestsInput) -> TestExecutionResult {
    let project_path = &ctx.project_path;
    let test_path = input.test_path.as_deref().unwrap_or("res://tests/");

    // Determine Godot executable path
    // 1. Check environment variable GODOT_BIN
    // 2. Default to "godot"
    let godot_bin = std::env::var("GODOT_BIN").unwrap_or_else(|_| "godot".to_string());

    // Execute GdUnit4 CLI
    // For now, we use a simple command-line invocation.
    // In a full implementation, we would use the GdUnit4 CLI tool or the specific addon script.

    let mut command = std::process::Command::new(&godot_bin);
    command.arg("--headless");
    command.arg("--path");
    command.arg(project_path);
    command.arg("-s");
    command.arg("res://addons/gdUnit4/bin/GdUnitCopyAndRun.gd");
    command.arg("--add");
    command.arg(test_path);

    // Additional args for GdUnit4
    // command.arg("--continue"); // Don't stop on first failure

    let output = match command.output() {
        Ok(out) => out,
        Err(_e) => {
            return TestExecutionResult {
                success: false,
                total_count: 0,
                passed_count: 0,
                failed_count: 0,
                error_count: 1,
                skipped_count: 0,
                duration_ms: 0,
                suites: vec![],
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse the output
    // For Phase 1, we'll implement a basic parser that looks for the summary line.
    // Future phases will use JUnit XML for more detail.
    parse_test_output(&stdout, &stderr)
}

/// Simple parser for GdUnit4 output
pub fn parse_test_output(stdout: &str, _stderr: &str) -> TestExecutionResult {
    let mut total_count = 0;
    let mut passed_count = 0;
    let mut failed_count = 0;
    let mut error_count = 0;
    let mut skipped_count = 0;

    // Look for a line like: "Total: 10, Passed: 8, Failed: 1, Errors: 1, Skipped: 0"
    // GdUnit4 output format varies, so this is a placeholder-level implementation.
    for line in stdout.lines() {
        if line.contains("Total:") && line.contains("Passed:") {
            // Regex would be safer, but let's do a simple count for now if possible
            // Example: "GdUnit4 Test Summary: Total: 3, Passed: 2, Failed: 1, Errors: 0, Skipped: 0, Duration: 123ms"
            if let Some(total) = extract_value(line, "Total:") {
                total_count = total;
            }
            if let Some(passed) = extract_value(line, "Passed:") {
                passed_count = passed;
            }
            if let Some(failed) = extract_value(line, "Failed:") {
                failed_count = failed;
            }
            if let Some(errors) = extract_value(line, "Errors:") {
                error_count = errors;
            }
            if let Some(skipped) = extract_value(line, "Skipped:") {
                skipped_count = skipped;
            }
        }
    }

    TestExecutionResult {
        success: failed_count == 0 && error_count == 0 && total_count > 0,
        total_count,
        passed_count,
        failed_count,
        error_count,
        skipped_count,
        duration_ms: 0, // TODO: Extract duration
        suites: vec![], // TODO: Parse individual suites
    }
}

fn extract_value(line: &str, key: &str) -> Option<i32> {
    let start = line.find(key)? + key.len();
    let rest = &line[start..];
    let end = rest.find(',').unwrap_or(rest.len());
    rest[..end].trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gdunit4_output() {
        let stdout = "GdUnit4 Test Summary: Total: 5, Passed: 3, Failed: 1, Errors: 1, Skipped: 0, Duration: 456ms";
        let result = parse_test_output(stdout, "");

        assert_eq!(result.total_count, 5);
        assert_eq!(result.passed_count, 3);
        assert_eq!(result.failed_count, 1);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.skipped_count, 0);
        assert!(!result.success);
    }
}
