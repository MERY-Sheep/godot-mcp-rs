# Godot MCP Server (Rust Implementation)

[![MCP Server](https://img.shields.io/badge/MCP-Server-orange.svg)](https://modelcontextprotocol.io)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Godot](https://img.shields.io/badge/Godot-4.x-green.svg)](https://godotengine.org)

An MCP server for highly advanced manipulation and analysis of Godot game engine projects from AI (LLMs).
AI can integrally support everything from scene construction and GDScript editing to project execution and debugging.

[日本語版はこちら (Japanese version)](README.ja.md)

## Key Features

- **GraphQL (GQL) Power (New! 🔥)**: Access all Godot features through a single, unified GraphQL interface.
- **Autonomous TDD Support**: Run GdUnit4 tests via GQL and get structured error reports. Enables automatic AI-driven test-fix loops.
- **Editor & Execution Control**: AI can launch the game, check logs, and debug.
- **Powerful Parser**: Analyzes `.tscn`, `.gd`, and `.tres` formats, enabling structural changes.
- **Streamlined MCP Toolset**: Only 3 core tools to learn, reducing LLM context overhead.
- **Fast Rust Implementation**: Low-latency synchronous and asynchronous processing based on the official `rmcp` SDK.

## Core Toolset

This server provides 3 unified tools that interface with the underlying GraphQL engine. Refer to [MIGRATION_GUIDE.md](docs/gql/MIGRATION_GUIDE.md) for shifting from legacy tools.

1. **`godot_query`**: Read project state, scenes, scripts, and stats using GQL.
2. **`godot_mutate`**: Modify project structure, create files, and interact with the Live editor.
3. **`godot_introspect`**: Discover available queries and mutations via SDL.

### Capabilities covered by GQL:

- **Project Analysis**: Stats, validation, and node type information.
- **Scene Manipulation**: Add/Remove nodes, edit properties, and hierarchical export.
- **Script & Resource**: GDScript parsing/editing and `.tres` resource management.
- **Live Interaction**: Real-time manipulation of the running Godot editor.

## Quick Start

### 1. Build

```bash
cargo build --release
```

### 2. Environment Variables (Optional)

```bash
set GODOT_PATH=C:\path\to\godot.exe
```

### 3. Claude Desktop Configuration

`%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "godot": {
      "command": "C:\\Work\\godot-mcp-rs\\target\\release\\godot-mcp-rs.exe"
    }
  }
}
```

See [mcp_config.json.example](mcp_config.json.example) for more options.

## License

MIT
