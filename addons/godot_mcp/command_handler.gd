@tool
extends Node

## コマンドハンドラ
## MCP からのコマンドを受け取り、エディター API で実行

var plugin: EditorPlugin

func handle_command(data: Dictionary) -> Dictionary:
	var command = data.get("command", "")
	var params = data.get("params", {})
	
	match command:
		"ping":
			return {"success": true, "message": "pong", "version": "1.1.0"}
		"add_node":
			return _handle_add_node(params)
		"remove_node":
			return _handle_remove_node(params)
		"set_property":
			return _handle_set_property(params)
		"get_tree":
			return _handle_get_tree(params)
		"save_scene":
			return _handle_save_scene(params)
		"connect_signal":
			return _handle_connect_signal(params)
		"disconnect_signal":
			return _handle_disconnect_signal(params)
		"list_signals":
			return _handle_list_signals(params)
		_:
			return {"error": "Unknown command: " + command}

func _handle_add_node(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var parent_path = params.get("parent", ".")
	var parent = root.get_node(parent_path) if parent_path != "." else root
	if not parent:
		return {"error": "Parent node not found: " + parent_path}
	
	var node_type = params.get("node_type", "Node")
	var node_name = params.get("name", "NewNode")
	
	# ノードを作成
	var new_node = ClassDB.instantiate(node_type)
	if not new_node:
		return {"error": "Failed to instantiate node type: " + node_type}
	
	new_node.name = node_name
	new_node.set_owner(root)
	
	# Undo/Redo 対応
	var ur = plugin.get_undo_redo()
	ur.create_action("Add Node via LLM: " + node_name)
	ur.add_do_method(parent, "add_child", new_node)
	ur.add_do_property(new_node, "owner", root)
	ur.add_do_reference(new_node)
	ur.add_undo_method(parent, "remove_child", new_node)
	ur.commit_action()
	
	return {"success": true, "node_path": str(new_node.get_path())}

func _handle_remove_node(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", "")
	if node_path == "" or node_path == ".":
		return {"error": "Cannot remove root node"}
	
	var node = root.get_node_or_null(node_path)
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var parent = node.get_parent()
	
	# Undo/Redo 対応
	var ur = plugin.get_undo_redo()
	ur.create_action("Remove Node via LLM: " + node.name)
	ur.add_do_method(parent, "remove_child", node)
	ur.add_undo_method(parent, "add_child", node)
	ur.add_undo_reference(node)
	ur.commit_action()
	
	return {"success": true, "removed": node_path}

func _handle_set_property(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", ".")
	var node = root.get_node_or_null(node_path) if node_path != "." else root
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var property = params.get("property", "")
	var value = params.get("value")
	
	if property == "":
		return {"error": "Property name is required"}
	
	var old_value = node.get(property)
	
	# Undo/Redo 対応
	var ur = plugin.get_undo_redo()
	ur.create_action("Set Property via LLM: " + property)
	ur.add_do_property(node, property, value)
	ur.add_undo_property(node, property, old_value)
	ur.commit_action()
	
	return {"success": true, "property": property, "value": value}

func _handle_get_tree(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var tree = _build_tree(root)
	return {"success": true, "tree": tree}

func _build_tree(node: Node, indent: int = 0) -> Array:
	var result = []
	var entry = {
		"name": node.name,
		"type": node.get_class(),
		"path": str(node.get_path()),
		"indent": indent
	}
	result.append(entry)
	
	for child in node.get_children():
		result.append_array(_build_tree(child, indent + 1))
	
	return result

func _handle_save_scene(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var scene_path = root.scene_file_path
	if scene_path == "":
		return {"error": "Scene has no file path"}
	
	var packed_scene = PackedScene.new()
	packed_scene.pack(root)
	var err = ResourceSaver.save(packed_scene, scene_path)
	
	if err != OK:
		return {"error": "Failed to save scene: " + str(err)}
	
	return {"success": true, "path": scene_path}

# === Signal Commands ===

func _handle_connect_signal(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var source_path = params.get("source", ".")
	var signal_name = params.get("signal", "")
	var target_path = params.get("target", ".")
	var method_name = params.get("method", "")
	
	if signal_name == "":
		return {"error": "Signal name is required"}
	if method_name == "":
		return {"error": "Method name is required"}
	
	var source = root.get_node_or_null(source_path) if source_path != "." else root
	var target = root.get_node_or_null(target_path) if target_path != "." else root
	
	if not source:
		return {"error": "Source node not found: " + source_path}
	if not target:
		return {"error": "Target node not found: " + target_path}
	
	# シグナルが存在するか確認
	if not source.has_signal(signal_name):
		return {"error": "Signal not found: " + signal_name + " on " + source_path}
	
	# 既に接続されているか確認
	if source.is_connected(signal_name, Callable(target, method_name)):
		return {"error": "Signal already connected"}
	
	# Undo/Redo 対応
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
	
	if signal_name == "":
		return {"error": "Signal name is required"}
	if method_name == "":
		return {"error": "Method name is required"}
	
	var source = root.get_node_or_null(source_path) if source_path != "." else root
	var target = root.get_node_or_null(target_path) if target_path != "." else root
	
	if not source:
		return {"error": "Source node not found: " + source_path}
	if not target:
		return {"error": "Target node not found: " + target_path}
	
	# 接続されているか確認
	if not source.is_connected(signal_name, Callable(target, method_name)):
		return {"error": "Signal is not connected"}
	
	# Undo/Redo 対応
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
		var sig_info = {
			"name": sig["name"],
			"args": []
		}
		for arg in sig["args"]:
			sig_info["args"].append(arg["name"])
		signals.append(sig_info)
	
	# 接続情報も取得
	var connections = []
	for conn in node.get_incoming_connections():
		connections.append({
			"signal": conn["signal"].get_name(),
			"from": str(conn["signal"].get_object().get_path()) if conn["signal"].get_object() else "unknown",
			"method": conn["callable"].get_method()
		})
	
	for sig_name in node.get_signal_list().map(func(s): return s["name"]):
		for conn in node.get_signal_connection_list(sig_name):
			connections.append({
				"signal": sig_name,
				"from": str(node.get_path()),
				"to": str(conn["callable"].get_object().get_path()) if conn["callable"].get_object() else "unknown",
				"method": conn["callable"].get_method()
			})
	
	return {
		"success": true,
		"node": node_path,
		"type": node.get_class(),
		"signals": signals,
		"connections": connections
	}

