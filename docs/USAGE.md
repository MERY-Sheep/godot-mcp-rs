# Godot MCP Server - 使い方

# Godot MCP Server - 使い方

## ツール一覧 (全 56 個)

### ✨ リアルタイム操作 (live-\*)

Godot エディター内で動作するプラグインと連携し、実行中のエディターに対して直接操作を行います。

| カテゴリ           | ツール                     | 説明                                   |
| :----------------- | :------------------------- | :------------------------------------- |
| **基本**           | `live-ping`                | プラグインとの接続確認                 |
| **ノード**         | `live-add-node`            | ノードをエディターに追加               |
|                    | `live-remove-node`         | ノードをエディターから削除             |
|                    | `live-rename-node`         | ノード名を変更                         |
|                    | `live-duplicate-node`      | ノードを複製                           |
|                    | `live-reparent-node`       | ノードの親を変更                       |
|                    | `live-instantiate-scene`   | シーンをインスタンスとして追加         |
| **プロパティ**     | `live-get-properties`      | ノードの全プロパティを取得             |
|                    | `live-set-property`        | ノードのプロパティをリアルタイム変更   |
| **シーン**         | `live-get-tree`            | 現在のエディター上のノードツリーを取得 |
|                    | `live-save-scene`          | 編集中のシーンを保存                   |
|                    | `live-open-scene`          | 指定したシーンをエディターで開く ✨    |
| **シグナル**       | `live-connect-signal`      | シグナルを接続                         |
|                    | `live-disconnect-signal`   | シグナルを切断                         |
|                    | `live-list-signals`        | ノードのシグナルと接続一覧を取得       |
| **アニメーション** | `live-create-animation`    | 新規アニメーションの作成               |
|                    | `live-add-animation-track` | アニメーションにトラックを追加         |
|                    | `live-add-animation-key`   | トラックにキーフレームを追加           |
|                    | `live-play-animation`      | アニメーションを再生                   |
|                    | `live-stop-animation`      | アニメーションを停止                   |
|                    | `live-list-animations`     | アニメーションリストの取得             |
| **デバッグ**       | `live-get-editor-log`      | エディターのログを取得（制限あり）     |
|                    | `live-clear-editor-log`    | エディターのログをクリア               |
| **プラグイン**     | `live-reload-plugin`       | プラグインを再読み込み ✨              |

---

### 🎮 エディター・実行制御 (ファイルベース/プロセス制御)

| ツール               | 説明                                                     |
| :------------------- | :------------------------------------------------------- |
| `get_godot_version`  | インストール済み Godot のバージョンとパスを取得          |
| `run_project`        | プロジェクトをデバッグモードで実行（出力キャプチャ開始） |
| `stop_project`       | 実行中のプロジェクトを強制終了                           |
| `get_debug_output`   | 実行中または終了後のコンソール出力を取得                 |
| `launch_editor`      | Godot エディターを起動してプロジェクトを開く             |
| `get_running_status` | プロジェクトが現在実行中かどうかを確認                   |
| `read-godot-log`     | プロジェクトの Godot ログファイルを読み取り ✨           |

---

### 🏗️ シーン操作 (.tscn)

| ツール                       | 説明                                                    |
| :--------------------------- | :------------------------------------------------------ |
| `create_scene`               | 新規シーン作成                                          |
| `create_scene_from_template` | テンプレートからシーン生成（player_3d, enemy_3d, etc.） |
| `read_scene`                 | シーン内容を構造化 JSON として取得                      |
| `add_node`                   | 単一ノード（ファイル内）を追加                          |
| `remove_node`                | 指定パスのノードを削除                                  |
| `set_node_property`          | ノードのプロパティ（position, scale 等）を設定          |
| `get_node_tree`              | ノードの親子構造をツリー形式で表示                      |
| `export_scene_as_json`       | 平坦な Godot シーンを階層構造 JSON に変換               |
| `compare_scenes`             | 2 つのシーンの差分を表示                                |

---

### 📜 スクリプト・プロジェクト分析

| ツール               | 説明                               |
| :------------------- | :--------------------------------- |
| `create_script`      | スクリプト作成（テンプレート対応） |
| `attach_script`      | シーン内のノードにスクリプトを接続 |
| `read_script`        | 関数・変数・シグナルを JSON 取得   |
| `add_function`       | 既存スクリプトに新しい関数を追加   |
| `analyze_script`     | スクリプトのサマリー情報を取得     |
| `get_project_stats`  | プロジェクト統計を取得             |
| `validate_project`   | プロジェクト全体を検証             |
| `get_node_type_info` | Godot ノード型の詳細情報を取得     |

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
