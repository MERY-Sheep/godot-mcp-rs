//! GraphQL Context
//!
//! Provides context data (e.g., project path) to resolvers.

use std::path::PathBuf;

/// Context for GraphQL resolvers
#[derive(Debug, Clone)]
pub struct GqlContext {
    /// Path to the Godot project directory
    pub project_path: PathBuf,
}

impl GqlContext {
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }
}
