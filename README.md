# Godot MCP Server (Rust Implementation)

[![MCP Server](https://img.shields.io/badge/MCP-Server-orange.svg)](https://modelcontextprotocol.io)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Godot](https://img.shields.io/badge/Godot-4.x-green.svg)](https://godotengine.org)

An MCP server for highly advanced manipulation and analysis of Godot game engine projects from AI (LLMs).
AI can integrally support everything from scene construction and GDScript editing to project execution and debugging.

[日本語版はこちら (Japanese version)](README.ja.md)

## Key Features

- **Editor & Execution Control (New! 🚀)**: AI can launch the game, check logs, and debug.
- **Powerful Parser**: Analyzes `.tscn`, `.gd`, and `.tres` formats, enabling structural changes.
- **Rich Toolset**: 34 tools covering everything from file exploration to advanced scene diffing and project validation.
- **Template Generation**: Instant scene construction from templates like `player_3d`, `enemy_3d`, `ui_menu`, etc.
- **Fast Rust Implementation**: Low-latency synchronous and asynchronous processing based on the official `rmcp` SDK.

## Toolset (34 tools)

Refer to [USAGE.en.md](docs/USAGE.en.md) for detailed usage.

### 🎮 Editor & Execution Control

- Run project (`run_project`), Stop project (`stop_project`)
- Capture debug output (`get_debug_output`)
- Launch editor, Check version

### 📁 Project Analysis

- **Project Stats** (`get_project_stats`), **Validation** (`validate_project`)
- **Node Type Info** (`get_node_type_info`) - Details on 25+ node types

### 🏗️ Scene Manipulation (.tscn)

- **Template Generation** (`create_scene_from_template`) - 5 templates
- Add/Remove/Set nodes, Hierarchical JSON export, Diff comparison

### 📜 Script Manipulation (.gd)

- Automatic insertion of functions and `@export` variables, static analysis

### 💎 Resource Manipulation (.tres)

- Resource parsing

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

## License

MIT
