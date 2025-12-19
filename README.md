# Godot MCP Server (Rust Implementation)

[![MCP Server](https://img.shields.io/badge/MCP-Server-orange.svg)](https://modelcontextprotocol.io)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Godot](https://img.shields.io/badge/Godot-4.x-green.svg)](https://godotengine.org)

[English](#english) | [日本語](#japanese)

---

<a name="english"></a>

## English

An MCP server for advanced manipulation and analysis of Godot game engine projects from AI (LLM).
It provides integrated support from scene construction, GDScript editing, to project execution and debugging.

### Key Features

- **Editor & Execution Control (New! 🚀)**: AI can launch the game, check logs, and debug autonomously.
- **Powerful Parsers**: Parses `.tscn`, `.gd`, `.tres` formats, enabling structural changes.
- **Rich Toolset**: Total of 34 tools covering everything from file exploration to advanced scene comparison and project verification.
- **Template Generation**: One-shot scene construction from templates like `player_3d`, `enemy_3d`, `ui_menu`.
- **Fast Rust Implementation**: Low-latency synchronous/asynchronous processing based on the official `rmcp` SDK.

### Toolset (34 tools in total)

See [USAGE.md](docs/USAGE.md) for detailed usage.

#### 🎮 Editor & Execution Control

- Run (`run_project`) and stop (`stop_project`) project
- Capture debug output (`get_debug_output`)
- Launch editor, check version

#### 📁 Project Analysis

- **Project Stats** (`get_project_stats`), **Validation** (`validate_project`)
- **Node Type Info** (`get_node_type_info`) - Details for over 25 node types

#### 🏗️ Scene Manipulation (.tscn)

- **Template Generation** (`create_scene_from_template`) - 5 types of templates
- Add/Remove/Set nodes, hierarchical JSON export, diff comparison

#### 📜 Script Manipulation (.gd)

- Auto-insertion of functions/`@export` variables, static analysis

#### 💎 Resource Manipulation (.tres)

- Resource analysis

### Quick Start

#### 1. Build

```bash
cargo build --release
```

#### 2. Set Environment Variable (Optional)

```bash
set GODOT_PATH=C:\path\to\godot.exe
```

#### 3. Claude Desktop Configuration

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

### License

MIT

---

<a name="japanese"></a>

## Japanese

Godot ゲームエンジンのプロジェクトを AI（LLM）から高度に操作・分析するための MCP サーバーです。
シーンの構築、GDScript の編集、プロジェクトの実行・デバッグまでを AI が統合的にサポートします。

### 主な特徴

- **エディター・実行制御 (New! 🚀)**: AI が自らゲームを起動し、ログを確認してデバッグ可能。
- **強力なパーサー**: `.tscn`, `.gd`, `.tres` 形式を解析し、構造的な変更を可能にします。
- **豊富なツールセット**: 全 34 種のツールにより、ファイル探索から高度なシーン差分比較、プロジェクト検証までカバー。
- **テンプレート生成**: `player_3d`, `enemy_3d`, `ui_menu` 等のテンプレートから一発でシーン構築。
- **高速な Rust 実装**: `rmcp` 公式 SDK をベースにした低遅延な同期・非同期処理。

### ツールセット (全 34 種)

詳細な使い方は [USAGE.md](docs/USAGE.md) を参照してください。

#### 🎮 エディター・実行制御

- プロジェクトの実行 (`run_project`)・停止 (`stop_project`)
- デバッグ出力キャプチャ (`get_debug_output`)
- エディター起動、バージョン確認

#### 📁 プロジェクト分析

- **プロジェクト統計** (`get_project_stats`)、**検証** (`validate_project`)
- **ノード型情報** (`get_node_type_info`) - 25 種以上のノード型詳細

#### 🏗️ シーン操作 (.tscn)

- **テンプレート生成** (`create_scene_from_template`) - 5 種のテンプレート
- ノードの追加・削除・設定、階層 JSON エクスポート、差分比較

#### 📜 スクリプト操作 (.gd)

- 関数・`@export` 変数の自動挿入、静的解析

#### 💎 リソース操作 (.tres)

- リソース解析

### クイックスタート

#### 1. ビルド

```bash
cargo build --release
```

#### 2. 環境変数設定 (任意)

```bash
set GODOT_PATH=C:\path\to\godot.exe
```

#### 3. Claude Desktop 設定

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

### ライセンス

MIT
