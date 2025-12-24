@tool
extends RefCounted
## Shader Handler
## Handles shader operations: create_visual_shader_node, validate_shader_live

var plugin: EditorPlugin

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"create_visual_shader_node":
			return _handle_create_visual_shader_node(params)
		"validate_shader_live":
			return _handle_validate_shader_live(params)
		_:
			return {"error": "Unknown shader command: " + command}

func _handle_create_visual_shader_node(params: Dictionary) -> Dictionary:
	var shader_path = params.get("shader_path", "")
	var node_type = params.get("node_type", "")
	var position_x = params.get("position_x", 0.0)
	var position_y = params.get("position_y", 0.0)

	if shader_path == "":
		return {"error": "shader_path is required"}
	if node_type == "":
		return {"error": "node_type is required"}

	# Load the visual shader
	var shader = load(shader_path)
	if not shader or not shader is VisualShader:
		return {"error": "Invalid or not a VisualShader: " + shader_path}

	# Create the visual shader node
	var vs_node = ClassDB.instantiate(node_type)
	if not vs_node or not vs_node is VisualShaderNode:
		return {"error": "Failed to create VisualShaderNode of type: " + node_type}

	# Find a unique ID for the new node
	var new_id = 1
	for existing_id in shader.get_node_list(VisualShader.TYPE_FRAGMENT):
		if existing_id >= new_id:
			new_id = existing_id + 1

	# Add the node to the shader (fragment type by default)
	shader.add_node(VisualShader.TYPE_FRAGMENT, vs_node, Vector2(position_x, position_y), new_id)

	# Save the shader
	var save_result = ResourceSaver.save(shader, shader_path)
	if save_result != OK:
		return {"error": "Failed to save shader after adding node: " + str(save_result)}

	return {
		"success": true,
		"node_id": new_id,
		"node_type": node_type,
		"position": {"x": position_x, "y": position_y}
	}

func _handle_validate_shader_live(params: Dictionary) -> Dictionary:
	var shader_code = params.get("shader_code", "")

	if shader_code == "":
		return {"error": "shader_code is required"}

	# Create a temporary shader resource for validation
	var shader = Shader.new()
	shader.code = shader_code

	# We can check basic syntax with the Godot API
	# For now, we do a basic structural check
	var errors = []
	var warnings = []

	# Check for shader_type declaration
	if not shader_code.contains("shader_type"):
		errors.append({
			"line": 1,
			"message": "Missing shader_type declaration"
		})

	# Simple brace balance check
	var brace_count = 0
	for c in shader_code:
		if c == "{":
			brace_count += 1
		elif c == "}":
			brace_count -= 1

	if brace_count != 0:
		errors.append({
			"line": null,
			"message": "Unbalanced braces: %d unclosed" % abs(brace_count)
		})

	return {
		"success": true,
		"is_valid": errors.size() == 0,
		"errors": errors,
		"warnings": warnings
	}
