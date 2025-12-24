# Godot MCP Server (Rust Implementation)

[![MCP Server](https://img.shields.io/badge/MCP-Server-orange.svg)](https://modelcontextprotocol.io)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Godot](https://img.shields.io/badge/Godot-4.x-green.svg)](https://godotengine.org)

An MCP server for highly advanced manipulation and analysis of Godot game engine projects from AI (LLMs).
It provides a powerful GraphQL API that seamlessly integrates static project analysis with real-time editor interaction.

[日本語版はこちら (Japanese version)](README.ja.md)

## 🛠️ Core Toolset (GraphQL API)

All Godot operations are consolidated into 3 versatile GraphQL tools, allowing LLMs to manipulate projects with minimal context overhead:

1. **`godot_query`**: Read-only operations.

   - **Project Analysis**: Fetch project metadata, statistics, and validation status.
   - **Structure Discovery**: Analyze scene hierarchies, script definitions, and resource dependencies (Dependency Graph).
   - **Live Monitoring**: Capture logs and inspect node/variable states in the running editor.

2. **`godot_mutate`**: Operations that modify the project.

   - **Project Editing**: Add/remove nodes, change properties, and modify scripts.
   - **Live Interaction**: Manipulate the active editor UI, play animations, and connect signals.
   - **Safe Change Flow**: Validate and preview changes (via Diff) before applying them.

3. **`godot_introspect`**: Self-describing API discovery.
   - **API Schema**: Get the full list of available queries, mutations, and types in SDL format.

## 🚀 Key Features

- **Autonomous TDD Support**: Run GdUnit4 tests via GQL and retrieve structured error reports. Facilitates AI-driven test-fix loops.
- **Editor Live Interaction**: Reflect changes instantly in the editor UI. All operations are recorded in the editor's **Undo/Redo history**.
- **Deep Static Analysis**: Directly parses `.tscn`, `.gd`, and `.tres` files to understand project structure even when the editor is closed.

## 🏗️ Architecture

This project uses a hybrid approach to balance high-performance analysis with deep engine integration:

- **Rust Side (Core Server)**:
  - Responsible for high-speed parsing of project files, providing the GraphQL schema, and MCP communication.
  - Being editor-independent, it works perfectly in CI environments or before the editor is launched.
- **GDScript Side (Editor Plugin)**:
  - Accesses internal Godot Editor APIs for real-time node manipulation and execution control.
  - Communicates with the Rust server via local high-speed protocols (SSE/HTTP).

## 🔧 Getting Started

### 1. Build

```bash
cargo build --release
```

### 2. Enable Godot Plugin

Copy `addons/godot_mcp` to your project's `addons/` directory and enable the plugin in Project Settings.

### 3. Claude Desktop Configuration

`%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "godot": {
      "command": "C:\\path\\to\\godot-mcp-rs.exe",
      "env": {
        "GODOT_PATH": "C:\\path\\to\\godot.exe"
      }
    }
  }
}
```

## 💻 CLI Mode

Use GraphQL for automation scripts and pipelines directly from your terminal.

```bash
# Query project statistics
godot-mcp-rs tool gql-query --project ./my_game --query "{ project { stats { sceneCount } } }"

# Introspect schema
godot-mcp-rs tool gql-introspect --project ./my_game --format SDL
```

## License

MIT
