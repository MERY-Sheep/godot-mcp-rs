@tool
extends RefCounted
## Group Handler
## Handles group operations: add_to_group, remove_from_group, list_groups, get_group_nodes

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"add_to_group":
			return _handle_add_to_group(params)
		"remove_from_group":
			return _handle_remove_from_group(params)
		"list_groups":
			return _handle_list_groups(params)
		"get_group_nodes":
			return _handle_get_group_nodes(params)
		_:
			return {"error": "Unknown group command: " + command}

func _handle_add_to_group(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var node_path = params.get("node_path", ".")
	var group = params.get("group", "")

	if group == "":
		return {"error": "Group name required"}

	var node = root.get_node_or_null(node_path) if node_path != "." else root
	if not node:
		return {"error": "Node not found: " + node_path}

	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Add to Group via LLM: " + group)
	ur.add_do_method(node, "add_to_group", group)
	ur.add_undo_method(node, "remove_from_group", group)
	ur.commit_action()

	return {
		"success": true,
		"node": node_path,
		"group": group,
		"action": "added"
	}

func _handle_remove_from_group(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var node_path = params.get("node_path", ".")
	var group = params.get("group", "")

	if group == "":
		return {"error": "Group name required"}

	var node = root.get_node_or_null(node_path) if node_path != "." else root
	if not node:
		return {"error": "Node not found: " + node_path}

	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Remove from Group via LLM: " + group)
	ur.add_do_method(node, "remove_from_group", group)
	ur.add_undo_method(node, "add_to_group", group)
	ur.commit_action()

	return {
		"success": true,
		"node": node_path,
		"group": group,
		"action": "removed"
	}

func _handle_list_groups(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var node_path = params.get("node_path", ".")

	var node = root.get_node_or_null(node_path) if node_path != "." else root
	if not node:
		return {"error": "Node not found: " + node_path}

	var groups = node.get_groups()

	return {
		"success": true,
		"node": node_path,
		"groups": groups
	}

func _handle_get_group_nodes(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var group = params.get("group", "")
	if group == "":
		return {"error": "Group name required"}

	var result = []
	_find_nodes_in_group(root, group, result)

	return {
		"success": true,
		"group": group,
		"nodes": result
	}

func _find_nodes_in_group(node: Node, group: String, result: Array) -> void:
	if node.is_in_group(group):
		result.append({
			"name": node.name,
			"type": node.get_class(),
			"path": str(node.get_path())
		})

	for child in node.get_children():
		_find_nodes_in_group(child, group, result)
