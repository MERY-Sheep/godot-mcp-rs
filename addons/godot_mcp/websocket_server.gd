@tool
extends Node

## WebSocket Server for Godot MCP Plugin
## Provides bidirectional, low-latency communication with Rust MCP server.
## Uses WebSocketPeer for Godot 4.x

const WS_PORT = 6061

var tcp_server: TCPServer
var _pending_peers: Array = []  # TCPServer peers waiting to handshake
var _connected_peers: Dictionary = {}  # peer_id -> WebSocketPeer

var command_handler  # Reference to command_handler.gd
var plugin: EditorPlugin

signal client_connected(peer_id: int)
signal client_disconnected(peer_id: int)
signal message_received(peer_id: int, message: String)

func _ready() -> void:
	tcp_server = TCPServer.new()
	var err = tcp_server.listen(WS_PORT)
	if err != OK:
		push_error("Godot MCP WebSocket: Failed to start server on port %d: %s" % [WS_PORT, error_string(err)])
		return
	
	print("Godot MCP WebSocket: Server started on port %d" % WS_PORT)

func _exit_tree() -> void:
	if tcp_server:
		tcp_server.stop()
	_pending_peers.clear()
	_connected_peers.clear()
	print("Godot MCP WebSocket: Server stopped")

func _process(_delta: float) -> void:
	# Accept new TCP connections
	while tcp_server and tcp_server.is_connection_available():
		var tcp_peer = tcp_server.take_connection()
		if tcp_peer:
			_pending_peers.append(tcp_peer)
	
	# Handle pending WebSocket handshakes
	var peers_to_remove = []
	for i in range(_pending_peers.size()):
		var tcp_peer = _pending_peers[i]
		if tcp_peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
			peers_to_remove.append(i)
			continue
		
		# Wait for HTTP upgrade request
		if tcp_peer.get_available_bytes() > 0:
			var request = tcp_peer.get_utf8_string(tcp_peer.get_available_bytes())
			if _handle_websocket_handshake(tcp_peer, request):
				# Handshake successful, create WebSocketPeer
				var ws_peer = WebSocketPeer.new()
				ws_peer.accept_stream(tcp_peer)
				var peer_id = tcp_peer.get_instance_id()
				_connected_peers[peer_id] = {"ws": ws_peer, "tcp": tcp_peer}
				peers_to_remove.append(i)
				print("Godot MCP WebSocket: Client connected (peer %d)" % peer_id)
				client_connected.emit(peer_id)
	
	# Remove processed pending peers (in reverse to preserve indices)
	peers_to_remove.reverse()
	for i in peers_to_remove:
		_pending_peers.remove_at(i)
	
	# Poll connected WebSocket peers
	var peers_to_disconnect = []
	for peer_id in _connected_peers:
		var peer_data = _connected_peers[peer_id]
		var ws_peer: WebSocketPeer = peer_data["ws"]
		
		ws_peer.poll()
		
		var state = ws_peer.get_ready_state()
		if state == WebSocketPeer.STATE_OPEN:
			while ws_peer.get_available_packet_count() > 0:
				var packet = ws_peer.get_packet()
				var message = packet.get_string_from_utf8()
				_handle_message(peer_id, ws_peer, message)
		elif state == WebSocketPeer.STATE_CLOSED:
			peers_to_disconnect.append(peer_id)
	
	# Clean up disconnected peers
	for peer_id in peers_to_disconnect:
		_connected_peers.erase(peer_id)
		print("Godot MCP WebSocket: Client disconnected (peer %d)" % peer_id)
		client_disconnected.emit(peer_id)

func _handle_websocket_handshake(tcp_peer: StreamPeerTCP, request: String) -> bool:
	# Check if this is a WebSocket upgrade request
	if not "Upgrade: websocket" in request:
		return false
	
	# Extract Sec-WebSocket-Key
	var key_start = request.find("Sec-WebSocket-Key: ")
	if key_start == -1:
		return false
	
	key_start += 19  # Length of "Sec-WebSocket-Key: "
	var key_end = request.find("\r\n", key_start)
	if key_end == -1:
		return false
	
	var client_key = request.substr(key_start, key_end - key_start)
	
	# Generate accept key
	var magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
	var accept_key = Marshalls.raw_to_base64(
		(client_key + magic).sha1_buffer()
	)
	
	# Send upgrade response
	var response = "HTTP/1.1 101 Switching Protocols\r\n"
	response += "Upgrade: websocket\r\n"
	response += "Connection: Upgrade\r\n"
	response += "Sec-WebSocket-Accept: %s\r\n" % accept_key
	response += "\r\n"
	
	tcp_peer.put_data(response.to_utf8_buffer())
	return true

func _handle_message(peer_id: int, ws_peer: WebSocketPeer, message: String) -> void:
	message_received.emit(peer_id, message)
	
	if plugin:
		plugin.add_log("WS Received: " + message)
	
	# Parse and handle command
	var json = JSON.new()
	var parse_result = json.parse(message)
	
	var response_body: String
	if parse_result == OK:
		var data = json.data
		if command_handler:
			var result = command_handler.handle_command(data)
			response_body = JSON.stringify(result)
		else:
			response_body = JSON.stringify({"error": "Command handler not available"})
	else:
		response_body = JSON.stringify({"error": "Invalid JSON"})
	
	# Send response
	ws_peer.send_text(response_body)

## Send a message to a specific peer
func send_to_peer(peer_id: int, message: String) -> void:
	if _connected_peers.has(peer_id):
		var ws_peer: WebSocketPeer = _connected_peers[peer_id]["ws"]
		if ws_peer.get_ready_state() == WebSocketPeer.STATE_OPEN:
			ws_peer.send_text(message)

## Broadcast a message to all connected peers
func broadcast(message: String) -> void:
	for peer_id in _connected_peers:
		send_to_peer(peer_id, message)
