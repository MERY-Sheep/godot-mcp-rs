# Godot MCP Server - Usage / 使い方

[English](#english) | [日本語](#japanese)

---

<a name="english"></a>

## English

### Tool List (34 tools)

#### Editor & Execution Control (New! 🚀)

| Tool | Description |
| :--- | :--- |
| `get_godot_version` | Get version and path of installed Godot |
| `run_project` | Run project in debug mode (start capturing output) |
| `stop_project` | Force stop the running project |
| `get_debug_output` | Get console output during or after execution |
| `launch_editor` | Launch Godot editor and open project |
| `get_running_status` | Check if project is currently running |

---

#### Project Exploration & Analysis

| Tool | Description |
| :--- | :--- |
| `list_project_files` | List files in the project |
| `read_file` | Read file content as text |
| `list_all_scenes` | Get list of all scenes (.tscn) with root_type |
| `search_in_project` | Search entire project by node_type, resource, script |
| `get_node_type_info` | Get detailed info on Godot node types (properties, methods, recommended children) |
| `get_project_stats` | Get project statistics (file counts, node counts, type aggregation) |
| `validate_project` | Validate entire project (detect parse errors, empty scenes, unused scripts) |

---

#### Scene Manipulation (.tscn)

| Tool | Description |
| :--- | :--- |
| `create_scene` | Create new scene |
| `create_scene_from_template` | Generate scene from template (player_3d, player_2d, enemy_3d, level_3d, ui_menu) |
| `read_scene` | Get scene content as structured JSON |
| `copy_scene` | Copy scene file |
| `add_node` | Add a single node |
| `batch_add_nodes` | Add multiple nodes at once |
| `remove_node` | Remove node at specified path |
| `set_node_property` | Set node properties (position, scale, etc.) |
| `get_node_tree` | Display node parent-child structure as tree |
| `get_scene_metadata` | Get scene statistics and dependencies |
| `validate_tscn` | Check file syntax |
| `compare_scenes` | Show difference between 2 scenes (node add/remove, property changes) |
| `export_scene_as_json` | Convert flat Godot scene to hierarchical JSON |

---

#### Script Manipulation (.gd)

| Tool | Description |
| :--- | :--- |
| `create_script` | Create script (template supported) |
| `attach_script` | Attach script to a node in scene |
| `read_script` | Parse script and get functions, variables, signals as JSON |
| `add_function` | Add new function to existing script |
| `add_export_var` | Add `@export` variable to existing script |
| `analyze_script` | Get summary info (function count, etc.) of script |

---

#### Resource Manipulation (.tres)

| Tool | Description |
| :--- | :--- |
| `list_resources` | List .tres files in project |
| `read_resource` | Parse .tres and get content (external resources, sub-resources, properties) as JSON |

---

### Feature Details

#### Project Execution and Debugging

A loop where AI runs the game itself, checks logs, and fixes issues is possible.

1. Start execution with `run_project`.
2. Check "what is happening" with `get_debug_output`.
3. If there are errors, fix them using parser tools.
4. Stop with `stop_project`.

#### Create Scene from Template

Using `create_scene_from_template`, you can generate commonly used scene structures in one go.

```json
{ "path": "scenes/player.tscn", "template": "player_3d" }
```

| Template | Root Type | Generated Nodes |
| :--- | :--- | :--- |
| `player_3d` | CharacterBody3D | Collision, Mesh, Camera, AnimationPlayer |
| `player_2d` | CharacterBody2D | Collision, Sprite, AnimatedSprite, Camera |
| `enemy_3d` | CharacterBody3D | Collision, Mesh, NavigationAgent, HitBox |
| `level_3d` | Node3D | Sun, Environment, Geometry, Spawns |
| `ui_menu` | Control | Container, Title, Start/Options/Quit Buttons |

---

### Claude Desktop Configuration

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

### Build

```bash
cargo build --release
```

---

<a name="japanese"></a>

## Japanese

### ツール一覧（34 個）

#### エディター・実行制御 (New! 🚀)

| ツール               | 説明                                                     |
| :------------------- | :------------------------------------------------------- |
| `get_godot_version`  | インストール済み Godot のバージョンとパスを取得          |
| `run_project`        | プロジェクトをデバッグモードで実行（出力キャプチャ開始） |
| `stop_project`       | 実行中のプロジェクトを強制終了                           |
| `get_debug_output`   | 実行中または終了後のコンソール出力を取得                 |
| `launch_editor`      | Godot エディターを起動してプロジェクトを開く             |
| `get_running_status` | プロジェクトが現在実行中かどうかを確認                   |

---

#### プロジェクト探索・分析

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

#### シーン操作 (.tscn)

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

#### スクリプト操作 (.gd)

| ツール           | 説明                                                   |
| :--------------- | :----------------------------------------------------- |
| `create_script`  | スクリプト作成（テンプレート対応）                     |
| `attach_script`  | シーン内のノードにスクリプトを接続                     |
| `read_script`    | スクリプトをパースして関数・変数・シグナルを JSON 取得 |
| `add_function`   | 既存スクリプトに新しい関数を追加                       |
| `add_export_var` | 既存スクリプトに `@export` 変数を追加                  |
| `analyze_script` | スクリプトのサマリー情報（関数数など）を取得           |

---

#### リソース操作 (.tres)

| ツール           | 説明                                                                 |
| :--------------- | :------------------------------------------------------------------- |
| `list_resources` | プロジェクト内の .tres ファイル一覧を取得                            |
| `read_resource`  | .tres をパースして内容（外部リソース、サブ、プロパティ）を JSON 取得 |

---

### 主な機能の詳細

#### プロジェクトの実行とデバッグ

AI が自らゲームを実行し、ログを確認して修正するループが可能です。

1. `run_project` で実行開始。
2. `get_debug_output` で「何が起きているか」を確認。
3. エラーがあれば、パーサー系ツールで修正。
4. `stop_project` で停止。

#### テンプレートからシーン作成

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

### Claude Desktop 設定

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

### ビルド

```bash
cargo build --release
```
