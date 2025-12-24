# Godot MCP Server - 使い方

> [!WARNING] > **重要: ツールセットの集約**
> 現在、本リポジトリの MCP ツールは GraphQL (GQL) 3 ツール（`godot_query`, `godot_mutate`, `godot_introspect`）に集約されています。
> 以下の個別ツール群は MCP サーバーからは削除されましたが、CLI モードでのデバッグ用、および GQL リゾルバの内部実装として引き続き利用・参照可能です。
> 新しい使い方については [MIGRATION_GUIDE.md](gql/MIGRATION_GUIDE.md) を参照してください。

## GraphQL CLI コマンド (推奨)

最新のツールセット（GQL）は CLI からも直接実行可能です。レガシーツールを使用するよりも、こちらの方が推奨されます。

### 1. クエリの実行 (`gql-query`)

```bash
godot-mcp-rs tool gql-query --project ./path/to/project --query "{ project { name stats { sceneCount } } }"
```

### 2. ミューテーションの実行 (`gql-mutate`)

```bash
godot-mcp-rs tool gql-mutate --project ./path/to/project --mutation "mutation { validateMutation(input: { operations: [] }) { isValid } }"
```

### 3. スキーマの取得 (`gql-introspect`)

```bash
godot-mcp-rs tool gql-introspect --project ./path/to/project --format SDL
```

## ツール一覧 (レガシー/内部実装)

### ✨ リアルタイム操作 (live-\*)

Godot エディター内で動作するプラグインと連携し、実行中のエディターに対して直接操作を行います。

| カテゴリ           | ツール                     | 説明                                   |
| :----------------- | :------------------------- | :------------------------------------- |
| **基本**           | `live_ping`                | プラグインとの接続確認                 |
| **ノード**         | `live_add_node`            | ノードをエディターに追加               |
|                    | `live_remove_node`         | ノードをエディターから削除             |
|                    | `live_instantiate_scene`   | シーンをインスタンスとして追加         |
| **プロパティ**     | `live_set_property`        | ノードのプロパティをリアルタイム変更   |
| **シーン**         | `live_get_tree`            | 現在のエディター上のノードツリーを取得 |
|                    | `live_save_scene`          | 編集中のシーンを保存                   |
|                    | `live_open_scene`          | 指定したシーンをエディターで開く ✨    |
| **シグナル**       | `live_connect_signal`      | シグナルを接続                         |
|                    | `live_disconnect_signal`   | シグナルを切断                         |
|                    | `live_list_signals`        | ノードのシグナルと接続一覧を取得       |
| **アニメーション** | `live_create_animation`    | 新規アニメーションの作成               |
|                    | `live_add_animation_track` | アニメーションにトラックを追加         |
|                    | `live_add_animation_key`   | トラックにキーフレームを追加           |
|                    | `live_play_animation`      | アニメーションを再生                   |
|                    | `live_stop_animation`      | アニメーションを停止                   |
|                    | `live_list_animations`     | アニメーションリストの取得             |
| **グループ**       | `live_add_to_group`        | ノードをグループに追加                 |
|                    | `live_remove_from_group`   | ノードをグループから削除               |
|                    | `live_list_groups`         | ノードの所属グループ一覧を取得         |
|                    | `live_get_group_nodes`     | グループ全ノードを取得                 |
| **デバッグ**       | `live_get_editor_log`      | エディターのログを取得                 |
|                    | `live_clear_editor_log`    | エディターのログをクリア               |
| **プラグイン**     | `live_reload_plugin`       | プラグインを再読み込み ✨              |

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

## ✨ GQL 開発・テスト支援 (TDD) (Phase 1)

AI が自律的にテストを回せるようにするための機能です。

### テストの実行 (`runTests`)

GdUnit4 と連携してテストを実行し、結果を構造化データで取得します。

```graphql
mutation {
  runTests(input: { testPath: "res://tests/" }) {
    success
    totalCount
    passedCount
    failedCount
    suites {
      name
      path
      success
      cases {
        name
        success
        message
        line
      }
    }
  }
}
```

---

## ✨ GQL デバッグ・ログ統合 (Phase 2)

AI がゲームの実行状態を把握し、問題を解決するための機能です。

### デバッグ情報の取得

デバッガーのエラー、エディターログ、実行時オブジェクトの状態を取得します。

```graphql
query {
  # デバッガーエラーの取得
  debuggerErrors {
    message
    stackInfo {
      file
      line
      function
    }
  }

  # ログの取得
  logs(limit: 20) {
    message
    severity
    timestamp
  }

  # 実行時オブジェクトのインスペクション (ID指定)
  objectById(objectId: "12345") {
    id
    class
    properties {
      name
      value
      type
    }
  }
}
```

### 実行制御

ブレークポイントの設定や、ステップ実行が可能です。

```graphql
mutation {
  # 実行の一時停止
  pause {
    success
  }

  # ステップ実行 (1行進める)
  step {
    success
  }

  # 実行の再開
  resume {
    success
  }

  # ブレークポイントの設定
  setBreakpoint(input: { path: "res://player.gd", line: 42, enabled: true }) {
    success
  }
}
```

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
