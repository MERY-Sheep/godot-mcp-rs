@tool
extends RefCounted
## Scene Handler
## Handles scene operations: save, open, instantiate, get_tree

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"save_scene":
			return _handle_save_scene(params)
		"open_scene":
			return _handle_open_scene(params)
		"instantiate_scene":
			return _handle_instantiate_scene(params)
		"get_tree":
			return _handle_get_tree(params)
		_:
			return {"error": "Unknown scene command: " + command}

func _handle_save_scene(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var scene_path = root.scene_file_path
	if scene_path == "":
		scene_path = params.get("path", "res://new_scene.tscn")
	
	var packed_scene = PackedScene.new()
	var result = packed_scene.pack(root)
	if result != OK:
		return {"error": "Failed to pack scene: " + str(result)}
	
	result = ResourceSaver.save(packed_scene, scene_path)
	if result != OK:
		return {"error": "Failed to save scene: " + str(result)}
	
	return {"success": true, "path": scene_path}

func _handle_open_scene(params: Dictionary) -> Dictionary:
	var scene_path = params.get("path", "")
	if scene_path == "":
		return {"error": "Scene path required"}
	
	if not FileAccess.file_exists(scene_path):
		return {"error": "Scene file not found: " + scene_path}
	
	# Open the scene in the editor
	EditorInterface.open_scene_from_path(scene_path)
	
	return {"success": true, "opened": scene_path}

func _handle_instantiate_scene(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var scene_path = params.get("scene_path", "")
	var parent_path = params.get("parent", ".")
	var instance_name = params.get("name", "")
	
	if scene_path == "":
		return {"error": "scene_path required"}
	
	# Load the scene
	var packed_scene = load(scene_path)
	if not packed_scene:
		return {"error": "Failed to load scene: " + scene_path}
	
	# Instantiate
	var instance = packed_scene.instantiate()
	if not instance:
		return {"error": "Failed to instantiate scene"}
	
	# Set custom name if provided
	if instance_name != "":
		instance.name = instance_name
	
	# Get parent
	var parent = root.get_node_or_null(parent_path) if parent_path != "." else root
	if not parent:
		instance.queue_free()
		return {"error": "Parent node not found: " + parent_path}
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Instantiate Scene via LLM: " + scene_path)
	ur.add_do_method(parent, "add_child", instance)
	ur.add_do_property(instance, "owner", root)
	ur.add_do_reference(instance)
	ur.add_undo_method(parent, "remove_child", instance)
	ur.commit_action()
	
	return {
		"success": true,
		"scene": scene_path,
		"instance_name": instance.name,
		"instance_path": str(instance.get_path())
	}

func _handle_get_tree(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var tree = _build_tree(root, 0)
	return {"success": true, "tree": tree, "root_name": root.name}

func _build_tree(node: Node, indent: int) -> Array:
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
