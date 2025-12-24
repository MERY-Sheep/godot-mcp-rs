//! Structured error handling for GraphQL operations
//!
//! Provides AI-friendly error types with detailed context for self-repair.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error categories for AI-friendly error handling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Connection errors (Godot plugin communication)
    Connection,
    /// Validation errors (input validation)
    Validation,
    /// Godot runtime errors
    Godot,
    /// File system errors
    FileSystem,
    /// Schema/Query errors
    Schema,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Connection => write!(f, "CONNECTION"),
            ErrorCategory::Validation => write!(f, "VALIDATION"),
            ErrorCategory::Godot => write!(f, "GODOT"),
            ErrorCategory::FileSystem => write!(f, "FILE_SYSTEM"),
            ErrorCategory::Schema => write!(f, "SCHEMA"),
        }
    }
}

/// Location information for errors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorLocation {
    /// File path (res:// or file://)
    pub file: Option<String>,
    /// Line number (1-indexed)
    pub line: Option<i32>,
    /// Column number (1-indexed)
    pub column: Option<i32>,
}

/// Stack frame for error traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStackFrame {
    /// Function or method name
    pub function: String,
    /// File path
    pub file: Option<String>,
    /// Line number
    pub line: Option<i32>,
}

/// Structured error with all context for AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredError {
    /// Error code (e.g., "CONN_TIMEOUT", "VALIDATION_NODE_NOT_FOUND")
    pub code: String,
    /// Category for grouping
    pub category: ErrorCategory,
    /// Human-readable message
    pub message: String,
    /// Location where error occurred
    pub location: Option<ErrorLocation>,
    /// Stack trace (if available)
    pub stack_trace: Vec<ErrorStackFrame>,
    /// Suggested fix for AI
    pub suggestion: Option<String>,
    /// Related documentation or help URL
    pub help_url: Option<String>,
    /// Additional context as key-value pairs
    pub context: HashMap<String, String>,
}

impl StructuredError {
    /// Create a new structured error
    pub fn new(
        code: impl Into<String>,
        category: ErrorCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            category,
            message: message.into(),
            location: None,
            stack_trace: vec![],
            suggestion: None,
            help_url: None,
            context: HashMap::new(),
        }
    }

    /// Add a suggestion for fixing the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add location information
    pub fn with_location(
        mut self,
        file: Option<String>,
        line: Option<i32>,
        column: Option<i32>,
    ) -> Self {
        self.location = Some(ErrorLocation { file, line, column });
        self
    }

    /// Add a context key-value pair
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Add a help URL
    pub fn with_help_url(mut self, url: impl Into<String>) -> Self {
        self.help_url = Some(url.into());
        self
    }

    /// Add a stack frame
    pub fn with_stack_frame(
        mut self,
        function: impl Into<String>,
        file: Option<String>,
        line: Option<i32>,
    ) -> Self {
        self.stack_trace.push(ErrorStackFrame {
            function: function.into(),
            file,
            line,
        });
        self
    }
}

impl std::fmt::Display for StructuredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

/// Main error type for GQL operations
#[derive(Debug, Error)]
pub enum GqlError {
    #[error("Connection error: {0}")]
    Connection(StructuredError),

    #[error("Validation error: {0}")]
    Validation(StructuredError),

    #[error("Godot error: {0}")]
    Godot(StructuredError),

    #[error("File system error: {0}")]
    FileSystem(StructuredError),

    #[error("Schema error: {0}")]
    Schema(StructuredError),
}

impl GqlError {
    /// Convert to StructuredError for GraphQL response
    pub fn to_structured(&self) -> StructuredError {
        match self {
            GqlError::Connection(e) => e.clone(),
            GqlError::Validation(e) => e.clone(),
            GqlError::Godot(e) => e.clone(),
            GqlError::FileSystem(e) => e.clone(),
            GqlError::Schema(e) => e.clone(),
        }
    }

    /// Get the error code
    pub fn code(&self) -> &str {
        match self {
            GqlError::Connection(e) => &e.code,
            GqlError::Validation(e) => &e.code,
            GqlError::Godot(e) => &e.code,
            GqlError::FileSystem(e) => &e.code,
            GqlError::Schema(e) => &e.code,
        }
    }
}

// ======================
// Convenience Constructors
// ======================

impl GqlError {
    /// Connection refused error
    pub fn connection_refused() -> Self {
        GqlError::Connection(
            StructuredError::new(
                "CONN_REFUSED",
                ErrorCategory::Connection,
                "Failed to connect to Godot editor plugin",
            )
            .with_suggestion("Godotエディターを起動し、MCPプラグインが有効か確認してください"),
        )
    }

    /// Connection timeout error
    pub fn connection_timeout() -> Self {
        GqlError::Connection(
            StructuredError::new(
                "CONN_TIMEOUT",
                ErrorCategory::Connection,
                "Request to Godot editor timed out",
            )
            .with_suggestion("Godotエディターが応答しているか確認してください"),
        )
    }

