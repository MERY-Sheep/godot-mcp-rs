# Godot MCP Server - Usage

## Tool List (56 tools total)

### ✨ Real-time Operations (live-\*)

Works with the plugin running inside the Godot Editor to perform direct operations on the active editor.

| Category       | Tool                       | Description                                |
| :------------- | :------------------------- | :----------------------------------------- |
| **Basic**      | `live-ping`                | Connection check with the plugin           |
| **Node**       | `live-add-node`            | Add a node to the editor                   |
|                | `live-remove-node`         | Remove a node from the editor              |
|                | `live-rename-node`         | Rename a node                              |
|                | `live-duplicate-node`      | Duplicate a node                           |
|                | `live-reparent-node`       | Change a node's parent                     |
|                | `live-instantiate-scene`   | Add a scene as an instance                 |
| **Properties** | `live-get-properties`      | Get all properties of a node               |
|                | `live-set-property`        | Change node properties in real-time        |
| **Scene**      | `live-get-tree`            | Get the node tree in the current editor    |
|                | `live-save-scene`          | Save the scene currently being edited      |
| **Signals**    | `live-connect-signal`      | Connect a signal                           |
|                | `live-disconnect-signal`   | Disconnect a signal                        |
|                | `live-list-signals`        | Get a list of node signals and connections |
| **Animation**  | `live-create-animation`    | Create a new animation                     |
|                | `live-add-animation-track` | Add a track to an animation                |
|                | `live-add-animation-key`   | Add a keyframe to a track                  |
|                | `live-play-animation`      | Play an animation                          |
|                | `live-stop-animation`      | Stop an animation                          |
|                | `live-list-animations`     | Get a list of animations                   |
| **Debug**      | `live-get-editor-log`      | Get editor logs                            |
|                | `live-clear-editor-log`    | Clear editor logs                          |

---

### 🎮 Editor & Execution Control (File-based/Process Control)

| Tool                 | Description                                           |
| :------------------- | :---------------------------------------------------- |
| `get_godot_version`  | Get installed Godot version and path                  |
| `run_project`        | Run the project in debug mode (starts output capture) |
| `stop_project`       | Force-stop the running project                        |
| `get_debug_output`   | Get console output during or after execution          |
| `launch_editor`      | Launch Godot Editor and open the project              |
| `get_running_status` | Check if the project is currently running             |

---

### 🏗️ Scene Manipulation (.tscn)

| Tool                         | Description                                              |
| :--------------------------- | :------------------------------------------------------- |
| `create_scene`               | Create a new scene                                       |
| `create_scene_from_template` | Generate scene from template (player_3d, enemy_3d, etc.) |
| `read_scene`                 | Get scene content as structured JSON                     |
| `add_node`                   | Add a single node (in-file)                              |
| `remove_node`                | Remove node at specified path                            |
| `set_node_property`          | Set node properties (position, scale, etc.)              |
| `get_node_tree`              | Display node hierarchy in tree format                    |
| `export_scene_as_json`       | Convert flat Godot scene to hierarchical JSON            |
| `compare_scenes`             | Show differences between two scenes                      |

---

### 📜 Script & Project Analysis

| Tool                 | Description                                  |
| :------------------- | :------------------------------------------- |
| `create_script`      | Create script (template support)             |
| `attach_script`      | Attach script to a node in a scene           |
| `read_script`        | Get functions, variables, signals as JSON    |
| `add_function`       | Add a new function to an existing script     |
| `analyze_script`     | Get summary information for a script         |
| `get_project_stats`  | Get project statistics                       |
| `validate_project`   | Validate the entire project                  |
| `get_node_type_info` | Get detailed information on Godot node types |

---

## Feature Details

### Run and Debug Project

Enables a loop where AI runs the game, checks logs, and makes fixes.

1. Start execution with `run_project`.
2. check "what's happening" with `get_debug_output`.
3. If there's an error, fix it using parser tools.
4. Stop with `stop_project`.

### Create Scene from Template

Use `create_scene_from_template` to instantly generate common scene configurations.

```json
{ "path": "scenes/player.tscn", "template": "player_3d" }
```

| Template    | Root Type       | Generated Nodes                              |
| ----------- | --------------- | -------------------------------------------- |
| `player_3d` | CharacterBody3D | Collision, Mesh, Camera, AnimationPlayer     |
| `player_2d` | CharacterBody2D | Collision, Sprite, AnimatedSprite, Camera    |
| `enemy_3d`  | CharacterBody3D | Collision, Mesh, NavigationAgent, HitBox     |
| `level_3d`  | Node3D          | Sun, Environment, Geometry, Spawns           |
| `ui_menu`   | Control         | Container, Title, Start/Options/Quit Buttons |

---

## Claude Desktop Configuration

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

## Build

```bash
cargo build --release
```
