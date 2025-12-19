# Godot MCP Plugin

LLM（AI）から Godot エディターをリアルタイム操作するためのプラグインです。

## インストール

1. `addons/godot_mcp/` フォルダをあなたの Godot プロジェクトの `addons/` にコピー
2. **Project → Project Settings → Plugins** で **Godot MCP** を有効化
3. エディターの Output パネルに `Godot MCP: Server started on port 6060` と表示されれば成功

## 使い方

プラグイン有効化後、ポート 6060 で HTTP リクエストを受け付けます。

### コマンド一覧

| コマンド       | 説明             | パラメータ                       |
| -------------- | ---------------- | -------------------------------- |
| `ping`         | ヘルスチェック   | なし                             |
| `add_node`     | ノード追加       | `parent`, `name`, `node_type`    |
| `remove_node`  | ノード削除       | `node_path`                      |
| `set_property` | プロパティ設定   | `node_path`, `property`, `value` |
| `get_tree`     | ノードツリー取得 | なし                             |
| `save_scene`   | シーン保存       | なし                             |

### 使用例（PowerShell）

```powershell
# ping
Invoke-RestMethod -Uri "http://localhost:6060" -Method POST -Body '{"command":"ping"}' -ContentType "application/json"

# ノード追加
Invoke-RestMethod -Uri "http://localhost:6060" -Method POST -Body '{"command":"add_node","params":{"parent":".","name":"MyNode","node_type":"Node3D"}}' -ContentType "application/json"

# ノードツリー取得
Invoke-RestMethod -Uri "http://localhost:6060" -Method POST -Body '{"command":"get_tree","params":{}}' -ContentType "application/json"
```

### 使用例（curl）

```bash
curl -X POST http://localhost:6060 -H "Content-Type: application/json" -d '{"command":"ping"}'
```

## 特徴

- **Undo/Redo 対応**: すべての操作がエディターの履歴に統合
- **リアルタイム反映**: ファイル書き込みなしで即座にノードツリーに反映
- **シンプル**: 純粋な HTTP + JSON、追加依存なし

## 技術仕様

- ポート: 6060
- プロトコル: HTTP POST
- フォーマット: JSON

## ライセンス

MIT
