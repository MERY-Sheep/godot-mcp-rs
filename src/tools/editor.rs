//! Editor control tools - Godot execution and debugging

use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{
    GetDebugOutputRequest, GetGodotVersionRequest, GetRunningStatusRequest, GodotTools,
    LaunchEditorRequest, RunProjectRequest, StopProjectRequest,
};

/// Get the PID file path
fn get_pid_file_path(project_root: &std::path::Path) -> PathBuf {
    project_root.join(".godot_mcp_pid")
}

impl GodotTools {
    /// Resolve the path to the Godot executable
    fn resolve_godot_path(&self) -> Result<PathBuf, McpError> {
        // 1. If explicitly set
        if let Some(ref path) = self.godot_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // 2. Environment variable GODOT_PATH
        if let Ok(env_path) = std::env::var("GODOT_PATH") {
            let path = PathBuf::from(&env_path);
            if path.exists() {
                return Ok(path);
            }
        }

        // 3. Search in PATH (Windows)
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("where").arg("godot").output() {
                if output.status.success() {
                    if let Ok(path_str) = String::from_utf8(output.stdout) {
                        if let Some(first_line) = path_str.lines().next() {
                            let path = PathBuf::from(first_line.trim());
                            if path.exists() {
                                return Ok(path);
                            }
                        }
                    }
                }
            }
            // Also try godot4
            if let Ok(output) = Command::new("where").arg("godot4").output() {
                if output.status.success() {
                    if let Ok(path_str) = String::from_utf8(output.stdout) {
                        if let Some(first_line) = path_str.lines().next() {
                            let path = PathBuf::from(first_line.trim());
                            if path.exists() {
                                return Ok(path);
                            }
                        }
                    }
                }
            }
        }

