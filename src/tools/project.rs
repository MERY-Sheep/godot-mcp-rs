//! Project-related tools - File operations and search

use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};
use std::path::{Path, PathBuf};

use super::{
    GetNodeTypeInfoRequest, GetProjectStatsRequest, GodotTools, ListFilesRequest, ReadFileRequest,
    SearchInProjectRequest, ValidateProjectRequest,
};
use crate::godot::tscn::GodotScene;

impl GodotTools {
    /// list_project_files - List project files
    pub async fn handle_list_project_files(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ListFilesRequest = if let Some(args) = args {
            serde_json::from_value(serde_json::Value::Object(args))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?
        } else {
            ListFilesRequest { path: None }
        };

        let base = self.get_base_path();
        let search_path = req
            .path
            .as_ref()
            .map(|p| base.join(p))
            .unwrap_or_else(|| base.to_path_buf());

        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&search_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string());
                files.push(serde_json::json!({
                    "path": path.strip_prefix(base).unwrap_or(&path).to_string_lossy(),
                    "is_directory": path.is_dir(),
                    "extension": ext,
                }));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&files).unwrap_or_default(),
        )]))
    }

    /// read_file - Read a file
    pub async fn handle_read_file(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ReadFileRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let content = std::fs::read_to_string(base.join(&req.path))
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    /// list_all_scenes - List all scenes
    pub async fn handle_list_all_scenes(
        &self,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let base = self.get_base_path();
        let mut scene_paths = Vec::new();

        fn find_scene_files(dir: &Path, base: &Path, paths: &mut Vec<PathBuf>) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        find_scene_files(&path, base, paths);
                    } else if path.extension().map(|e| e == "tscn").unwrap_or(false) {
                        paths.push(path);
                    }
                }
            }
        }

        find_scene_files(base, base, &mut scene_paths);

        let mut scenes = Vec::new();
        for path in scene_paths {
            let rel_path = path.strip_prefix(base).unwrap_or(&path).to_string_lossy();
            let res_path = format!("res://{}", rel_path.replace('\\', "/"));

            let root_type = if let Ok(content) = std::fs::read_to_string(&path) {
                GodotScene::parse(&content)
                    .ok()
                    .and_then(|s| s.nodes.first().map(|n| n.node_type.clone()))
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            scenes.push(serde_json::json!({
                "path": res_path,
                "root_type": root_type,
            }));
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&serde_json::json!({ "scenes": scenes }))
                .unwrap_or_default(),
        )]))
    }

    /// search_in_project - Search within the project
    pub async fn handle_search_in_project(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: SearchInProjectRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let mut results = Vec::new();

        fn find_tscn_files(dir: &Path) -> Vec<PathBuf> {
            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(find_tscn_files(&path));
                    } else if path.extension().map(|e| e == "tscn").unwrap_or(false) {
                        files.push(path);
                    }
                }
            }
            files
        }

        let tscn_files = find_tscn_files(base);

        for file_path in tscn_files {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(scene) = GodotScene::parse(&content) {
                    let rel_path = file_path.strip_prefix(base).unwrap_or(&file_path);
                    let res_path =
                        format!("res://{}", rel_path.to_string_lossy().replace('\\', "/"));

                    match req.search_type.as_str() {
                        "node_type" => {
                            for node in &scene.nodes {
                                if node.node_type.contains(&req.query) {
                                    results.push(serde_json::json!({
                                        "file": res_path.clone(),
                                        "node": node.name.clone(),
                                        "type": node.node_type.clone(),
                                    }));
                                }
                            }
                        }
                        "resource" => {
                            for res in &scene.ext_resources {
                                if res.path.contains(&req.query)
                                    || res.resource_type.contains(&req.query)
                                {
                                    results.push(serde_json::json!({
                                        "file": res_path.clone(),
                                        "resource_id": res.id.clone(),
                                        "resource_type": res.resource_type.clone(),
                                        "path": res.path.clone(),
                                    }));
                                }
                            }
                        }
                        "script" => {
                            for node in &scene.nodes {
                                if let Some(script) = node.properties.get("script") {
                                    if script.contains(&req.query) {
                                        results.push(serde_json::json!({
                                            "file": res_path.clone(),
                                            "node": node.name.clone(),
                                            "script": script.clone(),
                                        }));
                                    }
                                }
                            }
                        }
                        _ => {
                            return Err(McpError::invalid_params(
                                format!(
                                    "Unknown search_type: {}. Use: node_type, resource, script",
                                    req.search_type
                                ),
                                None,
                            ));
                        }
                    }
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&serde_json::json!({ "results": results }))
                .unwrap_or_default(),
        )]))
    }

    /// get_node_type_info - Get detailed information about a node type
    pub async fn handle_get_node_type_info(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: GetNodeTypeInfoRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let info = get_node_type_database(&req.node_type);

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&info).unwrap_or_default(),
        )]))
    }

    /// get_project_stats - Project statistics
    pub async fn handle_get_project_stats(
        &self,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let _req: GetProjectStatsRequest = Default::default();
        let base = self.get_base_path();

        fn count_files(dir: &Path, ext: &str) -> (usize, usize) {
            let mut count = 0;
            let mut total_bytes = 0;
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let (c, b) = count_files(&path, ext);
                        count += c;
                        total_bytes += b;
                    } else if path.extension().map(|e| e == ext).unwrap_or(false) {
                        count += 1;
                        total_bytes += std::fs::metadata(&path)
                            .map(|m| m.len() as usize)
                            .unwrap_or(0);
                    }
                }
            }
            (count, total_bytes)
        }

        let (scene_count, scene_bytes) = count_files(base, "tscn");
        let (script_count, script_bytes) = count_files(base, "gd");
        let (resource_count, resource_bytes) = count_files(base, "tres");

        // Calculate total node count
        let mut total_nodes = 0;
        let mut node_type_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        fn find_tscn_files_stat(dir: &Path) -> Vec<PathBuf> {
            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(find_tscn_files_stat(&path));
                    } else if path.extension().map(|e| e == "tscn").unwrap_or(false) {
                        files.push(path);
                    }
                }
            }
            files
        }

        for file_path in find_tscn_files_stat(base) {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(scene) = GodotScene::parse(&content) {
                    total_nodes += scene.nodes.len();
                    for node in &scene.nodes {
                        *node_type_counts.entry(node.node_type.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        let result = serde_json::json!({
            "files": {
                "scenes": { "count": scene_count, "bytes": scene_bytes },
                "scripts": { "count": script_count, "bytes": script_bytes },
                "resources": { "count": resource_count, "bytes": resource_bytes },
            },
            "nodes": {
                "total": total_nodes,
                "by_type": node_type_counts,
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// validate_project - Project validation
    pub async fn handle_validate_project(
        &self,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let _req: ValidateProjectRequest = Default::default();
        let base = self.get_base_path();

        let mut issues: Vec<serde_json::Value> = Vec::new();

        // Collect all .tscn files
        fn find_tscn_files_val(dir: &Path) -> Vec<PathBuf> {
            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(find_tscn_files_val(&path));
                    } else if path.extension().map(|e| e == "tscn").unwrap_or(false) {
                        files.push(path);
                    }
                }
            }
            files
        }

        fn find_gd_files(dir: &Path) -> Vec<PathBuf> {
            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(find_gd_files(&path));
                    } else if path.extension().map(|e| e == "gd").unwrap_or(false) {
                        files.push(path);
                    }
                }
            }
            files
        }

        // Collect script paths in use
        let mut used_scripts: std::collections::HashSet<String> = std::collections::HashSet::new();

        for file_path in find_tscn_files_val(base) {
            let rel_path = file_path.strip_prefix(base).unwrap_or(&file_path);
            let res_path = format!("res://{}", rel_path.to_string_lossy().replace('\\', "/"));

            if let Ok(content) = std::fs::read_to_string(&file_path) {
                match GodotScene::parse(&content) {
                    Ok(scene) => {
                        // Empty scene
                        if scene.nodes.is_empty() {
                            issues.push(serde_json::json!({
                                "type": "empty_scene",
                                "file": res_path,
                                "severity": "warning",
                            }));
                        }

                        // Collect script references
                        for res in &scene.ext_resources {
                            if res.resource_type == "Script" {
                                used_scripts.insert(res.path.clone());
                            }
                        }
                    }
                    Err(e) => {
                        issues.push(serde_json::json!({
                            "type": "parse_error",
                            "file": res_path,
                            "error": e.to_string(),
                            "severity": "error",
                        }));
                    }
                }
            }
        }

        // Detect unused scripts
        for file_path in find_gd_files(base) {
            let rel_path = file_path.strip_prefix(base).unwrap_or(&file_path);
            let res_path = format!("res://{}", rel_path.to_string_lossy().replace('\\', "/"));

            if !used_scripts.contains(&res_path) {
                issues.push(serde_json::json!({
                    "type": "unused_script",
                    "file": res_path,
                    "severity": "info",
                }));
            }
        }

        let error_count = issues.iter().filter(|i| i["severity"] == "error").count();
        let warning_count = issues.iter().filter(|i| i["severity"] == "warning").count();
        let info_count = issues.iter().filter(|i| i["severity"] == "info").count();

        let result = serde_json::json!({
            "valid": error_count == 0,
            "summary": {
                "errors": error_count,
                "warnings": warning_count,
                "info": info_count,
            },
            "issues": issues,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }
}

/// Static database of Godot node types
fn get_node_type_database(node_type: &str) -> serde_json::Value {
    match node_type {
        // 3D Physics Bodies
        "CharacterBody3D" => serde_json::json!({
            "name": "CharacterBody3D",
            "category": "3D Physics",
            "description": "A physics body for 3D character movement. Can move using move_and_slide() while handling collisions.",
            "inherits": ["PhysicsBody3D", "CollisionObject3D", "Node3D", "Node"],
            "key_properties": {
                "velocity": "Vector3 - Current velocity vector",
                "floor_max_angle": "float - Maximum angle to be considered as floor (rad)",
                "motion_mode": "MotionMode - GROUNDED or FLOATING"
            },
            "key_methods": ["move_and_slide()", "is_on_floor()", "is_on_wall()", "get_floor_normal()"],
            "common_children": ["CollisionShape3D", "MeshInstance3D", "Camera3D", "AnimationPlayer"],
            "script_template": "CharacterBody3D",
            "tips": "Always add a CollisionShape3D as a child. Set velocity and call move_and_slide()."
        }),
        "RigidBody3D" => serde_json::json!({
            "name": "RigidBody3D",
            "category": "3D Physics",
            "description": "An object moved by physics simulation. Gravity and collisions are handled automatically.",
            "inherits": ["PhysicsBody3D", "CollisionObject3D", "Node3D", "Node"],
            "key_properties": {
                "mass": "float - Mass (kg)",
                "gravity_scale": "float - Gravity scale coefficient",
                "linear_velocity": "Vector3 - Linear velocity",
                "angular_velocity": "Vector3 - Angular velocity"
            },
            "key_methods": ["apply_force()", "apply_impulse()", "apply_torque()"],
            "common_children": ["CollisionShape3D", "MeshInstance3D"],
            "tips": "Not suitable for player control. Use for projectiles or objects."
        }),
        "StaticBody3D" => serde_json::json!({
            "name": "StaticBody3D",
            "category": "3D Physics",
            "description": "A static collision object that does not move. Ideal for terrain and walls.",
            "inherits": ["PhysicsBody3D", "CollisionObject3D", "Node3D", "Node"],
            "key_properties": {
                "physics_material_override": "PhysicsMaterial - Friction and bounciness"
            },
            "common_children": ["CollisionShape3D", "MeshInstance3D"],
            "tips": "Good performance even with large quantities. Use for level geometry."
        }),
        "Area3D" => serde_json::json!({
            "name": "Area3D",
            "category": "3D Physics",
            "description": "A region for collision detection. Detects entry/exit via signals. Ideal for damage zones or triggers.",
            "inherits": ["CollisionObject3D", "Node3D", "Node"],
            "key_signals": ["body_entered(body)", "body_exited(body)", "area_entered(area)"],
            "common_children": ["CollisionShape3D"],
            "tips": "Detection is enabled with monitoring=true. detectable from other Areas with monitorable=true."
        }),

        // 3D Visual
        "MeshInstance3D" => serde_json::json!({
            "name": "MeshInstance3D",
            "category": "3D Visual",
            "description": "A node that displays a 3D mesh.",
            "inherits": ["GeometryInstance3D", "VisualInstance3D", "Node3D", "Node"],
            "key_properties": {
                "mesh": "Mesh - Mesh resource to display",
                "surface_material_override": "Material - Material override"
            },
            "common_meshes": ["BoxMesh", "SphereMesh", "CapsuleMesh", "CylinderMesh", "PlaneMesh"],
            "tips": "Shapes can be easily created using PrimitiveMesh."
        }),
        "Camera3D" => serde_json::json!({
            "name": "Camera3D",
            "category": "3D Visual",
            "description": "A camera that renders a 3D scene. Activated with current=true.",
            "inherits": ["Node3D", "Node"],
            "key_properties": {
                "current": "bool - Make this camera active",
                "fov": "float - Field of view (degrees)",
                "near": "float - Near clip distance",
                "far": "float - Far clip distance"
            },
            "key_methods": ["make_current()", "project_ray_origin()", "unproject_position()"],
            "tips": "TPS: Place as a child of the player. FPS: Place as a child of the Head node."
        }),
        "DirectionalLight3D" => serde_json::json!({
            "name": "DirectionalLight3D",
            "category": "3D Visual",
            "description": "A directional light source from infinity, like the sun.",
            "inherits": ["Light3D", "VisualInstance3D", "Node3D", "Node"],
            "key_properties": {
                "light_energy": "float - Light energy",
                "shadow_enabled": "bool - Enable shadows"
            },
            "tips": "Place one in the scene. Control sun direction via rotation."
        }),

        // 2D Physics Bodies
        "CharacterBody2D" => serde_json::json!({
            "name": "CharacterBody2D",
            "category": "2D Physics",
            "description": "A physics body for 2D character movement.",
            "inherits": ["PhysicsBody2D", "CollisionObject2D", "Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "velocity": "Vector2 - Current velocity vector",
                "floor_max_angle": "float - Maximum angle to be considered as floor"
            },
            "key_methods": ["move_and_slide()", "is_on_floor()", "is_on_wall()"],
            "common_children": ["CollisionShape2D", "Sprite2D", "AnimatedSprite2D"],
            "script_template": "CharacterBody2D",
            "tips": "Ideal for 2D platformers or top-down game players."
        }),
        "RigidBody2D" => serde_json::json!({
            "name": "RigidBody2D",
            "category": "2D Physics",
            "description": "A 2D physics simulation object.",
            "inherits": ["PhysicsBody2D", "CollisionObject2D", "Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "mass": "float - Mass",
                "gravity_scale": "float - Gravity scale"
            },
            "common_children": ["CollisionShape2D", "Sprite2D"]
        }),
        "Area2D" => serde_json::json!({
            "name": "Area2D",
            "category": "2D Physics",
            "description": "A 2D collision detection region.",
            "inherits": ["CollisionObject2D", "Node2D", "CanvasItem", "Node"],
            "key_signals": ["body_entered(body)", "body_exited(body)"],
            "common_children": ["CollisionShape2D"]
        }),

        // 2D Visual
        "Sprite2D" => serde_json::json!({
            "name": "Sprite2D",
            "category": "2D Visual",
            "description": "A node that displays a 2D texture.",
            "inherits": ["Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "texture": "Texture2D - Texture to display",
                "flip_h": "bool - Horizontal flip",
                "flip_v": "bool - Vertical flip"
            }
        }),
        "AnimatedSprite2D" => serde_json::json!({
            "name": "AnimatedSprite2D",
            "category": "2D Visual",
            "description": "A node for sprite sheet animation.",
            "inherits": ["Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "sprite_frames": "SpriteFrames - Collection of animation frames",
                "animation": "StringName - Name of the animation being played"
            },
            "key_methods": ["play()", "stop()"]
        }),
        "Camera2D" => serde_json::json!({
            "name": "Camera2D",
            "category": "2D Visual",
            "description": "A camera for 2D scenes. Includes smoothing and limit features.",
            "inherits": ["Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "zoom": "Vector2 - Zoom level",
                "position_smoothing_enabled": "bool - Smooth follow",
                "limit_left/right/top/bottom": "int - Camera movement limits"
            },
            "tips": "Make it a child of the player to follow them."
        }),

        // UI
        "Control" => serde_json::json!({
            "name": "Control",
            "category": "UI",
            "description": "Base class for UI elements. Layout via anchors and margins.",
            "inherits": ["CanvasItem", "Node"],
            "key_properties": {
                "anchor_*": "float - Position relative to parent (0.0 to 1.0)",
                "offset_*": "float - Pixel offset from anchor"
            },
            "tips": "Easy placement via Layout > Anchors Preset."
        }),
        "Button" => serde_json::json!({
            "name": "Button",
            "category": "UI",
            "description": "A clickable button.",
            "inherits": ["BaseButton", "Control", "CanvasItem", "Node"],
            "key_properties": {"text": "String - Button text"},
            "key_signals": ["pressed()"]
        }),
        "Label" => serde_json::json!({
            "name": "Label",
            "category": "UI",
            "description": "A node for displaying text.",
            "inherits": ["Control", "CanvasItem", "Node"],
            "key_properties": {
                "text": "String - Display text",
                "horizontal_alignment": "HorizontalAlignment"
            }
        }),

        // Animation
        "AnimationPlayer" => serde_json::json!({
            "name": "AnimationPlayer",
            "category": "Animation",
            "description": "Manages animation playback. Can animate properties, method calls, and signals.",
            "inherits": ["AnimationMixer", "Node"],
            "key_methods": ["play(name)", "stop()", "queue(name)"],
            "key_signals": ["animation_finished(name)"],
            "tips": "Set keyframes in the Animation editor."
        }),
        "AnimationTree" => serde_json::json!({
            "name": "AnimationTree",
            "category": "Animation",
            "description": "Complex animation control via state machines or blend trees.",
            "inherits": ["AnimationMixer", "Node"],
            "key_properties": {
                "anim_player": "NodePath - AnimationPlayer to be used",
                "active": "bool - Enable"
            },
            "tips": "Ideal for blending movement and attack animations of 3D characters."
        }),

        // Others
        "Node3D" => serde_json::json!({
            "name": "Node3D",
            "category": "3D Base",
            "description": "Base node for 3D space. Has position, rotation, and scale.",
            "inherits": ["Node"],
            "key_properties": {
                "position": "Vector3",
                "rotation": "Vector3 (radians)",
                "scale": "Vector3"
            },
            "tips": "Empty Node3Ds are useful for grouping or markers."
        }),
        "Node2D" => serde_json::json!({
            "name": "Node2D",
            "category": "2D Base",
            "description": "Base node for 2D space.",
            "inherits": ["CanvasItem", "Node"],
            "key_properties": {
                "position": "Vector2",
                "rotation": "float (radians)",
                "scale": "Vector2"
            }
        }),
        "Node" => serde_json::json!({
            "name": "Node",
            "category": "Core",
            "description": "Base class for all nodes.",
            "key_methods": ["_ready()", "_process(delta)", "_physics_process(delta)", "_input(event)"],
            "tips": "Can be used as a logic node with script only."
        }),
        "Timer" => serde_json::json!({
            "name": "Timer",
            "category": "Utility",
            "description": "A timer that fires a signal after a certain time.",
            "inherits": ["Node"],
            "key_properties": {
                "wait_time": "float - Wait time (seconds)",
                "one_shot": "bool - Fire only once",
                "autostart": "bool - Auto-start"
            },
            "key_signals": ["timeout()"],
            "key_methods": ["start()", "stop()"]
        }),
        "AudioStreamPlayer" | "AudioStreamPlayer2D" | "AudioStreamPlayer3D" => serde_json::json!({
            "name": node_type,
            "category": "Audio",
            "description": "An audio playback node. 2D/3D provides spatial audio based on position.",
            "key_properties": {
                "stream": "AudioStream - Audio to play",
                "volume_db": "float - Volume (dB)"
            },
            "key_methods": ["play()", "stop()"],
            "key_signals": ["finished()"]
        }),

        // Unknown node type
        _ => serde_json::json!({
            "name": node_type,
            "found": false,
            "message": format!("Information for node type '{}' is not in the database. Please refer to Godot documentation.", node_type),
            "doc_url": format!("https://docs.godotengine.org/en/stable/classes/class_{}.html", node_type.to_lowercase())
        }),
    }
}
