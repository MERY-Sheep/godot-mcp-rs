//! MCP Server implementation

use crate::tools::GodotTools;
use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};

/// Start the MCP server
pub async fn run() -> Result<()> {
    // #region agent log
    let log_path = ".cursor/debug.log";
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"C\",\"location\":\"server.rs:run:entry\",\"message\":\"server::run entry\",\"data\":{{}},\"timestamp\":{}}}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
    });
    // #endregion
    let tools = GodotTools::new();

    let transport = stdio();

    // #region agent log
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"C\",\"location\":\"server.rs:run:before_serve\",\"message\":\"before serve\",\"data\":{{}},\"timestamp\":{}}}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
    });
    // #endregion
    let server = tools.serve(transport).await.map_err(|e| {
        // #region agent log
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"C\",\"location\":\"server.rs:run:serve_error\",\"message\":\"serve error\",\"data\":{{\"error\":\"{}\"}},\"timestamp\":{}}}", e, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
        });
        // #endregion
        e
    })?;

    tracing::info!("MCP Server initialized: {:?}", server.peer_info());

    // #region agent log
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"C\",\"location\":\"server.rs:run:before_waiting\",\"message\":\"before waiting\",\"data\":{{}},\"timestamp\":{}}}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
    });
    // #endregion
    // Wait until the server exits
    server.waiting().await.map_err(|e| {
        // #region agent log
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"C\",\"location\":\"server.rs:run:waiting_error\",\"message\":\"waiting error\",\"data\":{{\"error\":\"{}\"}},\"timestamp\":{}}}", e, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
        });
        // #endregion
        e
    })?;

    Ok(())
}
