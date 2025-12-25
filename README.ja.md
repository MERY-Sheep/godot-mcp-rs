# Godot MCP Server (Rust Implementation)

[![MCP Server](https://img.shields.io/badge/MCP-Server-orange.svg)](https://modelcontextprotocol.io)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Godot](https://img.shields.io/badge/Godot-4.x-green.svg)](https://godotengine.org)

Godot ゲームエンジンのプロジェクトを AI (LLM) から高度に操作・分析するための Model Context Protocol (MCP) サーバーです。
強力な GraphQL API を提供し、プロジェクトの静的解析から実行中のエディターのリアルタイム操作までをシームレスに統合します。

## 🛠️ コア・ツールセット (GraphQL API)

すべての Godot 操作は、MCP 経由で公開される **3 つの GraphQL ツール** に統合されています。

| ツール名           | 説明                                                 |
| :----------------- | :--------------------------------------------------- |
| `godot_query`      | 読み取り専用クエリ（プロジェクト情報、シーン構造）   |
| `godot_mutate`     | 変更操作（ノード追加、プロパティ設定、シグナル接続） |
| `godot_introspect` | スキーマ取得（SDL/イントロスペクション）             |

> [!TIP] > `addNode`, `setProperty` 等の個々の操作は GraphQL の **フィールド** として定義されています。
> `godot_introspect` でスキーマを取得することで、利用可能なすべての操作を確認できます。

### 何ができるか

1. **`godot_query`**: 読み取り専用の操作。

   - **プロジェクト分析**: プロジェクトの基本情報、統計、バリデーション結果の取得。
   - **構造の把握**: シーン階層、スクリプト定義、リソース間の依存関係（依存グラフ）の解析。
   - **実行状態の監視**: 実行中のエディターからのログ取得、変数/ノードの状態インスペクション。

2. **`godot_mutate`**: 変更を伴う操作。

   - **プロジェクトの編集**: シーン内のノード追加/削除、プロパティ変更、スクリプトの修正。
   - **リアルタイム操作**: エディター上のノードを即座に動かす、アニメーションを再生する、シグナルを接続する。
   - **安全な変更フロー**: バリデーションとプレビュー（Diff 生成）を経て、段階的に変更を適用。

3. **`godot_introspect`**: 自己記述的な操作。
   - **API 探索**: 現在のサーバーが提供するすべてのクエリ、ミューテーション、およびデータ型を SDL 形式で取得。

## 🚀 主な機能

- **自律的 TDD 支援**: GraphQL 経由で GdUnit4 テストを実行し、失敗箇所を構造化データとして取得。AI による自動修正ループをサポート。
- **エディター・Live 操作**: エディターを開いたまま、操作内容を即座に UI に反映。すべての操作はエディターの **Undo/Redo 履歴**に残ります。
- **強力な静的解析**: エディターが起動していない状態でも、`.tscn`, `.gd`, `.tres` ファイルを直接解析し、プロジェクト構造を把握。

## 🏗️ アーキテクチャ

本プロジェクトは、効率的な解析と深いエンジン統合を両立させるため、ハイブリッド構成を採用しています。

- **Rust サイド (Core Server)**:
  - 膨大なプロジェクトファイルの高速パース、GraphQL スキーマの提供、MCP 通信を担当。
  - エディター非依存で動作するため、CI 環境やエディター起動前でも高い解析能力を発揮します。
- **GDScript サイド (Editor Plugin)**:
  - Godot エディター内部の API にアクセスし、リアルタイムのノード操作や実行制御を担当。
  - Rust サーバーとは、ローカルの高速通信（SSE/HTTP）を介して連携します。

## 🔧 セットアップ

### 1. ビルド

```bash
cargo build --release
```

### 2. Godot プラグインの有効化

`addons/godot_mcp` を自身の Godot プロジェクトの `addons/` フォルダに配置し、プロジェクト設定からプラグインを有効にしてください。

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

## 💻 CLI モード

CLI ツールとして、シェルスクリプトやパイプラインからも GraphQL を活用可能です。

```bash
# プロジェクト統計のクエリ
godot-mcp-rs tool gql-query --project ./my_game --query "{ project { stats { sceneCount scriptCount } } }"

# スキーマの探索
godot-mcp-rs tool gql-introspect --project ./my_game --format SDL
```

## ライセンス

MIT
