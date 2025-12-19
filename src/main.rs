//! Godot MCP Server
//!
//! MCP server for LLMs to manipulate Godot projects

mod godot;
mod server;
mod tools;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (output to stderr, stdout is for MCP communication)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "godot_mcp_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Godot MCP Server starting...");

    // Start MCP server
    server::run().await?;

    Ok(())
}
