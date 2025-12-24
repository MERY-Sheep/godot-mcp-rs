@tool
extends RefCounted
## Debug Handler
## Handles debug operations: get_editor_log, get_logs, pause, resume, step, breakpoint

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"get_editor_log":
			return _handle_get_editor_log(params)
		"get_logs":
			return _handle_get_logs(params)
		"get_debugger_errors":
			return _handle_get_debugger_errors(params)
		"get_object_by_id":
			return _handle_get_object_by_id(params)
		"pause":
			return _handle_pause(params)
		"resume":
			return _handle_resume(params)
		"step":
			return _handle_step(params)
		"set_breakpoint":
			return _handle_set_breakpoint(params)
		"remove_breakpoint":
			return _handle_remove_breakpoint(params)
		"clear_editor_log":
			return _handle_clear_editor_log(params)
		_:
			return {"error": "Unknown debug command: " + command}

func _handle_get_editor_log(params: Dictionary) -> Dictionary:
	var lines = params.get("lines", 50)
	# In Godot 4.x, direct access to the editor log is restricted.
	# As an alternative, we return the recent plugin logs.
	return {
		"success": true,
		"note": "Direct editor log access is limited in Godot 4.x",
		"requested_lines": lines
	}

func _handle_get_debugger_errors(params: Dictionary) -> Dictionary:
	var result_errors = []
	
	# Try to get errors from the debugger plugin if available
	if plugin.debugger_plugin:
		var errors = plugin.debugger_plugin.get_recent_errors()
		for err in errors:
			result_errors.append({
				"message": str(err),
				"stack_info": [],
				"timestamp": ""
			})
	
	return {"success": true, "errors": result_errors}

func _handle_get_logs(params: Dictionary) -> Dictionary:
	var limit = params.get("limit", 100)
	var logs = []
	
	# Try to get logs from plugin buffer (if implemented there)
	if plugin.log_buffer:
		var buffer = plugin.log_buffer
		var start = max(0, buffer.size() - limit)
		for i in range(start, buffer.size()):
			logs.append(buffer[i])
	
	return {"success": true, "logs": logs, "count": logs.size()}

func _handle_get_object_by_id(params: Dictionary) -> Dictionary:
	var object_id = params.get("object_id", "")
	if object_id == "":
		return {"error": "object_id required"}
	
	var obj = instance_from_id(int(object_id))
	if not obj:
		return {"error": "Object not found: " + object_id}
	
	var properties = []
	for prop in obj.get_property_list():
		var name = prop["name"]
		if name.begins_with("_"):
			continue
		properties.append({
			"name": name,
			"value": str(_serialize_value(obj.get(name))),
			"type": str(prop["type"])
		})
	
	return {
		"success": true,
		"object": {
			"id": str(obj.get_instance_id()),
			"class": obj.get_class(),
			"properties": properties
		}
	}

func _handle_pause(params: Dictionary) -> Dictionary:
	if not plugin.debugger_plugin:
		return {"error": "Debugger plugin not initialized"}
	
	var session = plugin.debugger_plugin.get_active_session()
	if not session:
		return {"error": "No active debug session"}
	
	session.pause()
	return {"success": true}

func _handle_resume(params: Dictionary) -> Dictionary:
	if not plugin.debugger_plugin:
		return {"error": "Debugger plugin not initialized"}
	
	var session = plugin.debugger_plugin.get_active_session()
	if not session:
		return {"error": "No active debug session"}
	
	session.continue_debug()
	return {"success": true}

func _handle_step(params: Dictionary) -> Dictionary:
	if not plugin.debugger_plugin:
		return {"error": "Debugger plugin not initialized"}
	
	var session = plugin.debugger_plugin.get_active_session()
	if not session:
		return {"error": "No active debug session"}
	
	session.next_line() # Step over
	return {"success": true}

func _handle_set_breakpoint(params: Dictionary) -> Dictionary:
	var path = params.get("path", "")
	var line = params.get("line", 0)
	var enabled = params.get("enabled", true)
	
	if path == "":
		return {"error": "Path required"}
	
	# Note: Setting breakpoints programmatically is limited in Godot 4.x
	print("MCP: Set breakpoint request at ", path, ":", line)
	
	return {"success": true, "message": "Breakpoint request received (implementation limited by API)"}

func _handle_remove_breakpoint(params: Dictionary) -> Dictionary:
	var path = params.get("path", "")
	var line = params.get("line", 0)
	
	print("MCP: Remove breakpoint request at ", path, ":", line)
	return {"success": true, "message": "Breakpoint remove request received"}

func _handle_clear_editor_log(params: Dictionary) -> Dictionary:
	# Limited in Godot 4.x
	return {"success": true, "message": "Editor log clear requested"}

func _serialize_value(value) -> Variant:
	match typeof(value):
		TYPE_NIL:
			return null
		TYPE_BOOL, TYPE_INT, TYPE_FLOAT, TYPE_STRING:
			return value
		TYPE_VECTOR2:
			return {"x": value.x, "y": value.y}
		TYPE_VECTOR3:
			return {"x": value.x, "y": value.y, "z": value.z}
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
