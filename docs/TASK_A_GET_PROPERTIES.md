# タスク A: ノードプロパティ一括取得

## 概要

開いているシーンのノードから全プロパティを JSON 形式で取得するコマンドを追加する。

## 実装内容

### 1. Godot プラグイン側

**ファイル**: `addons/godot_mcp/command_handler.gd`

```gdscript
# match文に追加
"get_properties":
    return _handle_get_properties(params)

# ハンドラ関数
func _handle_get_properties(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var node_path = params.get("node_path", ".")
    var node = root.get_node_or_null(node_path) if node_path != "." else root

    if not node:
        return {"error": "Node not found: " + node_path}

    var properties = {}
    for prop in node.get_property_list():
        var name = prop["name"]
        var value = node.get(name)
        # シリアライズ可能な値のみ
        if typeof(value) in [TYPE_NIL, TYPE_BOOL, TYPE_INT, TYPE_FLOAT, TYPE_STRING]:
            properties[name] = value
        elif typeof(value) == TYPE_VECTOR2:
            properties[name] = {"x": value.x, "y": value.y}
        elif typeof(value) == TYPE_VECTOR3:
            properties[name] = {"x": value.x, "y": value.y, "z": value.z}
        elif typeof(value) == TYPE_COLOR:
            properties[name] = {"r": value.r, "g": value.g, "b": value.b, "a": value.a}

    return {
        "success": true,
        "node": node_path,
        "type": node.get_class(),
        "properties": properties
    }
```

### 2. Rust CLI 側

**ファイル**: `src/cli.rs`

```rust
// enum ToolCommands に追加
/// Get all properties of a node via the Godot plugin
LiveGetProperties {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = ".")]
    node_path: String,
},

// run_cli 関数の match に追加
ToolCommands::LiveGetProperties { port, node_path } => {
    return run_live_command(port, "get_properties", serde_json::json!({
        "node_path": node_path
    })).await;
}
```

### 3. テスト方法

```bash
# ビルド
cargo build --release

# テスト（プラグイン再読み込み後）
.\target\release\godot-mcp-rs.exe tool live-get-properties --node-path .
.\target\release\godot-mcp-rs.exe tool live-get-properties --node-path Button
```

### 4. 期待される出力

```json
{
  "success": true,
  "node": "Button",
  "type": "Button",
  "properties": {
    "text": "Click Me",
    "disabled": false,
    "toggle_mode": false,
    "size_flags_horizontal": 1,
    "size_flags_vertical": 1
  }
}
```

## 工数見積もり

- Godot プラグイン: 30 分
- Rust CLI: 15 分
- テスト: 15 分
- **合計: 1 時間**
