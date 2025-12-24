# Godot MCP Server - Usage Guide

The main operations of this project are integrated into the GraphQL (GQL) interface. While legacy tools remain available via CLI, we recommend using GQL tools for standard MCP use and advanced automation.

## GraphQL CLI Commands (Recommended)

The latest toolset (GQL) can be executed directly from the CLI. This is recommended over using legacy tools.

### 1. Execute Query (`gql-query`)

```bash
godot-mcp-rs tool gql-query --project ./path/to/project --query "{ project { name stats { sceneCount } } }"
```

### 2. Execute Mutation (`gql-mutate`)

```bash
godot-mcp-rs tool gql-mutate --project ./path/to/project --mutation "mutation { validateMutation(input: { operations: [] }) { isValid } }"
```

### 3. Get Schema (`gql-introspect`)

```bash
godot-mcp-rs tool gql-introspect --project ./path/to/project --format SDL
```

## Legacy Toolset (Internal Implementation)

### ✨ Real-time Operations (live-\*)

Works with the plugin running inside the Godot Editor to perform direct operations on the active editor.

| Category       | Tool                       | Description                                |
| :------------- | :------------------------- | :----------------------------------------- |
| **Basic**      | `live_ping`                | Connection check with the plugin           |
| **Node**       | `live_add_node`            | Add a node to the editor                   |
|                | `live_remove_node`         | Remove a node from the editor              |
|                | `live_instantiate_scene`   | Add a scene as an instance                 |
| **Properties** | `live_set_property`        | Change node properties in real-time        |
| **Scene**      | `live_get_tree`            | Get the node tree in the current editor    |
|                | `live_save_scene`          | Save the scene currently being edited      |
|                | `live_open_scene`          | Open a specified scene in the editor ✨    |
| **Signals**    | `live_connect_signal`      | Connect a signal                           |
|                | `live_disconnect_signal`   | Disconnect a signal                        |
|                | `live_list_signals`        | Get a list of node signals and connections |
| **Animation**  | `live_create_animation`    | Create a new animation                     |
|                | `live_add_animation_track` | Add a track to an animation                |
|                | `live_add_animation_key`   | Add a keyframe to a track                  |
|                | `live_play_animation`      | Play an animation                          |
|                | `live_stop_animation`      | Stop an animation                          |
|                | `live_list_animations`     | Get a list of animations                   |
| **Groups**     | `live_add_to_group`        | Add a node to a group                      |
|                | `live_remove_from_group`   | Remove a node from a group                 |
|                | `live_list_groups`         | Get a list of node groups                  |
|                | `live_get_group_nodes`     | Get all nodes in a group                   |
| **Debug**      | `live_get_editor_log`      | Get editor logs                            |
|                | `live_clear_editor_log`    | Clear editor logs                          |
| **Plugin**     | `live_reload_plugin`       | Reload the plugin ✨                       |

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
| `read-godot-log`     | Read the project's Godot log file ✨                  |

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
