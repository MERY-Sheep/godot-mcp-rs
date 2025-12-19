//! Scene Related Tools - Creation, Editing & Analysis

use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};
use std::collections::HashMap;

use super::{
    AddNodeRequest, BatchAddNodesRequest, CompareScenesRequest, CopySceneRequest,
    CreateSceneFromTemplateRequest, CreateSceneRequest, ExportSceneAsJsonRequest,
    GetNodeTreeRequest, GetSceneMetadataRequest, GodotTools, ReadSceneRequest, RemoveNodeRequest,
    SetNodePropertyRequest, ValidateTscnRequest,
};
use crate::godot::tscn::{GodotScene, SceneNode};

impl GodotTools {
    /// create_scene - Create scene
    pub(super) async fn handle_create_scene(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: CreateSceneRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.path);

        let root_name = req.root_name.unwrap_or_else(|| {
            full_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Root")
                .to_string()
        });

        let scene = GodotScene::new(&root_name, &req.root_type);
        let content = scene.to_tscn();

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }
        std::fs::write(&full_path, &content)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created scene: {}",
            req.path
        ))]))
    }

    /// read_scene - Read scene
    pub(super) async fn handle_read_scene(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ReadSceneRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read scene: {}", e), None))?;

        let scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse scene: {}", e), None))?;

        let result = serde_json::json!({
            "format_version": scene.format_version(),
            "load_steps": scene.load_steps(),
            "nodes": scene.nodes.iter().map(|n| {
                serde_json::json!({
                    "name": n.name,
                    "type": n.node_type,
                    "parent": n.parent,
                    "properties": n.properties,
                })
            }).collect::<Vec<_>>(),
            "external_resources": scene.external_resources(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// add_node - Add node
    pub(super) async fn handle_add_node(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AddNodeRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.scene_path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read scene: {}", e), None))?;

        let mut scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse scene: {}", e), None))?;

        scene.add_node(SceneNode {
            name: req.name.clone(),
            node_type: req.node_type.clone(),
            parent: Some(req.parent.clone()),
            properties: HashMap::new(),
        });

        std::fs::write(&full_path, scene.to_tscn())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added node '{}' (type: {}) under '{}'",
            req.name, req.node_type, req.parent
        ))]))
    }

    /// remove_node - Remove node
    pub(super) async fn handle_remove_node(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: RemoveNodeRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.scene_path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read scene: {}", e), None))?;

        let mut scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse scene: {}", e), None))?;

        scene
            .remove_node(&req.node_path)
            .map_err(|e| McpError::internal_error(e, None))?;

        std::fs::write(&full_path, scene.to_tscn())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Removed node '{}'",
            req.node_path
        ))]))
    }

    /// set_node_property - Set property
    pub(super) async fn handle_set_node_property(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: SetNodePropertyRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.scene_path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read scene: {}", e), None))?;

        let mut scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse scene: {}", e), None))?;

        scene
            .set_property(&req.node_path, &req.property, &req.value)
            .map_err(|e| McpError::internal_error(e, None))?;

        std::fs::write(&full_path, scene.to_tscn())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set {}.{} = {}",
            req.node_path, req.property, req.value
        ))]))
    }

    /// get_node_tree - Get node tree
    pub(super) async fn handle_get_node_tree(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: GetNodeTreeRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read scene: {}", e), None))?;

        let scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse scene: {}", e), None))?;

        let mut tree = String::new();
        for node in &scene.nodes {
            let depth = if node.parent.is_none() {
                0
            } else if node.parent.as_deref() == Some(".") {
                1
            } else {
                node.parent
                    .as_ref()
                    .map(|p| p.matches('/').count() + 2)
                    .unwrap_or(1)
            };
            let indent = "  ".repeat(depth);
            tree.push_str(&format!("{}{} ({})\n", indent, node.name, node.node_type));
        }

        Ok(CallToolResult::success(vec![Content::text(tree)]))
    }

    /// validate_tscn - Validate scene
    pub(super) async fn handle_validate_tscn(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ValidateTscnRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        match GodotScene::parse(&content) {
            Ok(scene) => {
                let mut issues: Vec<String> = Vec::new();

                if scene.nodes.is_empty() {
                    issues.push("No root node".to_string());
                }

                for node in &scene.nodes {
                    if let Some(ref parent) = node.parent {
                        if parent != "."
                            && !scene
                                .nodes
                                .iter()
                                .any(|n| n.name == parent.split('/').next().unwrap_or(""))
                        {
                            // For future validation
                        }
                    }
                }

                let result = if issues.is_empty() {
                    serde_json::json!({
                        "valid": true,
                        "node_count": scene.nodes.len(),
                        "resource_count": scene.ext_resources.len(),
                        "message": "Validation successful"
                    })
                } else {
                    serde_json::json!({
                        "valid": false,
                        "issues": issues,
                        "node_count": scene.nodes.len(),
                    })
                };

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
            Err(e) => {
                let result = serde_json::json!({
                    "valid": false,
                    "parse_error": e.to_string(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
        }
    }

    /// copy_scene - Copy scene
    pub(super) async fn handle_copy_scene(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: CopySceneRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let source = base.join(&req.source);
        let dest = base.join(&req.destination);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        std::fs::copy(&source, &dest)
            .map_err(|e| McpError::internal_error(format!("Failed to copy: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Copied '{}' to '{}'",
            req.source, req.destination
        ))]))
    }

    /// get_scene_metadata - Get scene metadata
    pub(super) async fn handle_get_scene_metadata(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: GetSceneMetadataRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse: {}", e), None))?;

        let root = scene.nodes.first();
        let node_types: std::collections::HashMap<String, usize> =
            scene
                .nodes
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, n| {
                    *acc.entry(n.node_type.clone()).or_insert(0) += 1;
                    acc
                });

        let result = serde_json::json!({
            "path": req.path,
            "format_version": scene.format_version(),
            "root_node": root.map(|n| serde_json::json!({
                "name": n.name,
                "type": n.node_type,
            })),
            "total_nodes": scene.nodes.len(),
            "node_types": node_types,
            "external_resources": scene.ext_resources.len(),
            "has_scripts": scene.nodes.iter().any(|n| n.properties.contains_key("script")),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// compare_scenes - Compare scenes
    pub(super) async fn handle_compare_scenes(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: CompareScenesRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();

        let path_a = req.path_a.strip_prefix("res://").unwrap_or(&req.path_a);
        let path_b = req.path_b.strip_prefix("res://").unwrap_or(&req.path_b);

        let content_a = std::fs::read_to_string(base.join(path_a)).map_err(|e| {
            McpError::internal_error(format!("Failed to read {}: {}", path_a, e), None)
        })?;
        let content_b = std::fs::read_to_string(base.join(path_b)).map_err(|e| {
            McpError::internal_error(format!("Failed to read {}: {}", path_b, e), None)
        })?;

        let scene_a = GodotScene::parse(&content_a).map_err(|e| {
            McpError::internal_error(format!("Failed to parse {}: {}", path_a, e), None)
        })?;
        let scene_b = GodotScene::parse(&content_b).map_err(|e| {
            McpError::internal_error(format!("Failed to parse {}: {}", path_b, e), None)
        })?;

        let nodes_a: std::collections::HashSet<String> =
            scene_a.nodes.iter().map(|n| n.name.clone()).collect();
        let nodes_b: std::collections::HashSet<String> =
            scene_b.nodes.iter().map(|n| n.name.clone()).collect();

        let added_nodes: Vec<&String> = nodes_b.difference(&nodes_a).collect();
        let removed_nodes: Vec<&String> = nodes_a.difference(&nodes_b).collect();

        let mut modified_properties: serde_json::Map<String, serde_json::Value> =
            serde_json::Map::new();

        for node_a in &scene_a.nodes {
            if let Some(node_b) = scene_b.nodes.iter().find(|n| n.name == node_a.name) {
                let mut prop_changes: serde_json::Map<String, serde_json::Value> =
                    serde_json::Map::new();

                for (key, val_a) in &node_a.properties {
                    if let Some(val_b) = node_b.properties.get(key) {
                        if val_a != val_b {
                            prop_changes.insert(
                                key.clone(),
                                serde_json::json!({
                                    "old": val_a,
                                    "new": val_b,
                                }),
                            );
                        }
                    } else {
                        prop_changes.insert(
                            key.clone(),
                            serde_json::json!({
                                "old": val_a,
                                "new": null,
                            }),
                        );
                    }
                }

                for (key, val_b) in &node_b.properties {
                    if !node_a.properties.contains_key(key) {
                        prop_changes.insert(
                            key.clone(),
                            serde_json::json!({
                                "old": null,
                                "new": val_b,
                            }),
                        );
                    }
                }

                if !prop_changes.is_empty() {
                    modified_properties
                        .insert(node_a.name.clone(), serde_json::Value::Object(prop_changes));
                }
            }
        }

        let result = serde_json::json!({
            "path_a": req.path_a,
            "path_b": req.path_b,
            "differences": {
                "added_nodes": added_nodes,
                "removed_nodes": removed_nodes,
                "modified_properties": modified_properties,
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// export_scene_as_json - Export scene as JSON
    pub(super) async fn handle_export_scene_as_json(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ExportSceneAsJsonRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse: {}", e), None))?;

        fn build_node_tree(
            nodes: &[SceneNode],
            parent_path: Option<&str>,
        ) -> Vec<serde_json::Value> {
            let expected_parent = parent_path.unwrap_or(".");

            nodes
                .iter()
                .filter(|n| match (&n.parent, parent_path) {
                    (None, None) => true,
                    (Some(p), Some(expected)) if p == expected => true,
                    (Some(p), None) if p == "." => true,
                    _ => false,
                })
                .map(|n| {
                    let child_parent = if parent_path.is_none() {
                        n.name.clone()
                    } else {
                        format!("{}/{}", expected_parent, n.name)
                    };

                    let children = build_node_tree(nodes, Some(&child_parent));

                    let mut node_obj = serde_json::json!({
                        "name": n.name,
                        "type": n.node_type,
                        "properties": n.properties,
                    });

                    if !children.is_empty() {
                        node_obj["children"] = serde_json::Value::Array(children);
                    }

                    node_obj
                })
                .collect()
        }

        let root = scene
            .nodes
            .first()
            .ok_or_else(|| McpError::internal_error("Scene has no root node".to_string(), None))?;

        let children = build_node_tree(&scene.nodes, Some("."));

        let mut result = serde_json::json!({
            "name": root.name,
            "type": root.node_type,
            "properties": root.properties,
        });

        if !children.is_empty() {
            result["children"] = serde_json::Value::Array(children);
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// batch_add_nodes - Batch add nodes
    pub(super) async fn handle_batch_add_nodes(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: BatchAddNodesRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req
            .scene_path
            .strip_prefix("res://")
            .unwrap_or(&req.scene_path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let mut scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse: {}", e), None))?;

        let mut added = Vec::new();
        for entry in &req.nodes {
            scene.add_node(SceneNode {
                name: entry.name.clone(),
                node_type: entry.node_type.clone(),
                parent: Some(entry.parent.clone()),
                properties: std::collections::HashMap::new(),
            });
            added.push(format!("{} ({})", entry.name, entry.node_type));
        }

        std::fs::write(&full_path, scene.to_tscn())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added {} nodes: {}",
            added.len(),
            added.join(", ")
        ))]))
    }

    /// create_scene_from_template - Create scene from template
    pub(super) async fn handle_create_scene_from_template(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: CreateSceneFromTemplateRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.path);

        let root_name = req.root_name.unwrap_or_else(|| {
            full_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Root")
                .to_string()
        });

        // Build scene based on template
        let (root_type, nodes) = match req.template.as_str() {
            "player_3d" => (
                "CharacterBody3D",
                vec![
                    ("CollisionShape3D", "Collision"),
                    ("MeshInstance3D", "Mesh"),
                    ("Camera3D", "Camera"),
                    ("AnimationPlayer", "AnimationPlayer"),
                ],
            ),
            "player_2d" => (
                "CharacterBody2D",
                vec![
                    ("CollisionShape2D", "Collision"),
                    ("Sprite2D", "Sprite"),
                    ("AnimatedSprite2D", "AnimatedSprite"),
                    ("Camera2D", "Camera"),
                ],
            ),
            "enemy_3d" => (
                "CharacterBody3D",
                vec![
                    ("CollisionShape3D", "Collision"),
                    ("MeshInstance3D", "Mesh"),
                    ("NavigationAgent3D", "NavigationAgent"),
                    ("Area3D", "HitBox"),
                ],
            ),
            "level_3d" => (
                "Node3D",
                vec![
                    ("DirectionalLight3D", "Sun"),
                    ("WorldEnvironment", "Environment"),
                    ("Node3D", "Geometry"),
                    ("Node3D", "Spawns"),
                ],
            ),
            "ui_menu" => (
                "Control",
                vec![
                    ("VBoxContainer", "Container"),
                    ("Label", "Title"),
                    ("Button", "StartButton"),
                    ("Button", "OptionsButton"),
                    ("Button", "QuitButton"),
                ],
            ),
            _ => {
                return Err(McpError::invalid_params(
                    format!(
                        "Unknown template: '{}'. Available: player_3d, player_2d, enemy_3d, level_3d, ui_menu",
                        req.template
                    ),
                    None,
                ));
            }
        };

        let mut scene = GodotScene::new(&root_name, root_type);

        for (node_type, node_name) in &nodes {
            scene.add_node(SceneNode {
                name: node_name.to_string(),
                node_type: node_type.to_string(),
                parent: Some(".".to_string()),
                properties: HashMap::new(),
            });
        }

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }
        std::fs::write(&full_path, scene.to_tscn())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let node_list: Vec<String> = nodes
            .iter()
            .map(|(t, n)| format!("{} ({})", n, t))
            .collect();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created {} scene '{}' with template '{}'\nRoot: {} ({})\nChildren: {}",
            req.template,
            req.path,
            req.template,
            root_name,
            root_type,
            node_list.join(", ")
        ))]))
    }
}
