//! Godot MCP Server
//!
//! LLMがGodotプロジェクトを操作するためのMCPサーバー

mod godot;
mod server;
mod tools;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化（stderrに出力、stdoutはMCP通信用）
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "godot_mcp_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Godot MCP Server starting...");

    // MCPサーバー起動
    server::run().await?;

    Ok(())
}
