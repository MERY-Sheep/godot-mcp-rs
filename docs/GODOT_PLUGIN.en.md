# Godot Editor Plugin (Godot MCP)

## Overview

Executes commands from the MCP server in real-time through a plugin running inside the Godot Editor.
It fully supports Undo/Redo, allowing AI-made changes to be reverted from the editor's history.

## Installation

1. Copy (or symlink) the `addons/godot_mcp` directory of this repository into your target project's `addons/` directory.
2. Open the Godot Editor and select the **Project -> Project Settings -> Plugins** tab.
3. Check the **Enable** checkbox for the **Godot MCP** plugin.
4. It's successful if `Godot MCP Plugin started on port 6060` appears in the Output panel.

## Technical Specifications

### Server Configuration

- **Protocol**: Simple TCP server (HTTP-like JSON POST)
- **Port**: Default `6060` (changeable)
- **Scripts**: `plugin.gd` manages the server, while `command_handler.gd` processes commands.

### Supported Commands (Live Commands)

Currently, the following commands support real-time operations:

| Category       | Command                                                                                           | Undo/Redo |
| :------------- | :------------------------------------------------------------------------------------------------ | :-------: |
| **Node**       | `add_node`, `remove_node`, `rename_node`, `duplicate_node`, `reparent_node`, `instantiate_scene`  |    ✅     |
| **Properties** | `set_property`                                                                                    |    ✅     |
|                | `get_properties`                                                                                  |     -     |
| **Scene**      | `get_tree`, `save_scene`                                                                          |     -     |
| **Signals**    | `connect_signal`, `disconnect_signal`                                                             |    ✅     |
|                | `list_signals`                                                                                    |     -     |
| **Animation**  | `create_animation`                                                                                |    ✅     |
|                | `add_animation_track`, `add_animation_key`, `play_animation`, `stop_animation`, `list_animations` |     -     |
| **Debug**      | `get_editor_log`, `clear_editor_log`                                                              |     -     |

## About Undo/Redo

All destructive operations (adding nodes, changing properties, connecting signals, etc.) are executed via Godot's `EditorUndoRedoManager`.
This allows operations performed by the AI to be individually undone or redone using `Ctrl+Z`.

## For Developers: Adding New Commands

1. Add a new case to the `handle_command` match statement in `command_handler.gd`.
2. Implement the corresponding `_handle_xxxx` function, using `ur.create_action()` to register Undo/Redo as needed.
3. Add the corresponding `LiveXXXX` command to the Rust CLI (`src/cli.rs`).
