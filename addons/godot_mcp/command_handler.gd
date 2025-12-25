@tool
extends Node

## Command Handler - Router
## Routes commands to domain-specific handlers.
## Decomposed from monolithic command_handler.gd into:
## - node_handler: add, remove, duplicate, rename, reparent
## - scene_handler: save, open, instantiate, get_tree
## - signal_handler: connect, disconnect, list_signals
## - property_handler: set_property, get_properties
## - animation_handler: create, add_track, add_key, play, stop, list
## - debug_handler: logs, errors, pause, resume, step, breakpoints
## - group_handler: add_to_group, remove_from_group, list_groups, get_group_nodes
## - shader_handler: create_visual_shader_node, validate_shader_live

var plugin: EditorPlugin

# Handler instances
var _node_handler
var _scene_handler
var _signal_handler
var _property_handler
var _animation_handler
var _debug_handler
var _group_handler
var _shader_handler
var _introspect_handler

# Command to handler mapping
var _command_handlers: Dictionary = {}

func _ready() -> void:
	_load_handlers()
	_build_command_map()

func _load_handlers() -> void:
	var NodeHandler = load("res://addons/godot_mcp/handlers/node_handler.gd")
	var SceneHandler = load("res://addons/godot_mcp/handlers/scene_handler.gd")
	var SignalHandler = load("res://addons/godot_mcp/handlers/signal_handler.gd")
	var PropertyHandler = load("res://addons/godot_mcp/handlers/property_handler.gd")
	var AnimationHandler = load("res://addons/godot_mcp/handlers/animation_handler.gd")
	var DebugHandler = load("res://addons/godot_mcp/handlers/debug_handler.gd")
	var GroupHandler = load("res://addons/godot_mcp/handlers/group_handler.gd")
	var ShaderHandler = load("res://addons/godot_mcp/handlers/shader_handler.gd")
	var IntrospectHandler = load("res://addons/godot_mcp/handlers/introspect_handler.gd")
	
	_node_handler = NodeHandler.new(plugin)
	_scene_handler = SceneHandler.new(plugin)
	_signal_handler = SignalHandler.new(plugin)
	_property_handler = PropertyHandler.new(plugin)
	_animation_handler = AnimationHandler.new(plugin)
	_debug_handler = DebugHandler.new(plugin)
	_group_handler = GroupHandler.new(plugin)
	_shader_handler = ShaderHandler.new(plugin)
	_introspect_handler = IntrospectHandler.new(plugin)

func _build_command_map() -> void:
	# Node operations
	_command_handlers["add_node"] = _node_handler
	_command_handlers["remove_node"] = _node_handler
	_command_handlers["duplicate_node"] = _node_handler
	_command_handlers["rename_node"] = _node_handler
	_command_handlers["reparent_node"] = _node_handler
	
	# Scene operations
	_command_handlers["save_scene"] = _scene_handler
	_command_handlers["open_scene"] = _scene_handler
	_command_handlers["instantiate_scene"] = _scene_handler
	_command_handlers["get_tree"] = _scene_handler
	
	# Signal operations
	_command_handlers["connect_signal"] = _signal_handler
	_command_handlers["disconnect_signal"] = _signal_handler
	_command_handlers["list_signals"] = _signal_handler
	
	# Property operations
	_command_handlers["set_property"] = _property_handler
	_command_handlers["get_properties"] = _property_handler
	
	# Animation operations
	_command_handlers["create_animation"] = _animation_handler
	_command_handlers["add_animation_track"] = _animation_handler
	_command_handlers["add_animation_key"] = _animation_handler
	_command_handlers["play_animation"] = _animation_handler
	_command_handlers["stop_animation"] = _animation_handler
	_command_handlers["list_animations"] = _animation_handler
	
	# Debug operations
	_command_handlers["get_editor_log"] = _debug_handler
	_command_handlers["get_logs"] = _debug_handler
	_command_handlers["get_debugger_errors"] = _debug_handler
	_command_handlers["get_object_by_id"] = _debug_handler
	_command_handlers["pause"] = _debug_handler
	_command_handlers["resume"] = _debug_handler
	_command_handlers["step"] = _debug_handler
	_command_handlers["set_breakpoint"] = _debug_handler
	_command_handlers["remove_breakpoint"] = _debug_handler
	_command_handlers["clear_editor_log"] = _debug_handler
	
	# Group operations
	_command_handlers["add_to_group"] = _group_handler
	_command_handlers["remove_from_group"] = _group_handler
	_command_handlers["list_groups"] = _group_handler
	_command_handlers["get_group_nodes"] = _group_handler
	
	# Shader operations
	_command_handlers["create_visual_shader_node"] = _shader_handler
	_command_handlers["validate_shader_live"] = _shader_handler
	
	# Phase 3: Debug Enhanced
	_command_handlers["get_parse_errors"] = _debug_handler
	_command_handlers["get_stack_frame_vars"] = _debug_handler
	
	# Introspect operations (Phase 1: Dynamic Type Discovery)
	_command_handlers["get_type_info"] = _introspect_handler
	_command_handlers["list_all_types"] = _introspect_handler

func handle_command(data: Dictionary) -> Dictionary:
	var command = data.get("command", "")
	var params = data.get("params", {})
	
	# Handle ping specially
	if command == "ping":
		return {"success": true, "message": "pong", "version": "1.1.0"}
	
	# Handle reload_plugin specially
	if command == "reload_plugin":
		return _handle_reload_plugin(params)
	
	# Route to appropriate handler
	if _command_handlers.has(command):
		return _command_handlers[command].handle(command, params)
	
	return {"error": "Unknown command: " + command}

func _handle_reload_plugin(_params: Dictionary) -> Dictionary:
	# Get the plugin name
	var plugin_name = "godot_mcp"
	
	# Schedule the reload to happen after returning the response
	call_deferred("_do_reload_plugin", plugin_name)
	
	return {"success": true, "message": "Plugin reload scheduled"}

func _do_reload_plugin(plugin_name: String) -> void:
	EditorInterface.set_plugin_enabled(plugin_name, false)
	
	# Wait for the plugin to fully disable
	await get_tree().create_timer(0.1).timeout
	
	EditorInterface.set_plugin_enabled(plugin_name, true)
	print("[MCP] Plugin reloaded successfully")
