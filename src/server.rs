//! MCPサーバー実装

use crate::tools::GodotTools;
use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};

/// MCPサーバーを起動
pub async fn run() -> Result<()> {
    let tools = GodotTools::new();

    let transport = stdio();

    let server = tools.serve(transport).await?;

    tracing::info!("MCP Server initialized: {:?}", server.peer_info());

    // サーバーが終了するまで待機
    server.waiting().await?;

    Ok(())
}
