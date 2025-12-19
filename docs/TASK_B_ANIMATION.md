# タスク B: アニメーション操作

## 概要

AnimationPlayer ノードを操作し、アニメーションの作成・編集・再生制御を行うコマンドを追加する。

## 実装内容

### 1. 追加するコマンド

| コマンド              | 説明                       |
| --------------------- | -------------------------- |
| `create_animation`    | 新しいアニメーションを作成 |
| `add_animation_track` | トラックを追加             |
| `add_animation_key`   | キーフレームを追加         |
| `play_animation`      | アニメーションを再生       |
| `stop_animation`      | アニメーションを停止       |
| `list_animations`     | アニメーション一覧を取得   |

### 2. Godot プラグイン側

**ファイル**: `addons/godot_mcp/command_handler.gd`

```gdscript
# match文に追加
"create_animation":
    return _handle_create_animation(params)
"add_animation_track":
    return _handle_add_animation_track(params)
"add_animation_key":
    return _handle_add_animation_key(params)
"play_animation":
    return _handle_play_animation(params)
"stop_animation":
    return _handle_stop_animation(params)
"list_animations":
    return _handle_list_animations(params)

# ハンドラ関数例
func _handle_create_animation(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var player_path = params.get("player", "AnimationPlayer")
    var anim_name = params.get("name", "new_animation")
    var length = params.get("length", 1.0)

    var player = root.get_node_or_null(player_path)
    if not player or not player is AnimationPlayer:
        return {"error": "AnimationPlayer not found: " + player_path}

    var anim = Animation.new()
    anim.length = length

    var ur = plugin.get_undo_redo()
    ur.create_action("Create Animation via LLM: " + anim_name)
    ur.add_do_method(player, "add_animation", anim_name, anim)
    ur.add_undo_method(player, "remove_animation", anim_name)
    ur.commit_action()

    return {"success": true, "animation": anim_name, "length": length}

func _handle_add_animation_track(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var player_path = params.get("player", "AnimationPlayer")
    var anim_name = params.get("animation", "")
    var track_path = params.get("track_path", "")  # e.g., "Sprite2D:position"
    var track_type = params.get("track_type", "value")  # value, position, rotation, scale

    var player = root.get_node_or_null(player_path)
    if not player or not player is AnimationPlayer:
        return {"error": "AnimationPlayer not found: " + player_path}

    var anim = player.get_animation(anim_name)
    if not anim:
        return {"error": "Animation not found: " + anim_name}

    var type_map = {
        "value": Animation.TYPE_VALUE,
        "position": Animation.TYPE_POSITION_3D,
        "rotation": Animation.TYPE_ROTATION_3D,
        "scale": Animation.TYPE_SCALE_3D,
    }

    var idx = anim.add_track(type_map.get(track_type, Animation.TYPE_VALUE))
    anim.track_set_path(idx, track_path)

    return {"success": true, "track_index": idx, "path": track_path}

func _handle_add_animation_key(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var player_path = params.get("player", "AnimationPlayer")
    var anim_name = params.get("animation", "")
    var track_idx = params.get("track", 0)
    var time = params.get("time", 0.0)
    var value = params.get("value")

    var player = root.get_node_or_null(player_path)
    if not player or not player is AnimationPlayer:
        return {"error": "AnimationPlayer not found"}

    var anim = player.get_animation(anim_name)
    if not anim:
        return {"error": "Animation not found: " + anim_name}

    anim.track_insert_key(track_idx, time, value)

    return {"success": true, "track": track_idx, "time": time}

func _handle_play_animation(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var player_path = params.get("player", "AnimationPlayer")
    var anim_name = params.get("animation", "")

    var player = root.get_node_or_null(player_path)
    if not player or not player is AnimationPlayer:
        return {"error": "AnimationPlayer not found"}

    player.play(anim_name)
    return {"success": true, "playing": anim_name}

func _handle_stop_animation(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var player_path = params.get("player", "AnimationPlayer")

    var player = root.get_node_or_null(player_path)
    if not player or not player is AnimationPlayer:
        return {"error": "AnimationPlayer not found"}

    player.stop()
    return {"success": true, "stopped": true}

func _handle_list_animations(params: Dictionary) -> Dictionary:
    var root = EditorInterface.get_edited_scene_root()
    if not root:
        return {"error": "No scene is open"}

    var player_path = params.get("player", "AnimationPlayer")

    var player = root.get_node_or_null(player_path)
    if not player or not player is AnimationPlayer:
        return {"error": "AnimationPlayer not found"}

    var anims = []
    for name in player.get_animation_list():
        var anim = player.get_animation(name)
        anims.append({
            "name": name,
            "length": anim.length,
            "tracks": anim.get_track_count()
        })

    return {"success": true, "animations": anims}
```

