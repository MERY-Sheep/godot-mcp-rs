@tool
extends RefCounted
## Introspect Handler
## Handles type introspection operations using ClassDB

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"get_type_info":
			return _get_type_info(params)
		"list_all_types":
			return _list_all_types(params)
		_:
			return {"error": "Unknown introspect command: " + command}


func _get_type_info(params: Dictionary) -> Dictionary:
	var type_name = params.get("type_name", "")
	if type_name == "":
		return {"error": "type_name is required"}
	
	if not ClassDB.class_exists(type_name):
		return {"error": "Type not found: " + type_name}
	
	var properties = []
	for prop in ClassDB.class_get_property_list(type_name, true):
		# Skip internal properties (starting with _)
		if prop.name.begins_with("_"):
			continue
		
		properties.append({
			"name": prop.name,
			"type": _type_to_string(prop.type),
			"hint": _hint_to_string(prop.hint)
		})
	
	var signals_list = []
	for sig in ClassDB.class_get_signal_list(type_name, true):
		var args = []
		for arg in sig.args:
			args.append(arg.name)
		signals_list.append({
			"name": sig.name,
			"arguments": args
		})
	
	return {
		"type_name": type_name,
		"properties": properties,
		"signals": signals_list,
		"parent_class": ClassDB.get_parent_class(type_name)
	}


func _list_all_types(params: Dictionary) -> Dictionary:
	var parent_class = params.get("parent_class", "Node")
	var include_abstract = params.get("include_abstract", false)
	
	var types = []
	for cls in ClassDB.get_inheriters_from_class(parent_class):
		if not include_abstract and ClassDB.is_class_enabled(cls):
			types.append(cls)
		elif include_abstract:
			types.append(cls)
	
	return {
		"parent_class": parent_class,
		"types": types,
		"count": types.size()
	}


func _type_to_string(type: int) -> String:
	match type:
		TYPE_NIL: return "Nil"
		TYPE_BOOL: return "bool"
		TYPE_INT: return "int"
		TYPE_FLOAT: return "float"
		TYPE_STRING: return "String"
		TYPE_VECTOR2: return "Vector2"
		TYPE_VECTOR2I: return "Vector2i"
		TYPE_RECT2: return "Rect2"
		TYPE_RECT2I: return "Rect2i"
		TYPE_VECTOR3: return "Vector3"
		TYPE_VECTOR3I: return "Vector3i"
		TYPE_TRANSFORM2D: return "Transform2D"
		TYPE_VECTOR4: return "Vector4"
		TYPE_VECTOR4I: return "Vector4i"
		TYPE_PLANE: return "Plane"
		TYPE_QUATERNION: return "Quaternion"
		TYPE_AABB: return "AABB"
		TYPE_BASIS: return "Basis"
		TYPE_TRANSFORM3D: return "Transform3D"
		TYPE_PROJECTION: return "Projection"
		TYPE_COLOR: return "Color"
		TYPE_STRING_NAME: return "StringName"
		TYPE_NODE_PATH: return "NodePath"
		TYPE_RID: return "RID"
		TYPE_OBJECT: return "Object"
		TYPE_CALLABLE: return "Callable"
		TYPE_SIGNAL: return "Signal"
		TYPE_DICTIONARY: return "Dictionary"
		TYPE_ARRAY: return "Array"
		TYPE_PACKED_BYTE_ARRAY: return "PackedByteArray"
		TYPE_PACKED_INT32_ARRAY: return "PackedInt32Array"
		TYPE_PACKED_INT64_ARRAY: return "PackedInt64Array"
		TYPE_PACKED_FLOAT32_ARRAY: return "PackedFloat32Array"
		TYPE_PACKED_FLOAT64_ARRAY: return "PackedFloat64Array"
		TYPE_PACKED_STRING_ARRAY: return "PackedStringArray"
		TYPE_PACKED_VECTOR2_ARRAY: return "PackedVector2Array"
		TYPE_PACKED_VECTOR3_ARRAY: return "PackedVector3Array"
		TYPE_PACKED_COLOR_ARRAY: return "PackedColorArray"
		TYPE_PACKED_VECTOR4_ARRAY: return "PackedVector4Array"
		_: return "Unknown"


func _hint_to_string(hint: int) -> String:
	match hint:
		PROPERTY_HINT_NONE: return ""
		PROPERTY_HINT_RANGE: return "range"
		PROPERTY_HINT_ENUM: return "enum"
		PROPERTY_HINT_ENUM_SUGGESTION: return "enum_suggestion"
		PROPERTY_HINT_EXP_EASING: return "exp_easing"
		PROPERTY_HINT_LINK: return "link"
		PROPERTY_HINT_FLAGS: return "flags"
		PROPERTY_HINT_LAYERS_2D_RENDER: return "layers_2d_render"
		PROPERTY_HINT_LAYERS_2D_PHYSICS: return "layers_2d_physics"
		PROPERTY_HINT_LAYERS_2D_NAVIGATION: return "layers_2d_navigation"
		PROPERTY_HINT_LAYERS_3D_RENDER: return "layers_3d_render"
		PROPERTY_HINT_LAYERS_3D_PHYSICS: return "layers_3d_physics"
		PROPERTY_HINT_LAYERS_3D_NAVIGATION: return "layers_3d_navigation"
		PROPERTY_HINT_FILE: return "file"
		PROPERTY_HINT_DIR: return "dir"
		PROPERTY_HINT_GLOBAL_FILE: return "global_file"
		PROPERTY_HINT_GLOBAL_DIR: return "global_dir"
		PROPERTY_HINT_RESOURCE_TYPE: return "resource_type"
		PROPERTY_HINT_MULTILINE_TEXT: return "multiline_text"
		PROPERTY_HINT_EXPRESSION: return "expression"
		PROPERTY_HINT_PLACEHOLDER_TEXT: return "placeholder_text"
		PROPERTY_HINT_COLOR_NO_ALPHA: return "color_no_alpha"
		PROPERTY_HINT_OBJECT_ID: return "object_id"
		PROPERTY_HINT_TYPE_STRING: return "type_string"
		PROPERTY_HINT_NODE_PATH_TO_EDITED_NODE: return "node_path_to_edited_node"
		PROPERTY_HINT_OBJECT_TOO_BIG: return "object_too_big"
		PROPERTY_HINT_NODE_PATH_VALID_TYPES: return "node_path_valid_types"
		PROPERTY_HINT_SAVE_FILE: return "save_file"
		PROPERTY_HINT_GLOBAL_SAVE_FILE: return "global_save_file"
		PROPERTY_HINT_INT_IS_OBJECTID: return "int_is_objectid"
		PROPERTY_HINT_INT_IS_POINTER: return "int_is_pointer"
		PROPERTY_HINT_ARRAY_TYPE: return "array_type"
		PROPERTY_HINT_LOCALE_ID: return "locale_id"
		PROPERTY_HINT_LOCALIZABLE_STRING: return "localizable_string"
		PROPERTY_HINT_NODE_TYPE: return "node_type"
		PROPERTY_HINT_HIDE_QUATERNION_EDIT: return "hide_quaternion_edit"
		PROPERTY_HINT_PASSWORD: return "password"
		_: return ""
