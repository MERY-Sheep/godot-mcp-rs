@tool
extends RefCounted
## Animation Handler
## Handles animation operations: create, add_track, add_key, play, stop, list

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
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
		_:
			return {"error": "Unknown animation command: " + command}

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
	
	var library = player.get_animation_library("")
	if not library:
		library = AnimationLibrary.new()
		player.add_animation_library("", library)
	
	library.add_animation(anim_name, anim)
	
	return {"success": true, "animation": anim_name, "length": length}

func _handle_add_animation_track(params: Dictionary) -> Dictionary:
	var root = EditorInterface.get_edited_scene_root()
	if not root:
		return {"error": "No scene is open"}
	
	var player_path = params.get("player", "AnimationPlayer")
	var anim_name = params.get("animation", "")
	var track_path = params.get("track_path", "")
	var track_type = params.get("track_type", "value")
	
	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found"}
	
	var anim = player.get_animation(anim_name)
	if not anim:
		return {"error": "Animation not found: " + anim_name}
	
	var type_map = {
		"value": Animation.TYPE_VALUE,
		"position_2d": Animation.TYPE_POSITION_2D,
		"rotation_2d": Animation.TYPE_ROTATION_2D,
		"scale_2d": Animation.TYPE_SCALE_2D,
		"position_3d": Animation.TYPE_POSITION_3D,
		"rotation_3d": Animation.TYPE_ROTATION_3D,
		"scale_3d": Animation.TYPE_SCALE_3D,
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
	var value = params.get("value", null)
	
	var player = root.get_node_or_null(player_path)
	if not player or not player is AnimationPlayer:
		return {"error": "AnimationPlayer not found"}
	
	var anim = player.get_animation(anim_name)
	if not anim:
		return {"error": "Animation not found: " + anim_name}
	
	if track_idx >= anim.get_track_count():
		return {"error": "Track index out of range"}
	
	var key_idx = anim.track_insert_key(track_idx, time, value)
	
	return {"success": true, "key_index": key_idx, "time": time}

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
