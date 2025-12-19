@tool
extends Node

## コマンドハンドラ
## MCP からのコマンドを受け取り、エディター API で実行

var plugin: EditorPlugin

func handle_command(data: Dictionary) -> Dictionary:
	var command = data.get("command", "")
	var params = data.get("params", {})
	
	match command:
		"ping":
			return {"success": true, "message": "pong", "version": "1.1.0"}
		"add_node":
			return _handle_add_node(params)
		"remove_node":
			return _handle_remove_node(params)
		"set_property":
			return _handle_set_property(params)
		"get_tree":
			return _handle_get_tree(params)
		"save_scene":
			return _handle_save_scene(params)
		"connect_signal":
			return _handle_connect_signal(params)
		"disconnect_signal":
			return _handle_disconnect_signal(params)
		"list_signals":
			return _handle_list_signals(params)
		# === Animation Commands ===
		"create_animation":
			return _handle_create_animation(params)
		"add_animation_track":
			return _handle_add_animation_track(params)
		"add_animation_key":
			return _handle_add_animation_key(params)
		"play_animation":
			return _handle_play_animation(params)
		"stop_animation":
			return _handle_stop_animation(params)
		"list_animations":
			return _handle_list_animations(params)
		# === Debug Log Commands ===
		"get_editor_log":
			return _handle_get_editor_log(params)
		"clear_editor_log":
			return _handle_clear_editor_log(params)
		# === Scene Instance Commands ===
		"instantiate_scene":
			return _handle_instantiate_scene(params)
		_:
			return {"error": "Unknown command: " + command}

