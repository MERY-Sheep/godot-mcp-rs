@tool
extends EditorDebuggerPlugin

## Debugger Plugin for MCP
## Captures debug errors and manages debug sessions.

var sessions: Array[EditorDebuggerSession] = []
var captured_errors: Array = []
var log_buffer: Array = []
var max_errors = 50

func _setup_session(session_id: int) -> void:
	var session = get_session(session_id)
	if not session:
		return
		
	sessions.append(session)
	print("MCP Debugger: Session started: ", session_id)
	
	# Connect to signals if available (limited in GDScript API)
	# Alternatively, we may need to rely on polling or specific hooks if available.
	# For now, we will try to hook into the session's broken signal if possible,
	# but EditorDebuggerSession API is limited.
	
	# As a workaround for capturing errors, we might need to rely on standard output redirection 
	# or check if specific properties/methods are exposed in future versions.
	# For Godot 4, we primarily use this to send commands TO the debugger (break, resume).

func _has_capture(capture) -> bool:
	return capture == "mcp_debug"

func _capture(message, data, session_id) -> bool:
	if message == "mcp_debug:error":
		_add_error(data)
		return true
	return false

func _add_error(data):
	captured_errors.append(data)
	if captured_errors.size() > max_errors:
		captured_errors.pop_front()

func get_active_session() -> EditorDebuggerSession:
	if sessions.is_empty():
		return null
	return sessions[-1] # Return most recent session

func get_errors() -> Array:
	return captured_errors

func clear_errors():
	captured_errors.clear()
