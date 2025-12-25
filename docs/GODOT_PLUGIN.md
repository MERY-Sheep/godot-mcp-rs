# Godot エディタープラグイン (Godot MCP)

## 概要

Godot エディター内で動作するプラグインを介して、MCP サーバーからのコマンドをリアルタイムで実行します。
Undo/Redo に完全対応しており、AI による変更をエディターの履歴から元に戻すことができます。

## インストール方法

1. このリポジトリの `addons/godot_mcp` ディレクトリを、ターゲットプロジェクトの `addons/` ディレクトリにコピー（またはシンボリックリンクを作成）します。
2. Godot エディターを開き、**Project -> Project Settings -> Plugins** タブを選択します。
3. **Godot MCP** プラグインの **Enable** チェックボックスをオンにします。
4. 出力パネルに `Godot MCP Plugin started on port 6060` と表示されれば成功です。

## 技術仕様

### サーバー構成

- **プロトコル**: シンプルな TCP サーバー (HTTP-like JSON POST)
- **ポート**: デフォルト `6060` (切替可能)
- **スクリプト**: `plugin.gd` がサーバーを管理し、`command_handler.gd` がコマンドを処理します。

### 対応コマンド (Live Commands)

現在、以下のコマンドがリアルタイム操作に対応しています：

| カテゴリ                 | コマンド                                                                                          | Undo/Redo |
| :----------------------- | :------------------------------------------------------------------------------------------------ | :-------: |
| **ノード**               | `add_node`, `remove_node`, `rename_node`, `duplicate_node`, `reparent_node`, `instantiate_scene`  |    ✅     |
| **プロパティ**           | `set_property`                                                                                    |    ✅     |
|                          | `get_properties`                                                                                  |     -     |
| **シーン**               | `get_tree`, `save_scene`                                                                          |     -     |
| **シグナル**             | `connect_signal`, `disconnect_signal`                                                             |    ✅     |
|                          | `list_signals`                                                                                    |     -     |
| **アニメーション**       | `create_animation`                                                                                |    ✅     |
|                          | `add_animation_track`, `add_animation_key`, `play_animation`, `stop_animation`, `list_animations` |     -     |
| **デバッグ**             | `get_editor_log`, `clear_editor_log`, `get_parse_errors`, `get_stack_frame_vars`                  |     -     |
| **イントロスペクション** | `get_type_info`, `list_all_types`                                                                 |     -     |
| **トランザクション**     | `begin_transaction`, `commit_transaction`, `rollback_transaction`                                 |    ✅     |
| **シェーダー**           | `create_visual_shader_node`, `validate_shader_live`                                               |     -     |

## Undo/Redo について

すべての破壊的な操作（ノード追加、プロパティ変更、シグナル接続など）は、Godot の `EditorUndoRedoManager` を介して実行されます。
これにより、AI が行った操作を `Ctrl+Z` で個別に取り消したり、やり直したりすることが可能です。

## 開発者向け: 新しいコマンドの追加

1. `command_handler.gd` の `handle_command` match ステートメントに新しいケースを追加します。
2. 対応する `_handle_xxxx` 関数を実装し、必要に応じて `ur.create_action()` を使用して Undo/Redo を登録します。
3. Rust CLI (`src/cli.rs`) に対応する `LiveXXXX` コマンドを追加します。