### 3. Rust CLI 側

**ファイル**: `src/cli.rs`

```rust
// enum ToolCommands に追加
/// Create a new animation
LiveCreateAnimation {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = "AnimationPlayer")]
    player: String,
    #[arg(long)]
    name: String,
    #[arg(long, default_value = "1.0")]
    length: f32,
},

/// Add a track to an animation
LiveAddAnimationTrack {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = "AnimationPlayer")]
    player: String,
    #[arg(long)]
    animation: String,
    #[arg(long)]
    track_path: String,
    #[arg(long, default_value = "value")]
    track_type: String,
},

/// Add a keyframe to an animation track
LiveAddAnimationKey {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = "AnimationPlayer")]
    player: String,
    #[arg(long)]
    animation: String,
    #[arg(long, default_value = "0")]
    track: u32,
    #[arg(long)]
    time: f32,
    #[arg(long)]
    value: String,
},

/// Play an animation
LivePlayAnimation {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = "AnimationPlayer")]
    player: String,
    #[arg(long)]
    animation: String,
},

/// Stop an animation
LiveStopAnimation {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = "AnimationPlayer")]
    player: String,
},

/// List all animations
LiveListAnimations {
    #[arg(long, default_value = "6060")]
    port: u16,
    #[arg(long, default_value = "AnimationPlayer")]
    player: String,
},

// run_cli 関数の match に追加
ToolCommands::LiveCreateAnimation { port, player, name, length } => {
    return run_live_command(port, "create_animation", serde_json::json!({
        "player": player,
        "name": name,
        "length": length
    })).await;
}
ToolCommands::LiveAddAnimationTrack { port, player, animation, track_path, track_type } => {
    return run_live_command(port, "add_animation_track", serde_json::json!({
        "player": player,
        "animation": animation,
        "track_path": track_path,
        "track_type": track_type
    })).await;
}
ToolCommands::LiveAddAnimationKey { port, player, animation, track, time, value } => {
    return run_live_command(port, "add_animation_key", serde_json::json!({
        "player": player,
        "animation": animation,
        "track": track,
        "time": time,
        "value": value
    })).await;
}
ToolCommands::LivePlayAnimation { port, player, animation } => {
    return run_live_command(port, "play_animation", serde_json::json!({
        "player": player,
        "animation": animation
    })).await;
}
ToolCommands::LiveStopAnimation { port, player } => {
    return run_live_command(port, "stop_animation", serde_json::json!({
        "player": player
    })).await;
}
ToolCommands::LiveListAnimations { port, player } => {
    return run_live_command(port, "list_animations", serde_json::json!({
        "player": player
    })).await;
}
```

### 4. テスト方法

```bash
# ビルド
cargo build --release

# テスト（AnimationPlayerがあるシーンを開いた状態で）
.\target\release\godot-mcp-rs.exe tool live-list-animations

# アニメーション作成
.\target\release\godot-mcp-rs.exe tool live-create-animation --name walk --length 2.0

# トラック追加
.\target\release\godot-mcp-rs.exe tool live-add-animation-track --animation walk --track-path "Sprite2D:position"

# キーフレーム追加
.\target\release\godot-mcp-rs.exe tool live-add-animation-key --animation walk --track 0 --time 0.0 --value "Vector2(0, 0)"
.\target\release\godot-mcp-rs.exe tool live-add-animation-key --animation walk --track 0 --time 1.0 --value "Vector2(100, 0)"

# 再生
.\target\release\godot-mcp-rs.exe tool live-play-animation --animation walk
```

## 工数見積もり

- Godot プラグイン: 2 時間
- Rust CLI: 1 時間
- テスト: 1 時間
- **合計: 4 時間**

## 注意事項

- キーフレームの値は JSON で渡すため、Vector2/Vector3 などの変換処理が必要
- Undo/Redo 対応は create_animation のみ（track/key 追加は複雑なため後回し可）
