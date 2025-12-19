//! Godot MCP Server
//!
//! LLMがGodotプロジェクトを操作するためのMCPサーバー
//! CLIモードでは直接ツールを実行可能

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
    // ロギング初期化（stderrに出力、stdoutはMCP通信用）
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "godot_mcp_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // コマンドライン引数をパース
    let args: Vec<String> = std::env::args().collect();

    // 引数がない場合、またはserveの場合はMCPサーバーモード
    if args.len() == 1 {
        tracing::info!("Godot MCP Server starting (MCP mode)...");
        server::run().await?;
    } else {
        // CLIモード
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
