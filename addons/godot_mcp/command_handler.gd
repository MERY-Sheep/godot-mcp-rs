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
			return {"success": true, "message": "pong", "version": "1.0.0"}
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
