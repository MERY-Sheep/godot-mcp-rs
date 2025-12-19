# タスク C: デバッグログ監視

## 概要

プロジェクト実行中の print/エラー出力をリアルタイムで取得するコマンドを追加する。

## 実装内容

### 1. 追加するコマンド

| コマンド           | 説明                           |
| ------------------ | ------------------------------ |
| `get_editor_log`   | エディターのデバッグ出力を取得 |
| `clear_editor_log` | ログをクリア                   |

### 2. Godot プラグイン側

**ファイル**: `addons/godot_mcp/command_handler.gd`

```gdscript
# match文に追加
"get_editor_log":
    return _handle_get_editor_log(params)
"clear_editor_log":
    return _handle_clear_editor_log(params)

# クラス変数としてログバッファを追加（plugin.gd に）
var log_buffer: Array = []
const MAX_LOG_LINES = 1000

# EditorPlugin の _ready() でログキャプチャを設定
func _ready():
    # デバッグログのキャプチャ（Godot 4.x では直接不可、代替手段を使用）
    pass

# ハンドラ関数
func _handle_get_editor_log(params: Dictionary) -> Dictionary:
    var lines = params.get("lines", 50)

    # EditorInterface.get_editor_log() は存在しない
    # 代替: Output パネルからテキストを取得（複雑）
    # または: プラグイン内でログをキャプチャ

    # 簡易実装: 最近の print 出力を返す
    # 注意: Godot のエディターログへの直接アクセスは制限あり

    return {
        "success": true,
        "lines": lines,
        "log": ["This feature requires custom log capture setup"],
        "note": "Direct editor log access is limited in Godot 4.x"
    }

func _handle_clear_editor_log(params: Dictionary) -> Dictionary:
    # ログバッファをクリア
    log_buffer.clear()
    return {"success": true, "cleared": true}
```

### 3. 代替実装案: カスタムログキャプチャ

Godot 4.x ではエディターログへの直接アクセスが制限されているため、以下の代替案がある：

#### 案 A: プロジェクト実行時のログファイル読み取り

`.godot_mcp_output` ファイル（既に `run_project` で作成）を利用：

```rust
// Rust CLI 側（既存の get-debug-output を拡張）
ToolCommands::LiveGetDebugOutput { port, lines } => {
    // 直接ファイルを読む
    let output_file = project.join(".godot_mcp_output");
    let content = std::fs::read_to_string(&output_file)?;
    // ...
}
```

#### 案 B: カスタムロガーをプロジェクトに注入

```gdscript
# autoload として追加
extends Node

signal log_message(message: String, type: String)

func _init():
    # カスタムロガーを設定
    pass

func log(message: String):
    emit_signal("log_message", message, "info")
    print(message)
```

### 4. Rust CLI 側

**ファイル**: `src/cli.rs`

```rust
// enum ToolCommands に追加
/// Get editor debug log
LiveGetEditorLog {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = "50")]
    lines: u32,
},

/// Clear editor debug log
LiveClearEditorLog {
    #[arg(long, default_value = "6060")]
    port: u16,
},

// run_cli 関数の match に追加
ToolCommands::LiveGetEditorLog { port, lines } => {
    return run_live_command(port, "get_editor_log", serde_json::json!({
        "lines": lines
    })).await;
}
ToolCommands::LiveClearEditorLog { port } => {
    return run_live_command(port, "clear_editor_log", serde_json::json!({})).await;
}
```

### 5. テスト方法

```bash
# ビルド
cargo build --release

# テスト
.\target\release\godot-mcp-rs.exe tool live-get-editor-log --lines 100
.\target\release\godot-mcp-rs.exe tool live-clear-editor-log
```

## 工数見積もり

- Godot プラグイン: 1 時間（制限あり、基本実装）
- Rust CLI: 30 分
- テスト: 30 分
- **合計: 2 時間**

## 注意事項

- Godot 4.x ではエディターの Output パネルへの直接アクセスが制限されている
- 完全なログキャプチャには追加の仕組み（autoload、ファイル監視等）が必要
- 既存の `get-debug-output` コマンド（ファイルベース）と連携することを推奨
