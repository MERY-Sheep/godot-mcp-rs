@tool
extends RefCounted
## Signal Handler  
## Handles signal operations: connect, disconnect, list_signals

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"connect_signal":
			return _handle_connect_signal(params)
		"disconnect_signal":
			return _handle_disconnect_signal(params)
		"list_signals":
			return _handle_list_signals(params)
		_:
			return {"error": "Unknown signal command: " + command}

func _handle_connect_signal(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var source_path = params.get("source", ".")
	var signal_name = params.get("signal", "")
	var target_path = params.get("target", ".")
	var method_name = params.get("method", "")
	
	if signal_name == "" or method_name == "":
		return {"error": "signal and method required"}
	
	var source = root.get_node_or_null(source_path) if source_path != "." else root
	var target = root.get_node_or_null(target_path) if target_path != "." else root
	
	if not source:
		return {"error": "Source node not found: " + source_path}
	if not target:
		return {"error": "Target node not found: " + target_path}
	
	if not source.has_signal(signal_name):
		return {"error": "Source node has no signal: " + signal_name}
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Connect Signal via LLM: " + signal_name)
	ur.add_do_method(source, "connect", signal_name, Callable(target, method_name))
	ur.add_undo_method(source, "disconnect", signal_name, Callable(target, method_name))
	ur.commit_action()
	
	return {
		"success": true,
		"source": source_path,
		"signal": signal_name,
		"target": target_path,
		"method": method_name
	}

func _handle_disconnect_signal(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var source_path = params.get("source", ".")
	var signal_name = params.get("signal", "")
	var target_path = params.get("target", ".")
	var method_name = params.get("method", "")
	
	if signal_name == "" or method_name == "":
		return {"error": "signal and method required"}
	
	var source = root.get_node_or_null(source_path) if source_path != "." else root
	var target = root.get_node_or_null(target_path) if target_path != "." else root
	
	if not source:
		return {"error": "Source node not found: " + source_path}
	if not target:
		return {"error": "Target node not found: " + target_path}
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Disconnect Signal via LLM: " + signal_name)
	ur.add_do_method(source, "disconnect", signal_name, Callable(target, method_name))
	ur.add_undo_method(source, "connect", signal_name, Callable(target, method_name))
	ur.commit_action()
	
	return {
		"success": true,
		"source": source_path,
		"signal": signal_name,
		"target": target_path,
		"method": method_name,
		"action": "disconnected"
	}

func _handle_list_signals(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", ".")
	var node = root.get_node_or_null(node_path) if node_path != "." else root
	
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var signals = []
	for sig in node.get_signal_list():
		signals.append({
			"name": sig["name"],
			"args": sig["args"].map(func(a): return a["name"])
		})
	
	var connections = []
	for conn in node.get_signal_connection_list(node_path):
		connections.append({
			"signal": conn["signal"],
			"target": str(conn["callable"].get_object().get_path()),
			"method": conn["callable"].get_method()
		})
	
	return {
		"success": true,
		"node": node_path,
		"type": node.get_class(),
		"signals": signals,
		"connections": connections
	}