    /// HTTP error from Godot plugin
    pub fn http_error(status: u16, message: impl Into<String>) -> Self {
        GqlError::Godot(
            StructuredError::new(
                "GODOT_HTTP_ERROR",
                ErrorCategory::Godot,
                format!("HTTP error ({}): {}", status, message.into()),
            )
            .with_context("http_status", status.to_string()),
        )
    }

    /// Node not found error
    pub fn node_not_found(path: &str) -> Self {
        GqlError::Validation(
            StructuredError::new(
                "VALIDATION_NODE_NOT_FOUND",
                ErrorCategory::Validation,
                format!("Node not found: {}", path),
            )
            .with_suggestion("currentScene クエリで有効なノードパスを確認してください")
            .with_context("node_path", path),
        )
    }

    /// Invalid property error
    pub fn invalid_property(node_path: &str, property: &str) -> Self {
        GqlError::Validation(
            StructuredError::new(
                "VALIDATION_INVALID_PROPERTY",
                ErrorCategory::Validation,
                format!("Invalid property '{}' for node '{}'", property, node_path),
            )
            .with_suggestion("nodeTypeInfo クエリでノードの有効なプロパティを確認してください")
            .with_context("node_path", node_path)
            .with_context("property", property),
        )
    }

    /// Type mismatch error
    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        GqlError::Validation(
            StructuredError::new(
                "VALIDATION_TYPE_MISMATCH",
                ErrorCategory::Validation,
                format!("Type mismatch: expected '{}', got '{}'", expected, actual),
            )
            .with_context("expected_type", expected)
            .with_context("actual_type", actual),
        )
    }

    /// File not found error
    pub fn file_not_found(path: &str) -> Self {
        GqlError::FileSystem(
            StructuredError::new(
                "FILE_NOT_FOUND",
                ErrorCategory::FileSystem,
                format!("File not found: {}", path),
            )
            .with_suggestion("ファイルパスが正しいか確認してください")
            .with_context("file_path", path),
        )
    }

    /// File permission denied error
    pub fn permission_denied(path: &str) -> Self {
        GqlError::FileSystem(
            StructuredError::new(
                "FILE_PERMISSION_DENIED",
                ErrorCategory::FileSystem,
                format!("Permission denied: {}", path),
            )
            .with_suggestion("ファイルの読み書き権限を確認してください")
            .with_context("file_path", path),
        )
    }

    /// Generic Godot operation failed error
    pub fn godot_operation_failed(operation: &str, message: impl Into<String>) -> Self {
        GqlError::Godot(
            StructuredError::new("GODOT_OPERATION_FAILED", ErrorCategory::Godot, message)
                .with_suggestion("Godotのデバッグログを確認してください")
                .with_context("operation", operation),
        )
    }

    /// Scene not open error
    pub fn scene_not_open() -> Self {
        GqlError::Validation(
            StructuredError::new(
                "VALIDATION_SCENE_NOT_OPEN",
                ErrorCategory::Validation,
                "No scene is currently open in the editor",
            )
            .with_suggestion("openScene ミューテーションでシーンを開いてください"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_error_creation() {
        let err = StructuredError::new("TEST_ERROR", ErrorCategory::Validation, "Test message")
            .with_suggestion("Try this fix")
            .with_context("key", "value");

        assert_eq!(err.code, "TEST_ERROR");
        assert_eq!(err.category, ErrorCategory::Validation);
        assert_eq!(err.message, "Test message");
        assert_eq!(err.suggestion, Some("Try this fix".to_string()));
        assert_eq!(err.context.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_gql_error_to_structured() {
        let err = GqlError::node_not_found("/root/Test");
        let structured = err.to_structured();

        assert_eq!(structured.code, "VALIDATION_NODE_NOT_FOUND");
        assert!(structured.suggestion.is_some());
        assert_eq!(
            structured.context.get("node_path"),
            Some(&"/root/Test".to_string())
        );
    }

    #[test]
    fn test_error_display() {
        let err = GqlError::connection_timeout();
        let display = format!("{}", err);
        assert!(display.contains("CONN_TIMEOUT"));
    }

    #[test]
    fn test_error_with_location() {
        let err = StructuredError::new("TEST", ErrorCategory::FileSystem, "Test").with_location(
            Some("res://test.gd".to_string()),
            Some(10),
            Some(5),
        );

        assert!(err.location.is_some());
        let loc = err.location.unwrap();
        assert_eq!(loc.file, Some("res://test.gd".to_string()));
        assert_eq!(loc.line, Some(10));
        assert_eq!(loc.column, Some(5));
    }

    #[test]
    fn test_error_with_stack_trace() {
        let err = StructuredError::new("TEST", ErrorCategory::Godot, "Test")
            .with_stack_frame("func1", Some("file1.gd".to_string()), Some(10))
            .with_stack_frame("func2", Some("file2.gd".to_string()), Some(20));

        assert_eq!(err.stack_trace.len(), 2);
        assert_eq!(err.stack_trace[0].function, "func1");
        assert_eq!(err.stack_trace[1].function, "func2");
    }
}
