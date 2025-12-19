# Godot MCP Server - 使い方

## ツール一覧（34 個）

### エディター・実行制御 (New! 🚀)

| ツール               | 説明                                                     |
| :------------------- | :------------------------------------------------------- |
| `get_godot_version`  | インストール済み Godot のバージョンとパスを取得          |
| `run_project`        | プロジェクトをデバッグモードで実行（出力キャプチャ開始） |
| `stop_project`       | 実行中のプロジェクトを強制終了                           |
| `get_debug_output`   | 実行中または終了後のコンソール出力を取得                 |
| `launch_editor`      | Godot エディターを起動してプロジェクトを開く             |
| `get_running_status` | プロジェクトが現在実行中かどうかを確認                   |

---

### プロジェクト探索・分析

| ツール               | 説明                                                                   |
| :------------------- | :--------------------------------------------------------------------- |
| `list_project_files` | プロジェクト内のファイル一覧を取得                                     |
| `read_file`          | ファイル内容をテキストとして読み取り                                   |
| `list_all_scenes`    | 全シーン(.tscn)一覧を root_type 付きで取得                             |
| `search_in_project`  | プロジェクト全体を node_type, resource, script で検索                  |
| `get_node_type_info` | Godot ノード型の詳細情報（プロパティ、メソッド、推奨子）を取得         |
| `get_project_stats`  | プロジェクト統計（ファイル数、ノード数、型別集計）を取得               |
| `validate_project`   | プロジェクト全体を検証（パースエラー、空シーン、未使用スクリプト検出） |

---

### シーン操作 (.tscn)

| ツール                       | 説明                                                                            |
| :--------------------------- | :------------------------------------------------------------------------------ |
| `create_scene`               | 新規シーン作成                                                                  |
| `create_scene_from_template` | テンプレートからシーン生成（player_3d, player_2d, enemy_3d, level_3d, ui_menu） |
| `read_scene`                 | シーン内容を構造化 JSON として取得                                              |
| `copy_scene`                 | シーンファイルをコピー                                                          |
| `add_node`                   | 単一ノードを追加                                                                |
| `batch_add_nodes`            | 複数のノードを一度に追加                                                        |
| `remove_node`                | 指定パスのノードを削除                                                          |
| `set_node_property`          | ノードのプロパティ（position, scale 等）を設定                                  |
| `get_node_tree`              | ノードの親子構造をツリー形式で表示                                              |
| `get_scene_metadata`         | シーンの統計情報や依存関係を取得                                                |
| `validate_tscn`              | ファイルの構文チェック                                                          |
| `compare_scenes`             | 2 つのシーンの差分（ノード増減、プロパティ変更）を表示                          |
| `export_scene_as_json`       | 平坦な Godot シーンを階層構造 JSON に変換して取得                               |

---

### スクリプト操作 (.gd)

| ツール           | 説明                                                   |
| :--------------- | :----------------------------------------------------- |
| `create_script`  | スクリプト作成（テンプレート対応）                     |
| `attach_script`  | シーン内のノードにスクリプトを接続                     |
| `read_script`    | スクリプトをパースして関数・変数・シグナルを JSON 取得 |
| `add_function`   | 既存スクリプトに新しい関数を追加                       |
| `add_export_var` | 既存スクリプトに `@export` 変数を追加                  |
| `analyze_script` | スクリプトのサマリー情報（関数数など）を取得           |

---

### リソース操作 (.tres)

| ツール           | 説明                                                                 |
| :--------------- | :------------------------------------------------------------------- |
| `list_resources` | プロジェクト内の .tres ファイル一覧を取得                            |
| `read_resource`  | .tres をパースして内容（外部リソース、サブ、プロパティ）を JSON 取得 |

---

## 主な機能の詳細

### プロジェクトの実行とデバッグ

AI が自らゲームを実行し、ログを確認して修正するループが可能です。

1. `run_project` で実行開始。
2. `get_debug_output` で「何が起きているか」を確認。
3. エラーがあれば、パーサー系ツールで修正。
4. `stop_project` で停止。

### テンプレートからシーン作成

`create_scene_from_template` を使うと、よく使うシーン構成を一発生成できます。

```json
{ "path": "scenes/player.tscn", "template": "player_3d" }
```

| テンプレート | ルート型        | 生成ノード                                   |
| ------------ | --------------- | -------------------------------------------- |
| `player_3d`  | CharacterBody3D | Collision, Mesh, Camera, AnimationPlayer     |
| `player_2d`  | CharacterBody2D | Collision, Sprite, AnimatedSprite, Camera    |
| `enemy_3d`   | CharacterBody3D | Collision, Mesh, NavigationAgent, HitBox     |
| `level_3d`   | Node3D          | Sun, Environment, Geometry, Spawns           |
| `ui_menu`    | Control         | Container, Title, Start/Options/Quit Buttons |

---

## Claude Desktop 設定

`%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "godot": {
      "command": "C:\\Work\\godot-mcp-rs\\target\\release\\godot-mcp-rs.exe",
      "env": {
        "GODOT_PATH": "C:\\path\\to\\godot.exe"
      }
    }
  }
}
```

## ビルド

```bash
cargo build --release
```
