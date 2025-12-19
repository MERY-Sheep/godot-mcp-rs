# Godot MCP Plugin

This plugin allows real-time manipulation of the Godot Editor from an LLM (AI).

## Installation

1. Copy the `addons/godot_mcp/` folder to your Godot project's `addons/` directory.
2. Enable **Godot MCP** under **Project → Project Settings → Plugins**.
3. It's successful if `Godot MCP: Server started on port 6060` appears in the editor's Output panel.

## Usage

After enabling the plugin, it accepts HTTP requests on port 6060.

### Command List

| Category      | Command                                                                                                               | Description                                                      |
| :------------ | :-------------------------------------------------------------------------------------------------------------------- | :--------------------------------------------------------------- |
| **Basic**     | `ping`                                                                                                                | Connectivity check                                               |
| **Node**      | `add_node`, `remove_node`, `rename_node`, `duplicate_node`, `reparent_node`, `instantiate_scene`                      | Add, remove, rename, duplicate, change parent, instantiate scene |
| **Property**  | `get_properties`, `set_property`                                                                                      | Get all properties, set individual property                      |
| **Scene**     | `get_tree`, `save_scene`                                                                                              | Get node tree, save scene                                        |
| **Signal**    | `connect_signal`, `disconnect_signal`, `list_signals`                                                                 | Connect, disconnect, list signals                                |
| **Animation** | `create_animation`, `add_animation_track`, `add_animation_key`, `play_animation`, `stop_animation`, `list_animations` | Create, edit, control playback, list animations                  |
| **Debug**     | `get_editor_log`, `clear_editor_log`                                                                                  | Get and clear editor logs                                        |

### Usage Example (PowerShell)

```powershell
# ping
Invoke-RestMethod -Uri "http://localhost:6060" -Method POST -Body '{"command":"ping"}' -ContentType "application/json"

# Add node
Invoke-RestMethod -Uri "http://localhost:6060" -Method POST -Body '{"command":"add_node","params":{"parent":".","name":"MyNode","node_type":"Node3D"}}' -ContentType "application/json"

# Get node tree
Invoke-RestMethod -Uri "http://localhost:6060" -Method POST -Body '{"command":"get_tree","params":{}}' -ContentType "application/json"
```

### Usage Example (curl)

```bash
curl -X POST http://localhost:6060 -H "Content-Type: application/json" -d '{"command":"ping"}'
```

## Features

- **Undo/Redo Support**: All operations are integrated into the editor's history.
- **Real-time Reflection**: Node tree changes reflect immediately without file writes.
- **Simple**: Pure HTTP + JSON, no additional dependencies.

## Technical Specifications

- Port: 6060
- Protocol: HTTP POST
- Format: JSON

## License

MIT
