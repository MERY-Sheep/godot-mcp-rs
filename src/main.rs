//! Godot MCP Server
//!
//! MCP server for LLM to interact with Godot projects.
//! Tools can be executed directly in CLI mode.

mod cli;
mod godot;
mod server;
pub mod tools;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (output to stderr, stdout is reserved for MCP communication)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "godot_mcp_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // If no arguments or "serve" command, start in MCP server mode.
    if args.len() == 1 {
        tracing::info!("Godot MCP Server starting (MCP mode)...");
        server::run().await?;
    } else {
        // CLI mode
        let cli = Cli::parse();
        match cli.command {
            Commands::Serve => {
                tracing::info!("Godot MCP Server starting (MCP mode)...");
                server::run().await?;
            }
            Commands::Tool(tool_cmd) => {
                cli::run_cli(tool_cmd).await?;
            }
        }
    }

    Ok(())
}
