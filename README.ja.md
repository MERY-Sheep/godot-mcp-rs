# Godot MCP Server (Rust Implementation)

[![MCP Server](https://img.shields.io/badge/MCP-Server-orange.svg)](https://modelcontextprotocol.io)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Godot](https://img.shields.io/badge/Godot-4.x-green.svg)](https://godotengine.org)

Godot ゲームエンジンのプロジェクトを AI (LLM) から高度に操作・分析するための Model Context Protocol (MCP) サーバーです。
ファイルベースの解析に加え、エディタープラグインを介した**リアルタイム (live) 操作**を強力にサポートします。

## 主な特徴

- **リアルタイム操作 (live-\*)**: エディターを開いたまま、ノードの追加・削除・プロパティ変更・シグナル接続・アニメーション作成を即座に反映。
- **完全な Undo/Redo サポート**: AI による live 操作はすべて Godot エディターの Undo 履歴に残るため、安心して試行錯誤が可能。
- **強力なパーサー**: `.tscn`, `.gd`, `.tres` 形式を解析し、エディターを閉じた状態でも構造的な変更を可能にします。
- **豊富なツールセット**: 全 56 種のツールにより、開発の全工程（構築・分析・実行・デバッグ）を AI が統合的にサポート。
- **プロジェクト実行制御**: AI が自らゲームを起動し、ログを確認して修正するデバッグループが可能。

## ツールセット (全 56 種)

詳細な使い方は [USAGE.md](docs/USAGE.md) を参照してください。

### ✨ リアルタイム操作 (live-\*)

Godot エディターとの直接連携により、UI 上で即座に変更を確認できます。

- ノード操作: `add`, `remove`, `rename`, `duplicate`, `reparent`
- プロパティ: `get-properties`, `set-property`
- 構築: `get-tree`, `instantiate-scene`, `save-scene`
- シグナル: `connect`, `disconnect`, `list-signals`
- アニメーション: `create`, `add-track`, `add-key`, `play`, `stop`, `list`
- デバッグ: `get-editor-log`, `clear-editor-log`

### 🏗️ ファイルベース操作

エディターを開かずにファイルを直接操作・分析します。

- シーン構築 (`create_scene`, `create_scene_from_template`)
- 階層解析 (`read_scene`, `export_scene_as_json`, `compare_scenes`)
- スクリプト編集 (`add_function`, `add_export_var`, `analyze_script`)

### 🎮 プロジェクト実行・分析

- プロジェクトの実行 (`run_project`)・停止 (`stop_project`)
- 統計解析 (`get_project_stats`)、検証 (`validate_project`)
- ノード型情報 (`get_node_type_info`)

## インストールとセットアップ

### 1. ビルド

```bash
cargo build --release
```

### 2. Godot プラグインのインストール

1. `addons/godot_mcp` ディレクトリをあなたのプロジェクトの `addons/` フォルダにコピーします。
2. Project Settings -> Plugins から **Godot MCP** を有効にします。

### 3. Claude Desktop 設定

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

詳細は [mcp_config.json.example](mcp_config.json.example) を参照してください。

## CLI モード

MCP サーバーとしてだけでなく、単体の CLI ツールとしても利用可能です。

```bash
# リアルタイムでノードを追加
godot-mcp-rs tool live-add-node --name "Bot" --node-type "CharacterBody3D"

# プロジェクトの状態を確認
godot-mcp-rs tool get-project-stats --project ./my_game
```

## ライセンス

MIT
