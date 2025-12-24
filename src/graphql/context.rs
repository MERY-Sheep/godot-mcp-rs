//! GraphQL Context
//!
//! Provides context data (e.g., project path) to resolvers.

use std::path::PathBuf;

/// Context for GraphQL resolvers
#[derive(Debug, Clone)]
pub struct GqlContext {
    /// Path to the Godot project directory
    pub project_path: PathBuf,
    /// Port for Godot editor plugin HTTP API (default: 6060)
    pub godot_port: u16,
    /// HTTP request timeout in milliseconds (default: 5000)
    pub timeout_ms: u64,
}

impl GqlContext {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            godot_port: 6060,
            timeout_ms: 5000,
        }
    }

    /// Create context with custom port
    pub fn with_port(mut self, port: u16) -> Self {
        self.godot_port = port;
        self
    }

    /// Create context with custom timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}
