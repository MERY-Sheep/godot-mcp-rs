@tool
extends EditorPlugin

## Godot MCP Plugin
## LLM からの HTTP リクエストを受け取り、エディター内でノード操作を実行

const PORT = 6060

var tcp_server: TCPServer
var command_handler: Node

func _enter_tree():
	# コマンドハンドラを読み込み
	var handler_script = load("res://addons/godot_mcp/command_handler.gd")
	command_handler = handler_script.new()
	command_handler.plugin = self
	add_child(command_handler)
	
	# TCP サーバー起動
	tcp_server = TCPServer.new()
	var err = tcp_server.listen(PORT)
	if err != OK:
		push_error("Godot MCP: Failed to start TCP server on port %d" % PORT)
		return
	
	print("Godot MCP: Server started on port %d" % PORT)

func _exit_tree():
	if tcp_server:
		tcp_server.stop()
	if command_handler:
		command_handler.queue_free()
	print("Godot MCP: Server stopped")

func _process(_delta):
	if tcp_server and tcp_server.is_connection_available():
		var peer = tcp_server.take_connection()
		if peer:
			_handle_connection(peer)

func _handle_connection(peer: StreamPeerTCP):
	# リクエストを読み取り
	peer.set_no_delay(true)
	
	# 接続待機（最大1秒）
	var start_time = Time.get_ticks_msec()
	while peer.get_available_bytes() == 0:
		if Time.get_ticks_msec() - start_time > 1000:
			peer.disconnect_from_host()
			return
		await get_tree().process_frame
	
	var request_data = peer.get_utf8_string(peer.get_available_bytes())
	
	# HTTP リクエストをパース（ヘッダーとボディを分離）
	var double_newline_pos = request_data.find("\r\n\r\n")
	var body = ""
	if double_newline_pos != -1:
		body = request_data.substr(double_newline_pos + 4)
	
	print("Godot MCP: Received body: ", body)
	
	# JSON をパース
	var json = JSON.new()
	var parse_result = json.parse(body)
	
	var response_body: String
	if parse_result == OK:
		var data = json.data
		var result = command_handler.handle_command(data)
		response_body = JSON.stringify(result)
	else:
		response_body = JSON.stringify({"error": "Invalid JSON"})
	
	# HTTP レスポンスを送信
	var response = "HTTP/1.1 200 OK\r\n"
	response += "Content-Type: application/json\r\n"
	response += "Content-Length: %d\r\n" % response_body.length()
	response += "Access-Control-Allow-Origin: *\r\n"
	response += "\r\n"
	response += response_body
	
	peer.put_data(response.to_utf8_buffer())
	peer.disconnect_from_host()
