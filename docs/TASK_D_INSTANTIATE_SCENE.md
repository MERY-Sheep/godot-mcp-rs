# タスク D: シーンインスタンス化

## 概要

PackedScene をインスタンス化してシーンに追加するコマンドを追加する（プレハブ配置）。

## 実装内容

### 1. 追加するコマンド

| コマンド            | 説明                           |
| ------------------- | ------------------------------ |
| `instantiate_scene` | シーンをインスタンス化して追加 |

### 2. Godot プラグイン側

**ファイル**: `addons/godot_mcp/command_handler.gd`

```gdscript
# match文に追加
"instantiate_scene":
    return _handle_instantiate_scene(params)

# ハンドラ関数
func _handle_instantiate_scene(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var scene_path = params.get("scene_path", "")
    var parent_path = params.get("parent", ".")
    var instance_name = params.get("name", "")
    var position = params.get("position", null)

    if scene_path == "":
        return {"error": "scene_path is required"}

    # res:// プレフィックスを追加
    if not scene_path.begins_with("res://"):
        scene_path = "res://" + scene_path

    # シーンを読み込み
    var packed_scene = load(scene_path)
    if not packed_scene:
        return {"error": "Failed to load scene: " + scene_path}

    # インスタンス化
    var instance = packed_scene.instantiate()
    if not instance:
        return {"error": "Failed to instantiate scene"}

    # 名前を設定
    if instance_name != "":
        instance.name = instance_name

    # 位置を設定（3Dノードの場合）
    if position and instance is Node3D:
        instance.position = Vector3(
            position.get("x", 0),
            position.get("y", 0),
            position.get("z", 0)
        )
    elif position and instance is Node2D:
        instance.position = Vector2(
            position.get("x", 0),
            position.get("y", 0)
        )

    # 親ノードを取得
    var parent = root.get_node_or_null(parent_path) if parent_path != "." else root
    if not parent:
        instance.queue_free()
        return {"error": "Parent node not found: " + parent_path}

    # Undo/Redo 対応
    var ur = plugin.get_undo_redo()
    ur.create_action("Instantiate Scene via LLM: " + scene_path)
    ur.add_do_method(parent, "add_child", instance)
    ur.add_do_property(instance, "owner", root)
    ur.add_do_reference(instance)
    ur.add_undo_method(parent, "remove_child", instance)
    ur.commit_action()

    return {
        "success": true,
        "scene": scene_path,
        "instance_name": instance.name,
        "instance_path": str(instance.get_path())
    }
```

### 3. Rust CLI 側

**ファイル**: `src/cli.rs`

```rust
// enum ToolCommands に追加
/// Instantiate a scene (like placing a prefab)
LiveInstantiateScene {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long)]
    scene_path: String,
    #[arg(long, default_value = ".")]
    parent: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    x: Option<f32>,
    #[arg(long)]
    y: Option<f32>,
    #[arg(long)]
    z: Option<f32>,
},

// run_cli 関数の match に追加
ToolCommands::LiveInstantiateScene { port, scene_path, parent, name, x, y, z } => {
    let mut params = serde_json::json!({
        "scene_path": scene_path,
        "parent": parent
    });

    if let Some(n) = name {
        params["name"] = serde_json::Value::String(n);
    }

    if x.is_some() || y.is_some() || z.is_some() {
        params["position"] = serde_json::json!({
            "x": x.unwrap_or(0.0),
            "y": y.unwrap_or(0.0),
            "z": z.unwrap_or(0.0)
        });
    }

    return run_live_command(port, "instantiate_scene", params).await;
}
```

### 4. テスト方法

```bash
# ビルド
cargo build --release

# テスト（enemy.tscn を main.tscn に配置）
.\target\release\godot-mcp-rs.exe tool live-instantiate-scene --scene-path "scenes/enemy.tscn" --name Enemy1

# 位置を指定して配置
.\target\release\godot-mcp-rs.exe tool live-instantiate-scene --scene-path "scenes/enemy.tscn" --name Enemy2 --x 5 --y 0 --z 10

# 特定の親ノード下に配置
.\target\release\godot-mcp-rs.exe tool live-instantiate-scene --scene-path "scenes/enemy.tscn" --parent "Level" --name SpawnedEnemy
```

### 5. 期待される出力

```json
{
  "success": true,
  "scene": "res://scenes/enemy.tscn",
  "instance_name": "Enemy1",
  "instance_path": "/root/Main/Enemy1"
}
```

## 工数見積もり

- Godot プラグイン: 30 分
- Rust CLI: 20 分
- テスト: 10 分
- **合計: 1 時間**

## ユースケース

1. **敵のスポーン**: LLM が敵シーンを動的に配置
2. **レベルデザイン**: プレハブを座標指定で配置
3. **UI 要素追加**: ボタンやパネルをテンプレートから追加
