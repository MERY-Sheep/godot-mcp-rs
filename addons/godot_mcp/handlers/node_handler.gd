@tool
extends RefCounted
## Node Handler
## Handles node operations: add, remove, duplicate, rename, reparent

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"add_node":
			return _handle_add_node(params)
		"remove_node":
			return _handle_remove_node(params)
		"duplicate_node":
			return _handle_duplicate_node(params)
		"rename_node":
			return _handle_rename_node(params)
		"reparent_node":
			return _handle_reparent_node(params)
		_:
			return {"error": "Unknown node command: " + command}

func _handle_add_node(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var parent_path = params.get("parent", ".")
	var node_name = params.get("name", "NewNode")
	var node_type = params.get("node_type", "Node")
	
	var parent = root.get_node_or_null(parent_path) if parent_path != "." else root
	if not parent:
		return {"error": "Parent node not found: " + parent_path}
	
	var new_node = ClassDB.instantiate(node_type)
	if not new_node:
		return {"error": "Failed to create node type: " + node_type}
	
	new_node.name = node_name
	
	# Undo/Redo support
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
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Remove Node via LLM: " + node.name)
	ur.add_do_method(parent, "remove_child", node)
	ur.add_undo_method(parent, "add_child", node)
	ur.add_undo_reference(node)
	ur.commit_action()
	
	return {"success": true, "removed": node_path}

func _handle_duplicate_node(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", "")
	if node_path == "":
		return {"error": "Node path required"}
	
	var node = root.get_node_or_null(node_path)
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var duplicate = node.duplicate()
	if not duplicate:
		return {"error": "Failed to duplicate node"}
	
	var parent = node.get_parent()
	duplicate.name = node.name + "_copy"
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Duplicate Node via LLM: " + node.name)
	ur.add_do_method(parent, "add_child", duplicate)
	ur.add_do_property(duplicate, "owner", root)
	ur.add_do_reference(duplicate)
	ur.add_undo_method(parent, "remove_child", duplicate)
	ur.commit_action()
	
	_set_owner_recursive(duplicate, root)
	
	return {
		"success": true,
		"original": node_path,
		"duplicate_name": duplicate.name,
		"duplicate_path": str(duplicate.get_path())
	}

func _handle_rename_node(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", "")
	var new_name = params.get("new_name", "")
	
	if node_path == "" or new_name == "":
		return {"error": "node_path and new_name required"}
	
	var node = root.get_node_or_null(node_path)
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var old_name = node.name
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Rename Node via LLM: " + old_name + " -> " + new_name)
	ur.add_do_property(node, "name", new_name)
	ur.add_undo_property(node, "name", old_name)
	ur.commit_action()
	
	return {
		"success": true,
		"old_name": old_name,
		"new_name": new_name,
		"new_path": str(node.get_path())
	}

func _handle_reparent_node(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", "")
	var new_parent_path = params.get("new_parent", ".")
	
	if node_path == "":
		return {"error": "node_path required"}
	
	var node = root.get_node_or_null(node_path)
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var new_parent = root.get_node_or_null(new_parent_path) if new_parent_path != "." else root
	if not new_parent:
		return {"error": "New parent not found: " + new_parent_path}
	
	var old_parent = node.get_parent()
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Reparent Node via LLM: " + node.name)
	ur.add_do_method(old_parent, "remove_child", node)
	ur.add_do_method(new_parent, "add_child", node)
	ur.add_do_property(node, "owner", root)
	ur.add_undo_method(new_parent, "remove_child", node)
	ur.add_undo_method(old_parent, "add_child", node)
	ur.add_undo_property(node, "owner", root)
	ur.commit_action()
	
	return {
		"success": true,
		"node": node.name,
		"old_parent": str(old_parent.get_path()),
		"new_parent": str(new_parent.get_path()),
		"new_path": str(node.get_path())
	}

func _set_owner_recursive(node: Node, owner: Node) -> void:
	node.owner = owner
	for child in node.get_children():
		_set_owner_recursive(child, owner)
