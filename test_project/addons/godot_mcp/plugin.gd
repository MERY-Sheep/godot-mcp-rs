@tool
extends EditorPlugin

## Godot MCP Plugin
## Receives HTTP/WebSocket requests from LLM and performs node operations within the editor.
## - HTTP (port 6060): Legacy support, simple request/response
## - WebSocket (port 6061): Low-latency bidirectional communication

const PORT = 6060
const MAX_LOG_LINES = 1000

var tcp_server: TCPServer
var command_handler: Node
var websocket_server: Node
var debugger_plugin: EditorDebuggerPlugin
var log_buffer: Array = []

func _enter_tree():
	# Load debugger plugin
	var debugger_script = load("res://addons/godot_mcp/debugger_plugin.gd")
	debugger_plugin = debugger_script.new()
	add_debugger_plugin(debugger_plugin)

	# Load command handler
	var handler_script = load("res://addons/godot_mcp/command_handler.gd")
	command_handler = handler_script.new()
	command_handler.plugin = self
	add_child(command_handler)
	
	# Start WebSocket server (port 6061)
	var ws_script = load("res://addons/godot_mcp/websocket_server.gd")
	websocket_server = ws_script.new()
	websocket_server.command_handler = command_handler
	websocket_server.plugin = self
	add_child(websocket_server)
	
	# Start TCP server (HTTP fallback, port 6060)
	tcp_server = TCPServer.new()
	var err = tcp_server.listen(PORT)
	if err != OK:
		push_error("Godot MCP: Failed to start TCP server on port %d" % PORT)
		return
	
	print("Godot MCP: HTTP Server on port %d, WebSocket Server on port 6061" % PORT)

func _exit_tree():
	if tcp_server:
		tcp_server.stop()
	if command_handler:
		command_handler.queue_free()
	if websocket_server:
		websocket_server.queue_free()
	if debugger_plugin:
		remove_debugger_plugin(debugger_plugin)
	print("Godot MCP: Servers stopped")

func _process(_delta):
	if tcp_server and tcp_server.is_connection_available():
		var peer = tcp_server.take_connection()
		if peer:
			_handle_connection(peer)

func _handle_connection(peer: StreamPeerTCP):
	# Read request
	peer.set_no_delay(true)
	
	# Wait for connection (max 1 second)
	var start_time = Time.get_ticks_msec()
	while peer.get_available_bytes() == 0:
		if Time.get_ticks_msec() - start_time > 1000:
			peer.disconnect_from_host()
			return
		await get_tree().process_frame
	
	var request_data = peer.get_utf8_string(peer.get_available_bytes())
	
	# Parse HTTP request (separate header and body)
	var double_newline_pos = request_data.find("\r\n\r\n")
	var body = ""
	if double_newline_pos != -1:
		body = request_data.substr(double_newline_pos + 4)
	
	print("Godot MCP: Received body: ", body)
	add_log("Received: " + body)
	
	# Parse JSON
	var json = JSON.new()
	var parse_result = json.parse(body)
	
	var response_body: String
	if parse_result == OK:
		var data = json.data
		var result = command_handler.handle_command(data)
		response_body = JSON.stringify(result)
	else:
		response_body = JSON.stringify({"error": "Invalid JSON"})
	
	# Send HTTP response
	var response = "HTTP/1.1 200 OK\r\n"
	response += "Content-Type: application/json\r\n"
	response += "Content-Length: %d\r\n" % response_body.length()
	response += "Access-Control-Allow-Origin: *\r\n"
	response += "\r\n"
	response += response_body
	
	peer.put_data(response.to_utf8_buffer())
	peer.disconnect_from_host()

# === Log Buffer Methods ===

func add_log(message: String) -> void:
	var timestamp = Time.get_datetime_string_from_system()
	log_buffer.append("[%s] %s" % [timestamp, message])
	if log_buffer.size() > MAX_LOG_LINES:
		log_buffer.pop_front()

func get_log_buffer() -> Array:
	return log_buffer

func clear_log_buffer() -> void:
	log_buffer.clear()
