# Godot MCP Server Architecture

[English](#english) | [日本語](#japanese)

---

<a name="english"></a>

## English

### Overview

An MCP server for LLMs (Claude, Cursor, etc.) to highly manipulate and analyze Godot projects.
It not only performs simple file operations but also parses Godot's unique `tscn`, `gd`, `tres` files internally to understand the structure, and also performs **process execution control**, realizing an "autonomous debug loop" by AI.

### System Configuration

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
            ED["Editor/Run<br/>(Run/Debug)"]
            PT["Project<br/>(Explore/Stats/Validate)"]
            ST["Scene<br/>(Scene Ops)"]
            SCT["Script<br/>(GDScript Edit)"]
            RT["Resource<br/>(Resource)"]
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

### Tool Categories (34 total)

| Category | Tool Count | Main Features |
| --- | --- | --- |
| Editor/Run | 6 | Project run, stop, log retrieval, check version |
| Project | 7 | File exploration, search, stats, validation, node type info |
| Scene | 13 | Create, read, edit, compare, template generation |
| Script | 6 | Create, analyze, add function/variable |
| Resource | 2 | Resource list, parse |

### Module Roles

#### 1. Tool Layer (`src/tools/`)

- **`editor.rs`**: **NEW** Godot child process management, PID tracking, async output capture
- **`project.rs`**: Stats, validation, node type info
- **`scene.rs`**: Node manipulation, template generation
- **`script.rs`**: GDScript editing

#### 2. Parser Layer (`src/godot/`)

- TSCN, GDScript, TRES parsers

### Editor Control Mechanism (Status Management)

Since MCP is a stateless protocol, process state is persisted and managed as follows:

1. **PID Management**: Keeps process ID in `.godot_mcp_pid` directly under the project root.
2. **Output Buffer**: Writes stdout and stderr integrated into `.godot_mcp_output`, reading requested number of lines from the end with `get_debug_output`.
3. **Godot Path**: Automatically detects executable file in environment variable `GODOT_PATH` or PATH.

---

<a name="japanese"></a>

## Japanese

### 概要

LLM（Claude, Cursor 等）が Godot プロジェクトを高度に操作・分析するための MCP サーバーです。
単なるファイル操作ではなく、Godot 独自の `tscn`, `gd`, `tres` ファイルを内部でパースして構造を理解し、さらに**プロセスの実行制御**も行うことで、AI による「自律デバッグループ」を実現します。

### システム構成

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

### ツール分類 (全 34 種)

| カテゴリ   | ツール数 | 主な機能                                         |
| ---------- | -------- | ------------------------------------------------ |
| Editor/Run | 6        | プロジェクト実行、停止、ログ取得、バージョン確認 |
| Project    | 7        | ファイル探索、検索、統計、検証、ノード型情報     |
| Scene      | 13       | 作成、読取、編集、比較、テンプレート生成         |
| Script     | 6        | 作成、解析、関数/変数追加                        |
| Resource   | 2        | リソース一覧、パース                             |

### 各モジュールの役割

#### 1. ツールレイヤー (`src/tools/`)

- **`editor.rs`**: **NEW** Godot の子プロセス管理、PID 追跡、非同期出力キャプチャ
- **`project.rs`**: 統計、検証、ノード型情報
- **`scene.rs`**: ノード操作、テンプレート生成
- **`script.rs`**: GDScript 編集

#### 2. パーサーレイヤー (`src/godot/`)

- TSCN, GDScript, TRES の各パーサー

### エディター制御の仕組み (Status Management)

MCP はステートレスなプロトコルのため、プロセスの状態を以下のように永続化して管理します。

1. **PID 管理**: プロジェクトルート直下の `.godot_mcp_pid` にプロセス ID を保持。
2. **出力バッファ**: `.godot_mcp_output` に標準出力と標準エラーを統合して書き込み、`get_debug_output` で要求された行数を末尾から読み取ります。
3. **Godot パス**: 環境変数 `GODOT_PATH` または PATH 内の実行ファイルを自動検出します。