        // 4. Default paths (Windows)
        #[cfg(windows)]
        {
            let default_paths = [
                r"C:\Program Files\Godot\Godot.exe",
                r"C:\Program Files (x86)\Godot\Godot.exe",
                r"C:\Godot\Godot.exe",
            ];
            for path_str in &default_paths {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        Err(McpError::internal_error(
            "Godot executable not found. Set GODOT_PATH environment variable or add Godot to PATH."
                .to_string(),
            None,
        ))
    }

    /// get_godot_version - Get Godot version
    pub async fn handle_get_godot_version(
        &self,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let _req: GetGodotVersionRequest = Default::default();

        let godot_path = self.resolve_godot_path()?;

        let output = Command::new(&godot_path)
            .arg("--version")
            .output()
            .map_err(|e| McpError::internal_error(format!("Failed to run Godot: {}", e), None))?;

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let result = serde_json::json!({
            "version": version,
            "path": godot_path.to_string_lossy(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// run_project - Run project
    pub async fn handle_run_project(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: RunProjectRequest = if let Some(a) = args {
            serde_json::from_value(serde_json::Value::Object(a))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?
        } else {
            RunProjectRequest::default()
        };

        let godot_path = self.resolve_godot_path()?;
        let project_root = self.get_base_path();
        let pid_file = get_pid_file_path(project_root);

        // If already running, stop it first (auto-restart behavior)
        let mut stopped_previous = false;
        if pid_file.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    // Check if process still exists
                    #[cfg(windows)]
                    {
                        let check = Command::new("tasklist")
                            .args(["/FI", &format!("PID eq {}", pid)])
                            .output();
                        if let Ok(out) = check {
                            let output_str = String::from_utf8_lossy(&out.stdout);
                            if output_str.contains(&pid.to_string()) {
                                // Stop the running project automatically
                                let _ = Command::new("taskkill")
                                    .args(["/PID", &pid.to_string(), "/F"])
                                    .output();
                                stopped_previous = true;
                                // Brief wait for process to terminate
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                        }
                    }
                    // Clean up PID file
                    fs::remove_file(&pid_file).ok();
                }
            }
        }

        let mut cmd = Command::new(&godot_path);
        cmd.arg("--path").arg(project_root);

        if let Some(ref scene) = req.scene {
            let scene_path = scene.strip_prefix("res://").unwrap_or(scene);
            cmd.arg(scene_path);
        }

        cmd.arg("--debug");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| McpError::internal_error(format!("Failed to start Godot: {}", e), None))?;

        let pid = child.id();

        // Save PID to file
        fs::write(&pid_file, pid.to_string())
            .map_err(|e| McpError::internal_error(format!("Failed to save PID: {}", e), None))?;

        // Create output capture file
        let output_file = project_root.join(".godot_mcp_output");
        fs::write(&output_file, "").ok();

        // Start thread to asynchronously write output to file
        let output_file_clone = output_file.clone();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        std::thread::spawn(move || {
            let mut all_output = String::new();

            if let Some(stdout) = stdout {
                let reader = BufReader::new(stdout);
                for line in reader.lines().filter_map(|l| l.ok()) {
                    all_output.push_str(&line);
                    all_output.push('\n');
                    fs::write(&output_file_clone, &all_output).ok();
                }
            }
            if let Some(stderr) = stderr {
                let reader = BufReader::new(stderr);
                for line in reader.lines().filter_map(|l| l.ok()) {
                    all_output.push_str("ERROR: ");
                    all_output.push_str(&line);
                    all_output.push('\n');
                    fs::write(&output_file_clone, &all_output).ok();
                }
            }
        });

        let message = if stopped_previous {
            format!("Stopped previous instance and started project{}", req.scene.as_ref().map(|s| format!(" with scene: {}", s)).unwrap_or_default())
        } else {
            format!("Project started{}", req.scene.as_ref().map(|s| format!(" with scene: {}", s)).unwrap_or_default())
        };

        let result = serde_json::json!({
            "success": true,
            "pid": pid,
            "restarted": stopped_previous,
            "message": message,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// stop_project - Stop project
    pub async fn handle_stop_project(
        &self,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let _req: StopProjectRequest = Default::default();

        let project_root = self.get_base_path();
        let pid_file = get_pid_file_path(project_root);

        if !pid_file.exists() {
            return Err(McpError::internal_error(
                "No running project found".to_string(),
                None,
            ));
        }

        let pid_str = fs::read_to_string(&pid_file).map_err(|e| {
            McpError::internal_error(format!("Failed to read PID file: {}", e), None)
        })?;

        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|_| McpError::internal_error("Invalid PID in file".to_string(), None))?;

        // Terminate process
        #[cfg(windows)]
        {
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output()
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to kill process: {}", e), None)
                })?;
        }

        // Delete PID file
        fs::remove_file(&pid_file).ok();

        let result = serde_json::json!({
            "success": true,
            "pid": pid,
            "message": "Project stopped",
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// get_debug_output - Get debug output
    pub async fn handle_get_debug_output(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: GetDebugOutputRequest = if let Some(a) = args {
            serde_json::from_value(serde_json::Value::Object(a))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?
        } else {
            GetDebugOutputRequest::default()
        };

        let project_root = self.get_base_path();
        let output_file = project_root.join(".godot_mcp_output");
        let pid_file = get_pid_file_path(project_root);

        let running = pid_file.exists();

        let output = if output_file.exists() {
            fs::read_to_string(&output_file).unwrap_or_default()
        } else {
            String::new()
        };

        let lines: Vec<&str> = output.lines().collect();
        let total_lines = lines.len();
        let max_lines = req.lines.unwrap_or(100) as usize;
        let display_lines: Vec<&str> = if lines.len() > max_lines {
            lines[lines.len() - max_lines..].to_vec()
        } else {
            lines
        };

        let error_count = display_lines
            .iter()
            .filter(|l| l.contains("ERROR") || l.contains("error"))
            .count();

        let result = serde_json::json!({
            "running": running,
            "output": display_lines.join("\n"),
            "total_lines": total_lines,
            "error_count": error_count,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// launch_editor - Launch editor
    pub async fn handle_launch_editor(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LaunchEditorRequest = if let Some(a) = args {
            serde_json::from_value(serde_json::Value::Object(a))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?
        } else {
            LaunchEditorRequest::default()
        };

        let godot_path = self.resolve_godot_path()?;
        let project_root = self.get_base_path();

        let mut cmd = Command::new(&godot_path);
        cmd.arg("--path").arg(project_root);
        cmd.arg("--editor");

        if let Some(ref scene) = req.scene {
            let scene_path = scene.strip_prefix("res://").unwrap_or(scene);
            cmd.arg(scene_path);
        }

        let child = cmd.spawn().map_err(|e| {
            McpError::internal_error(format!("Failed to launch editor: {}", e), None)
        })?;

        let result = serde_json::json!({
            "success": true,
            "pid": child.id(),
            "message": "Editor launched",
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// get_running_status - Get running status
    pub async fn handle_get_running_status(
        &self,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let _req: GetRunningStatusRequest = Default::default();

        let project_root = self.get_base_path();
        let pid_file = get_pid_file_path(project_root);

        if !pid_file.exists() {
            let result = serde_json::json!({
                "running": false,
                "message": "No project is running",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]));
        }

        let pid_str = fs::read_to_string(&pid_file).unwrap_or_default();
        let pid: u32 = pid_str.trim().parse().unwrap_or(0);

        // Check if process still exists
        let mut still_running = false;
        #[cfg(windows)]
        {
            if let Ok(out) = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid)])
                .output()
            {
                let output_str = String::from_utf8_lossy(&out.stdout);
                still_running = output_str.contains(&pid.to_string());
            }
        }

        if !still_running {
            fs::remove_file(&pid_file).ok();
        }

        let result = serde_json::json!({
            "running": still_running,
            "pid": if still_running { Some(pid) } else { None },
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }
}
