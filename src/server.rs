//! MCP Server implementation

use crate::tools::GodotTools;
use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};

/// Start the MCP server
pub async fn run() -> Result<()> {
    let tools = GodotTools::new();

    let transport = stdio();

    let server = tools.serve(transport).await?;

    tracing::info!("MCP Server initialized: {:?}", server.peer_info());

    // Wait until the server exits
    server.waiting().await?;

    Ok(())
}
