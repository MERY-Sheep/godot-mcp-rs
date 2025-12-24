//! Godot MCP Server
//!
//! MCP server for LLM to interact with Godot projects.
//! Tools can be executed directly in CLI mode.

mod cli;
mod server;

// Re-export from lib for internal use
use godot_mcp_rs::{godot, graphql, tools};

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // #region agent log
    let log_path = ".cursor/debug.log";
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\",\"location\":\"main.rs:main:entry\",\"message\":\"main entry\",\"data\":{{}},\"timestamp\":{}}}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
    });
    // #endregion
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
    // #region agent log
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\",\"location\":\"main.rs:main:args_parsed\",\"message\":\"args parsed\",\"data\":{{\"args_len\":{}}},\"timestamp\":{}}}", args.len(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
    });
    // #endregion

    // If no arguments or "serve" command, start in MCP server mode.
    if args.len() == 1 {
        // #region agent log
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\",\"location\":\"main.rs:main:mcp_mode\",\"message\":\"starting MCP mode\",\"data\":{{}},\"timestamp\":{}}}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
        });
        // #endregion
        tracing::info!("Godot MCP Server starting (MCP mode)...");
        server::run().await.map_err(|e| {
            // #region agent log
            let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\",\"location\":\"main.rs:main:server_error\",\"message\":\"server run error\",\"data\":{{\"error\":\"{}\"}},\"timestamp\":{}}}", e, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
            });
            // #endregion
            e
        })?;
    } else {
        // CLI mode
        // #region agent log
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\",\"location\":\"main.rs:main:cli_mode\",\"message\":\"starting CLI mode\",\"data\":{{}},\"timestamp\":{}}}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
        });
        // #endregion
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
