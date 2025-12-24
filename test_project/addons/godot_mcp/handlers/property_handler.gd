@tool
extends RefCounted
## Property Handler
## Handles property operations: set_property, get_properties

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"set_property":
			return _handle_set_property(params)
		"get_properties":
			return _handle_get_properties(params)
		_:
			return {"error": "Unknown property command: " + command}

func _handle_set_property(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", ".")
	var property = params.get("property", "")
	var value = params.get("value", null)
	
	if property == "":
		return {"error": "Property name required"}
	
	var node = root.get_node_or_null(node_path) if node_path != "." else root
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var old_value = node.get(property)
	
	# Parse value based on type
	var parsed_value = _parse_value(value, typeof(old_value))
	
	# Undo/Redo support
	var ur = plugin.get_undo_redo()
	ur.create_action("Set Property via LLM: " + property)
	ur.add_do_property(node, property, parsed_value)
	ur.add_undo_property(node, property, old_value)
	ur.commit_action()
	
	return {
		"success": true,
		"node": node_path,
		"property": property,
		"old_value": _serialize_value(old_value),
		"new_value": _serialize_value(parsed_value)
	}

func _handle_get_properties(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", ".")
	var node = root.get_node_or_null(node_path) if node_path != "." else root
	
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var properties = {}
	for prop in node.get_property_list():
		var name = prop["name"]
		if name.begins_with("_") or prop["usage"] & PROPERTY_USAGE_SCRIPT_VARIABLE == 0:
			continue
		
		var value = node.get(name)
		properties[name] = _serialize_value(value)
	
	return {
		"success": true,
		"node": node_path,
		"type": node.get_class(),
		"properties": properties
	}

func _parse_value(value: Variant, target_type: int) -> Variant:
	if value == null:
		return null
	
	# If value is already correct type, return as-is
	if typeof(value) == target_type:
		return value
	
	# Try to parse string values
	if typeof(value) == TYPE_STRING:
		match target_type:
			TYPE_BOOL:
				return value.to_lower() == "true"
			TYPE_INT:
				return int(value)
			TYPE_FLOAT:
				return float(value)
			TYPE_VECTOR2:
				return _parse_vector2(value)
			TYPE_VECTOR3:
				return _parse_vector3(value)
			TYPE_COLOR:
				return Color(value)
	
	return value

func _parse_vector2(s: String) -> Vector2:
	var cleaned = s.replace("(", "").replace(")", "").replace(" ", "")
	var parts = cleaned.split(",")
	if parts.size() >= 2:
		return Vector2(float(parts[0]), float(parts[1]))
	return Vector2.ZERO

func _parse_vector3(s: String) -> Vector3:
	var cleaned = s.replace("(", "").replace(")", "").replace(" ", "")
	var parts = cleaned.split(",")
	if parts.size() >= 3:
		return Vector3(float(parts[0]), float(parts[1]), float(parts[2]))
	return Vector3.ZERO

func _serialize_value(value) -> Variant:
	match typeof(value):
		TYPE_NIL:
			return null
		TYPE_BOOL, TYPE_INT, TYPE_FLOAT, TYPE_STRING:
			return value
		TYPE_VECTOR2:
			return {"x": value.x, "y": value.y}
		TYPE_VECTOR2I:
			return {"x": value.x, "y": value.y}
		TYPE_VECTOR3:
			return {"x": value.x, "y": value.y, "z": value.z}
		TYPE_VECTOR3I:
			return {"x": value.x, "y": value.y, "z": value.z}
		TYPE_COLOR:
			return {"r": value.r, "g": value.g, "b": value.b, "a": value.a}
		TYPE_RECT2:
			return {"position": {"x": value.position.x, "y": value.position.y}, "size": {"x": value.size.x, "y": value.size.y}}
		TYPE_TRANSFORM2D:
			return str(value)
		TYPE_TRANSFORM3D:
			return str(value)
		TYPE_ARRAY:
			var arr = []
			for item in value:
				arr.append(_serialize_value(item))
			return arr
		TYPE_DICTIONARY:
			var dict = {}
			for key in value:
				dict[str(key)] = _serialize_value(value[key])
			return dict
		_:
			return str(value)