func _handle_add_node(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var parent_path = params.get("parent", ".")
	var parent = root.get_node(parent_path) if parent_path != "." else root
	if not parent:
		return {"error": "Parent node not found: " + parent_path}
	
	var node_type = params.get("node_type", "Node")
	var node_name = params.get("name", "NewNode")
	
	# ノードを作成
	var new_node = ClassDB.instantiate(node_type)
	if not new_node:
		return {"error": "Failed to instantiate node type: " + node_type}
	
	new_node.name = node_name
	new_node.set_owner(root)
	
	# Undo/Redo 対応
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
	
	# Undo/Redo 対応
	var ur = plugin.get_undo_redo()
	ur.create_action("Remove Node via LLM: " + node.name)
	ur.add_do_method(parent, "remove_child", node)
	ur.add_undo_method(parent, "add_child", node)
	ur.add_undo_reference(node)
	ur.commit_action()
	
	return {"success": true, "removed": node_path}

func _handle_set_property(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", ".")
	var node = root.get_node_or_null(node_path) if node_path != "." else root
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var property = params.get("property", "")
	var value = params.get("value")
	
	if property == "":
		return {"error": "Property name is required"}
	
	var old_value = node.get(property)
	
	# Undo/Redo 対応
	var ur = plugin.get_undo_redo()
	ur.create_action("Set Property via LLM: " + property)
	ur.add_do_property(node, property, value)
	ur.add_undo_property(node, property, old_value)
	ur.commit_action()
	
	return {"success": true, "property": property, "value": value}

func _handle_get_tree(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var tree = _build_tree(root)
	return {"success": true, "tree": tree}

func _build_tree(node: Node, indent: int = 0) -> Array:
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

func _handle_save_scene(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var scene_path = root.scene_file_path
	if scene_path == "":
		return {"error": "Scene has no file path"}
	
	var packed_scene = PackedScene.new()
	packed_scene.pack(root)
	var err = ResourceSaver.save(packed_scene, scene_path)
	
	if err != OK:
		return {"error": "Failed to save scene: " + str(err)}
	
	return {"success": true, "path": scene_path}

# === Signal Commands ===

func _handle_connect_signal(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var source_path = params.get("source", ".")
	var signal_name = params.get("signal", "")
	var target_path = params.get("target", ".")
	var method_name = params.get("method", "")
	
	if signal_name == "":
		return {"error": "Signal name is required"}
	if method_name == "":
		return {"error": "Method name is required"}
	
	var source = root.get_node_or_null(source_path) if source_path != "." else root
	var target = root.get_node_or_null(target_path) if target_path != "." else root
	
	if not source:
		return {"error": "Source node not found: " + source_path}
	if not target:
		return {"error": "Target node not found: " + target_path}
	
	# シグナルが存在するか確認
	if not source.has_signal(signal_name):
		return {"error": "Signal not found: " + signal_name + " on " + source_path}
	
	# 既に接続されているか確認
	if source.is_connected(signal_name, Callable(target, method_name)):
		return {"error": "Signal already connected"}
	
	# Undo/Redo 対応
	var ur = plugin.get_undo_redo()
	ur.create_action("Connect Signal via LLM: " + signal_name)
	ur.add_do_method(source, "connect", signal_name, Callable(target, method_name))
	ur.add_undo_method(source, "disconnect", signal_name, Callable(target, method_name))
	ur.commit_action()
	
	return {
		"success": true,
		"source": source_path,
		"signal": signal_name,
		"target": target_path,
		"method": method_name
	}

func _handle_disconnect_signal(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var source_path = params.get("source", ".")
	var signal_name = params.get("signal", "")
	var target_path = params.get("target", ".")
	var method_name = params.get("method", "")
	
	if signal_name == "":
		return {"error": "Signal name is required"}
	if method_name == "":
		return {"error": "Method name is required"}
	
	var source = root.get_node_or_null(source_path) if source_path != "." else root
	var target = root.get_node_or_null(target_path) if target_path != "." else root
	
	if not source:
		return {"error": "Source node not found: " + source_path}
	if not target:
		return {"error": "Target node not found: " + target_path}
	
	# 接続されているか確認
	if not source.is_connected(signal_name, Callable(target, method_name)):
		return {"error": "Signal is not connected"}
	
	# Undo/Redo 対応
	var ur = plugin.get_undo_redo()
	ur.create_action("Disconnect Signal via LLM: " + signal_name)
	ur.add_do_method(source, "disconnect", signal_name, Callable(target, method_name))
	ur.add_undo_method(source, "connect", signal_name, Callable(target, method_name))
	ur.commit_action()
	
	return {
		"success": true,
		"source": source_path,
		"signal": signal_name,
		"target": target_path,
		"method": method_name,
		"action": "disconnected"
	}

func _handle_list_signals(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var node_path = params.get("node_path", ".")
	var node = root.get_node_or_null(node_path) if node_path != "." else root
	
	if not node:
		return {"error": "Node not found: " + node_path}
	
	var signals = []
	for sig in node.get_signal_list():
		var sig_info = {
			"name": sig["name"],
			"args": []
		}
		for arg in sig["args"]:
			sig_info["args"].append(arg["name"])
		signals.append(sig_info)
	
	# 接続情報も取得
	var connections = []
	for conn in node.get_incoming_connections():
		connections.append({
			"signal": conn["signal"].get_name(),
			"from": str(conn["signal"].get_object().get_path()) if conn["signal"].get_object() else "unknown",
			"method": conn["callable"].get_method()
		})
	
	for sig_name in node.get_signal_list().map(func(s): return s["name"]):
		for conn in node.get_signal_connection_list(sig_name):
			connections.append({
				"signal": sig_name,
				"from": str(node.get_path()),
				"to": str(conn["callable"].get_object().get_path()) if conn["callable"].get_object() else "unknown",
				"method": conn["callable"].get_method()
			})
	
	return {
		"success": true,
		"node": node_path,
		"type": node.get_class(),
		"signals": signals,
		"connections": connections
	}

# === Animation Commands ===

func _handle_create_animation(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var player_path = params.get("player", "AnimationPlayer")
	var anim_name = params.get("name", "new_animation")
	var length = params.get("length", 1.0)

	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found: " + player_path}

	var anim = Animation.new()
	anim.length = length

	var ur = plugin.get_undo_redo()
	ur.create_action("Create Animation via LLM: " + anim_name)
	ur.add_do_method(player, "add_animation", anim_name, anim)
	ur.add_undo_method(player, "remove_animation", anim_name)
	ur.commit_action()

	return {"success": true, "animation": anim_name, "length": length}

func _handle_add_animation_track(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var player_path = params.get("player", "AnimationPlayer")
	var anim_name = params.get("animation", "")
	var track_path = params.get("track_path", "")  # e.g., "Sprite2D:position"
	var track_type = params.get("track_type", "value")  # value, position, rotation, scale

	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found: " + player_path}

	var anim = player.get_animation(anim_name)
	if not anim:
		return {"error": "Animation not found: " + anim_name}

	var type_map = {
		"value": Animation.TYPE_VALUE,
		"position": Animation.TYPE_POSITION_3D,
		"rotation": Animation.TYPE_ROTATION_3D,
		"scale": Animation.TYPE_SCALE_3D,
	}

	var idx = anim.add_track(type_map.get(track_type, Animation.TYPE_VALUE))
	anim.track_set_path(idx, track_path)

	return {"success": true, "track_index": idx, "path": track_path}

func _handle_add_animation_key(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var player_path = params.get("player", "AnimationPlayer")
	var anim_name = params.get("animation", "")
	var track_idx = params.get("track", 0)
	var time = params.get("time", 0.0)
	var value = params.get("value")

	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found"}

	var anim = player.get_animation(anim_name)
	if not anim:
		return {"error": "Animation not found: " + anim_name}

	anim.track_insert_key(track_idx, time, value)

	return {"success": true, "track": track_idx, "time": time}

func _handle_play_animation(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var player_path = params.get("player", "AnimationPlayer")
	var anim_name = params.get("animation", "")

	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found"}

	player.play(anim_name)
	return {"success": true, "playing": anim_name}

func _handle_stop_animation(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var player_path = params.get("player", "AnimationPlayer")

	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found"}

	player.stop()
	return {"success": true, "stopped": true}

func _handle_list_animations(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var player_path = params.get("player", "AnimationPlayer")

	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found"}

	var anims = []
	for name in player.get_animation_list():
		var anim = player.get_animation(name)
		anims.append({
			"name": name,
			"length": anim.length,
			"tracks": anim.get_track_count()
		})

	return {"success": true, "animations": anims}

# === Debug Log Commands ===

func _handle_get_editor_log(params: Dictionary) -> Dictionary:
	var lines = params.get("lines", 50)

	# プラグインのログバッファから取得
	var log_lines = []
	if plugin.has_method("get_log_buffer"):
		var buffer = plugin.get_log_buffer()
		var start_idx = max(0, buffer.size() - lines)
		for i in range(start_idx, buffer.size()):
			log_lines.append(buffer[i])
	else:
		log_lines = ["Log capture not initialized. Use run_project with output file."]

	return {
		"success": true,
		"lines": log_lines.size(),
		"log": log_lines,
		"note": "For full logs, use get-debug-output with .godot_mcp_output file"
	}

func _handle_clear_editor_log(params: Dictionary) -> Dictionary:
	if plugin.has_method("clear_log_buffer"):
		plugin.clear_log_buffer()
	return {"success": true, "cleared": true}

# === Scene Instance Commands ===

func _handle_instantiate_scene(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}

	var scene_path = params.get("scene_path", "")
	var parent_path = params.get("parent", ".")
	var instance_name = params.get("name", "")
	var position = params.get("position", null)

	if scene_path == "":
		return {"error": "scene_path is required"}

	# res:// プレフィックスを追加
	if not scene_path.begins_with("res://"):
		scene_path = "res://" + scene_path

	# シーンを読み込み
	var packed_scene = load(scene_path)
	if not packed_scene:
		return {"error": "Failed to load scene: " + scene_path}

	# インスタンス化
	var instance = packed_scene.instantiate()
	if not instance:
		return {"error": "Failed to instantiate scene"}

	# 名前を設定
	if instance_name != "":
		instance.name = instance_name

	# 位置を設定（3Dノードの場合）
	if position and instance is Node3D:
		instance.position = Vector3(
			position.get("x", 0),
			position.get("y", 0),
			position.get("z", 0)
		)
	elif position and instance is Node2D:
		instance.position = Vector2(
			position.get("x", 0),
			position.get("y", 0)
		)

	# 親ノードを取得
	var parent = root.get_node_or_null(parent_path) if parent_path != "." else root
	if not parent:
		instance.queue_free()
		return {"error": "Parent node not found: " + parent_path}

	# Undo/Redo 対応
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

