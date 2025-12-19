# Godot MCP Plugin

LLM（AI）から Godot エディターをリアルタイム操作するためのプラグインです。

## インストール

1. `addons/godot_mcp/` フォルダをあなたの Godot プロジェクトの `addons/` にコピー
2. **Project → Project Settings → Plugins** で **Godot MCP** を有効化
3. エディターの Output パネルに `Godot MCP: Server started on port 6060` と表示されれば成功

## 使い方

プラグイン有効化後、ポート 6060 で HTTP リクエストを受け付けます。

### コマンド一覧

| カテゴリ           | コマンド                                                                                                              | 説明                                             |
| :----------------- | :-------------------------------------------------------------------------------------------------------------------- | :----------------------------------------------- |
| **基本**           | `ping`                                                                                                                | 接続確認                                         |
| **ノード**         | `add_node`, `remove_node`, `rename_node`, `duplicate_node`, `reparent_node`, `instantiate_scene`                      | 追加、削除、名前変更、複製、親子変更、シーン配置 |
| **プロパティ**     | `get_properties`, `set_property`                                                                                      | 全プロパティ取得、個別プロパティ設定             |
| **シーン**         | `get_tree`, `save_scene`                                                                                              | ノードツリー取得、シーン保存                     |
| **シグナル**       | `connect_signal`, `disconnect_signal`, `list_signals`                                                                 | 接続、切断、一覧取得                             |
| **アニメーション** | `create_animation`, `add_animation_track`, `add_animation_key`, `play_animation`, `stop_animation`, `list_animations` | 作成、編集、再生制御、リスト取得                 |
| **デバッグ**       | `get_editor_log`, `clear_editor_log`                                                                                  | エディターログの取得とクリア                     |

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
