//! プロジェクト関連ツール - ファイル操作・検索

use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};
use std::path::{Path, PathBuf};

use super::{
    GetNodeTypeInfoRequest, GetProjectStatsRequest, GodotTools, ListFilesRequest, ReadFileRequest,
    SearchInProjectRequest, ValidateProjectRequest,
};
use crate::godot::tscn::GodotScene;

impl GodotTools {
    /// list_project_files - プロジェクトファイル一覧
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

    /// read_file - ファイル読み取り
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

    /// list_all_scenes - 全シーン一覧
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

    /// search_in_project - プロジェクト内検索
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

    /// get_node_type_info - ノード型の詳細情報を取得
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

    /// get_project_stats - プロジェクト統計
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

        // ノード数を計算
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

    /// validate_project - プロジェクト検証
    pub async fn handle_validate_project(
        &self,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let _req: ValidateProjectRequest = Default::default();
        let base = self.get_base_path();

        let mut issues: Vec<serde_json::Value> = Vec::new();

        // 全tscnファイル収集
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

        // 使用中のスクリプトパスを収集
        let mut used_scripts: std::collections::HashSet<String> = std::collections::HashSet::new();

        for file_path in find_tscn_files_val(base) {
            let rel_path = file_path.strip_prefix(base).unwrap_or(&file_path);
            let res_path = format!("res://{}", rel_path.to_string_lossy().replace('\\', "/"));

            if let Ok(content) = std::fs::read_to_string(&file_path) {
                match GodotScene::parse(&content) {
                    Ok(scene) => {
                        // 空シーン
                        if scene.nodes.is_empty() {
                            issues.push(serde_json::json!({
                                "type": "empty_scene",
                                "file": res_path,
                                "severity": "warning",
                            }));
                        }

                        // スクリプト参照を収集
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

        // 未使用スクリプト検出
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

/// Godot ノード型の静的データベース
fn get_node_type_database(node_type: &str) -> serde_json::Value {
    match node_type {
        // 3D Physics Bodies
        "CharacterBody3D" => serde_json::json!({
            "name": "CharacterBody3D",
            "category": "3D Physics",
            "description": "3Dキャラクター移動用の物理ボディ。move_and_slide()で衝突を処理しながら移動可能。",
            "inherits": ["PhysicsBody3D", "CollisionObject3D", "Node3D", "Node"],
            "key_properties": {
                "velocity": "Vector3 - 現在の速度ベクトル",
                "floor_max_angle": "float - 床として判定される最大角度(rad)",
                "motion_mode": "MotionMode - GROUNDED or FLOATING"
            },
            "key_methods": ["move_and_slide()", "is_on_floor()", "is_on_wall()", "get_floor_normal()"],
            "common_children": ["CollisionShape3D", "MeshInstance3D", "Camera3D", "AnimationPlayer"],
            "script_template": "CharacterBody3D",
            "tips": "CollisionShape3Dを子として必ず追加。velocityを設定してmove_and_slide()を呼ぶ。"
        }),
        "RigidBody3D" => serde_json::json!({
            "name": "RigidBody3D",
            "category": "3D Physics",
            "description": "物理シミュレーションで動くオブジェクト。重力や衝突が自動処理される。",
            "inherits": ["PhysicsBody3D", "CollisionObject3D", "Node3D", "Node"],
            "key_properties": {
                "mass": "float - 質量(kg)",
                "gravity_scale": "float - 重力の影響係数",
                "linear_velocity": "Vector3 - 線形速度",
                "angular_velocity": "Vector3 - 角速度"
            },
            "key_methods": ["apply_force()", "apply_impulse()", "apply_torque()"],
            "common_children": ["CollisionShape3D", "MeshInstance3D"],
            "tips": "プレイヤー操作には不向き。投擲物やオブジェクト用。"
        }),
        "StaticBody3D" => serde_json::json!({
            "name": "StaticBody3D",
            "category": "3D Physics",
            "description": "動かない静的なコリジョンオブジェクト。地形や壁に最適。",
            "inherits": ["PhysicsBody3D", "CollisionObject3D", "Node3D", "Node"],
            "key_properties": {
                "physics_material_override": "PhysicsMaterial - 摩擦・弾性"
            },
            "common_children": ["CollisionShape3D", "MeshInstance3D"],
            "tips": "大量配置してもパフォーマンス良好。レベルジオメトリに使用。"
        }),
        "Area3D" => serde_json::json!({
            "name": "Area3D",
            "category": "3D Physics",
            "description": "衝突検出用の領域。シグナルで入退出を検知。ダメージゾーンやトリガーに最適。",
            "inherits": ["CollisionObject3D", "Node3D", "Node"],
            "key_signals": ["body_entered(body)", "body_exited(body)", "area_entered(area)"],
            "common_children": ["CollisionShape3D"],
            "tips": "monitoring=trueで検出有効。monitorable=trueで他Areaから検出される。"
        }),

        // 3D Visual
        "MeshInstance3D" => serde_json::json!({
            "name": "MeshInstance3D",
            "category": "3D Visual",
            "description": "3Dメッシュを表示するノード。",
            "inherits": ["GeometryInstance3D", "VisualInstance3D", "Node3D", "Node"],
            "key_properties": {
                "mesh": "Mesh - 表示するメッシュリソース",
                "surface_material_override": "Material - マテリアル上書き"
            },
            "common_meshes": ["BoxMesh", "SphereMesh", "CapsuleMesh", "CylinderMesh", "PlaneMesh"],
            "tips": "PrimitiveMeshを使うと形状を簡単に作成可能。"
        }),
        "Camera3D" => serde_json::json!({
            "name": "Camera3D",
            "category": "3D Visual",
            "description": "3Dシーンを描画するカメラ。current=trueでアクティブ化。",
            "inherits": ["Node3D", "Node"],
            "key_properties": {
                "current": "bool - このカメラをアクティブにする",
                "fov": "float - 視野角(度)",
                "near": "float - ニアクリップ距離",
                "far": "float - ファークリップ距離"
            },
            "key_methods": ["make_current()", "project_ray_origin()", "unproject_position()"],
            "tips": "TPS: プレイヤーの子として配置。FPS: Head子ノードの子として配置。"
        }),
        "DirectionalLight3D" => serde_json::json!({
            "name": "DirectionalLight3D",
            "category": "3D Visual",
            "description": "太陽光のような無限遠からの平行光源。",
            "inherits": ["Light3D", "VisualInstance3D", "Node3D", "Node"],
            "key_properties": {
                "light_energy": "float - 光の強さ",
                "shadow_enabled": "bool - 影を生成"
            },
            "tips": "シーンに1つ配置。回転でsun方向を制御。"
        }),

        // 2D Physics Bodies
        "CharacterBody2D" => serde_json::json!({
            "name": "CharacterBody2D",
            "category": "2D Physics",
            "description": "2Dキャラクター移動用の物理ボディ。",
            "inherits": ["PhysicsBody2D", "CollisionObject2D", "Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "velocity": "Vector2 - 現在の速度ベクトル",
                "floor_max_angle": "float - 床として判定される最大角度"
            },
            "key_methods": ["move_and_slide()", "is_on_floor()", "is_on_wall()"],
            "common_children": ["CollisionShape2D", "Sprite2D", "AnimatedSprite2D"],
            "script_template": "CharacterBody2D",
            "tips": "2Dプラットフォーマーやトップダウンゲームのプレイヤーに最適。"
        }),
        "RigidBody2D" => serde_json::json!({
            "name": "RigidBody2D",
            "category": "2D Physics",
            "description": "2D物理シミュレーションオブジェクト。",
            "inherits": ["PhysicsBody2D", "CollisionObject2D", "Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "mass": "float - 質量",
                "gravity_scale": "float - 重力係数"
            },
            "common_children": ["CollisionShape2D", "Sprite2D"]
        }),
        "Area2D" => serde_json::json!({
            "name": "Area2D",
            "category": "2D Physics",
            "description": "2D衝突検出領域。",
            "inherits": ["CollisionObject2D", "Node2D", "CanvasItem", "Node"],
            "key_signals": ["body_entered(body)", "body_exited(body)"],
            "common_children": ["CollisionShape2D"]
        }),

        // 2D Visual
        "Sprite2D" => serde_json::json!({
            "name": "Sprite2D",
            "category": "2D Visual",
            "description": "2Dテクスチャを表示するノード。",
            "inherits": ["Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "texture": "Texture2D - 表示するテクスチャ",
                "flip_h": "bool - 水平反転",
                "flip_v": "bool - 垂直反転"
            }
        }),
        "AnimatedSprite2D" => serde_json::json!({
            "name": "AnimatedSprite2D",
            "category": "2D Visual",
            "description": "スプライトシートアニメーション用ノード。",
            "inherits": ["Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "sprite_frames": "SpriteFrames - アニメーションフレーム集",
                "animation": "StringName - 再生中のアニメーション名"
            },
            "key_methods": ["play()", "stop()"]
        }),
        "Camera2D" => serde_json::json!({
            "name": "Camera2D",
            "category": "2D Visual",
            "description": "2Dシーン用カメラ。スムージングやリミット機能付き。",
            "inherits": ["Node2D", "CanvasItem", "Node"],
            "key_properties": {
                "zoom": "Vector2 - ズームレベル",
                "position_smoothing_enabled": "bool - スムーズ追従",
                "limit_left/right/top/bottom": "int - カメラ移動制限"
            },
            "tips": "プレイヤーの子にして追従させる。"
        }),

        // UI
        "Control" => serde_json::json!({
            "name": "Control",
            "category": "UI",
            "description": "UI要素の基底クラス。アンカーとマージンでレイアウト。",
            "inherits": ["CanvasItem", "Node"],
            "key_properties": {
                "anchor_*": "float - 0.0~1.0で親に対する位置",
                "offset_*": "float - アンカーからのピクセルオフセット"
            },
            "tips": "Layout > Anchors Presetで簡単配置。"
        }),
        "Button" => serde_json::json!({
            "name": "Button",
            "category": "UI",
            "description": "クリック可能なボタン。",
            "inherits": ["BaseButton", "Control", "CanvasItem", "Node"],
            "key_properties": {"text": "String - ボタンテキスト"},
            "key_signals": ["pressed()"]
        }),
        "Label" => serde_json::json!({
            "name": "Label",
            "category": "UI",
            "description": "テキスト表示用ノード。",
            "inherits": ["Control", "CanvasItem", "Node"],
            "key_properties": {
                "text": "String - 表示テキスト",
                "horizontal_alignment": "HorizontalAlignment"
            }
        }),

        // Animation
        "AnimationPlayer" => serde_json::json!({
            "name": "AnimationPlayer",
            "category": "Animation",
            "description": "アニメーション再生を管理。プロパティ、メソッド呼び出し、シグナルをアニメーション可能。",
            "inherits": ["AnimationMixer", "Node"],
            "key_methods": ["play(name)", "stop()", "queue(name)"],
            "key_signals": ["animation_finished(name)"],
            "tips": "Animationエディタでキーフレームを設定。"
        }),
        "AnimationTree" => serde_json::json!({
            "name": "AnimationTree",
            "category": "Animation",
            "description": "ステートマシンやブレンドツリーで複雑なアニメーション制御。",
            "inherits": ["AnimationMixer", "Node"],
            "key_properties": {
                "anim_player": "NodePath - 使用するAnimationPlayer",
                "active": "bool - 有効化"
            },
            "tips": "3Dキャラクターの移動・攻撃アニメブレンドに最適。"
        }),

        // その他
        "Node3D" => serde_json::json!({
            "name": "Node3D",
            "category": "3D Base",
            "description": "3D空間の基底ノード。位置・回転・スケールを持つ。",
            "inherits": ["Node"],
            "key_properties": {
                "position": "Vector3",
                "rotation": "Vector3 (radians)",
                "scale": "Vector3"
            },
            "tips": "空のNode3Dはグループ化やマーカーに便利。"
        }),
        "Node2D" => serde_json::json!({
            "name": "Node2D",
            "category": "2D Base",
            "description": "2D空間の基底ノード。",
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
            "description": "すべてのノードの基底クラス。",
            "key_methods": ["_ready()", "_process(delta)", "_physics_process(delta)", "_input(event)"],
            "tips": "スクリプトのみのロジックノードとして使用可能。"
        }),
        "Timer" => serde_json::json!({
            "name": "Timer",
            "category": "Utility",
            "description": "一定時間後にシグナルを発火するタイマー。",
            "inherits": ["Node"],
            "key_properties": {
                "wait_time": "float - 待機時間(秒)",
                "one_shot": "bool - 一回だけ発火",
                "autostart": "bool - 自動開始"
            },
            "key_signals": ["timeout()"],
            "key_methods": ["start()", "stop()"]
        }),
        "AudioStreamPlayer" | "AudioStreamPlayer2D" | "AudioStreamPlayer3D" => serde_json::json!({
            "name": node_type,
            "category": "Audio",
            "description": "オーディオ再生ノード。2D/3Dは位置に応じた空間オーディオ。",
            "key_properties": {
                "stream": "AudioStream - 再生するオーディオ",
                "volume_db": "float - 音量(dB)"
            },
            "key_methods": ["play()", "stop()"],
            "key_signals": ["finished()"]
        }),

        // 不明なノード型
        _ => serde_json::json!({
            "name": node_type,
            "found": false,
            "message": format!("ノード型 '{}' の情報がデータベースにありません。Godotドキュメントを参照してください。", node_type),
            "doc_url": format!("https://docs.godotengine.org/en/stable/classes/class_{}.html", node_type.to_lowercase())
        }),
    }
}
