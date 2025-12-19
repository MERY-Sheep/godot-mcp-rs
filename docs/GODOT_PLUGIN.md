# Godot エディタープラグイン設計

## 概要

Godot エディター内で動作するプラグインを作成し、MCP サーバーからのコマンドをリアルタイムで実行する。
ファイル直接編集ではなく、`EditorInterface` API を使用することで、エディターを閉じずにノード追加などが可能になる。

## アーキテクチャ

```
┌─────────────────┐     HTTP/WS      ┌──────────────────────┐
│  MCP Server     │ ────────────────>│  Godot Editor Plugin │
│  (Rust CLI)     │                  │  (GDScript)          │
│                 │ <────────────────│                      │
│                 │     JSON Response│  EditorInterface API │
└─────────────────┘                  └──────────────────────┘
                                              ↓
                                     ノード追加・削除
                                     プロパティ変更
                                     シーン保存
                                     リアルタイム反映 ✨
```

## 通信プロトコル

### オプション A: HTTP Server (推奨・シンプル)

- プラグインが HTTP サーバー（ポート 6060）を起動
- Rust CLI が HTTP POST でコマンドを送信
- レスポンスを JSON で返却

### オプション B: WebSocket (双方向通信)

- より複雑だが、サーバーからプッシュ通知が可能
- 将来的な拡張に有利

---

## プラグイン構成

```
addons/
└── godot_mcp/
    ├── plugin.cfg          # プラグイン定義
    ├── plugin.gd           # メインプラグインスクリプト
    ├── http_server.gd      # HTTP リクエスト処理
    └── command_handler.gd  # コマンド実行ロジック
```

---

## 対応コマンド（第一弾）

| コマンド       | 説明             | EditorInterface API                      |
| -------------- | ---------------- | ---------------------------------------- |
| `add_node`     | ノード追加       | `get_edited_scene_root()`, `add_child()` |
| `remove_node`  | ノード削除       | `get_node()`, `queue_free()`             |
| `set_property` | プロパティ設定   | `set()`                                  |
| `save_scene`   | シーン保存       | `save_scene()`                           |
| `get_tree`     | ノードツリー取得 | 再帰的に取得                             |

---

## 実装例

### plugin.gd

```gdscript
@tool
extends EditorPlugin

var http_server: HTTPServer

func _enter_tree():
    http_server = HTTPServer.new()
    http_server.start(6060)
    add_child(http_server)
    print("Godot MCP Plugin started on port 6060")

func _exit_tree():
    http_server.stop()
```

### command_handler.gd（Undo/Redo 対応）

```gdscript
func handle_add_node(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    var parent_path = params.get("parent", ".")
    var parent = root.get_node(parent_path) if parent_path != "." else root

    var new_node = ClassDB.instantiate(params["node_type"])
    new_node.name = params["name"]
    new_node.owner = root

    # Undo/Redo 対応（エディターの履歴に統合）
    var ur = get_undo_redo()
    ur.create_action("Add Node via LLM: " + params["name"])
    ur.add_do_method(parent, "add_child", new_node)
    ur.add_do_reference(new_node)
    ur.add_undo_method(parent, "remove_child", new_node)
    ur.commit_action()

    return {"success": true, "node": new_node.get_path()}
```

---

## Rust CLI 側の変更

新しいコマンドを追加:

```bash
godot-mcp-rs tool live-add-node --port 6060 --parent . --name HealthBar --type ProgressBar
```

内部で HTTP POST を送信:

```rust
reqwest::Client::new()
    .post("http://localhost:6060/command")
    .json(&json!({
        "command": "add_node",
        "params": { "parent": ".", "name": "HealthBar", "node_type": "ProgressBar" }
    }))
    .send()
```

---

## 検証計画

1. プラグイン有効化後、`http://localhost:6060/ping` でヘルスチェック
2. `add_node` コマンドでエディター内にノードが追加されることを確認
3. エディターの Undo が機能することを確認
