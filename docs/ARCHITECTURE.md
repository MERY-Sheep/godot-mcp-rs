# Godot MCP Server アーキテクチャ

## 概要

LLM（Claude, Cursor 等）が Godot プロジェクトを高度に操作・分析するための MCP サーバーです。
単なるファイル操作ではなく、Godot 独自の `tscn`, `gd`, `tres` ファイルを内部でパースして構造を理解し、さらに**プロセスの実行制御**も行うことで、AI による「自律デバッグループ」を実現します。

## システム構成

```mermaid
flowchart TB
    subgraph Host["MCP Client (Claude Desktop / Cursor)"]
        Client["AI (LLM)"]
    end

    subgraph Server["godot-mcp-rs"]
        direction TB
        MCP["MCP Server Protocol Layer (rmcp)"]
        Handler["ServerHandler (GodotTools)"]

        subgraph Tools["Tools (src/tools/)"]
            direction LR
            ED["Editor/Run<br/>(実行・デバッグ)"]
            PT["Project<br/>(探索・統計・検証)"]
            ST["Scene<br/>(シーン操作)"]
            SCT["Script<br/>(GDScript編集)"]
            RT["Resource<br/>(リソース)"]
        end

        subgraph Parsers["Parsers (src/godot/)"]
            direction LR
            TSCN["TSCN Parser"]
            GD["GDScript Parser"]
            TRES["TRES Parser"]
        end

        subgraph Data["Static Data"]
            NT["Node Type Database"]
            TPL["Scene Templates"]
        end
    end

    subgraph FS["Godot Project File System"]
        FILES[".tscn / .gd / .tres ..."]
    end

    subgraph Godot["Godot Runtime"]
        Runtime["Game Process"]
        Output["Debug Output (Stdout/Stderr)"]
    end

    Client <-->|"JSON-RPC"| MCP
    MCP <--> Handler
    Handler --> Tools
    Tools --> Parsers
    Tools --> Data

    %% Execution Flow
    Tools -->|"Spawn"| Runtime
    Runtime -->|"Write"| Output
    Tools <--"Read"| Output

    Parsers <--> FS
```

## ツール分類 (全 34 種)

| カテゴリ   | ツール数 | 主な機能                                         |
| ---------- | -------- | ------------------------------------------------ |
| Editor/Run | 6        | プロジェクト実行、停止、ログ取得、バージョン確認 |
| Project    | 7        | ファイル探索、検索、統計、検証、ノード型情報     |
| Scene      | 13       | 作成、読取、編集、比較、テンプレート生成         |
| Script     | 6        | 作成、解析、関数/変数追加                        |
| Resource   | 2        | リソース一覧、パース                             |

## 各モジュールの役割

### 1. ツールレイヤー (`src/tools/`)

- **`editor.rs`**: **NEW** Godot の子プロセス管理、PID 追跡、非同期出力キャプチャ
- **`project.rs`**: 統計、検証、ノード型情報
- **`scene.rs`**: ノード操作、テンプレート生成
- **`script.rs`**: GDScript 編集

### 2. パーサーレイヤー (`src/godot/`)

- TSCN, GDScript, TRES の各パーサー

## エディター制御の仕組み (Status Management)

MCP はステートレスなプロトコルのため、プロセスの状態を以下のように永続化して管理します。

1. **PID 管理**: プロジェクトルート直下の `.godot_mcp_pid` にプロセス ID を保持。
2. **出力バッファ**: `.godot_mcp_output` に標準出力と標準エラーを統合して書き込み、`get_debug_output` で要求された行数を末尾から読み取ります。
3. **Godot パス**: 環境変数 `GODOT_PATH` または PATH 内の実行ファイルを自動検出します。
