//! Scene Resolver
//!
//! Handles scene parsing, conversion, and creation.

use std::fs;

use crate::godot::tscn::GodotScene;
use crate::path_utils;

use super::context::GqlContext;
use super::types::*;

/// Resolve scene from file path
pub fn resolve_scene(ctx: &GqlContext, res_path: &str) -> Option<Scene> {
    let file_path = path_utils::to_fs_path_unchecked(&ctx.project_path, res_path);
    let content = fs::read_to_string(&file_path).ok()?;
    let godot_scene = GodotScene::parse(&content).ok()?;

    Some(convert_godot_scene_to_gql(&godot_scene, res_path))
}

/// Convert GodotScene to GraphQL Scene
pub fn convert_godot_scene_to_gql(scene: &GodotScene, path: &str) -> Scene {
    // Build node tree
    let all_nodes: Vec<SceneNode> = scene
        .nodes
        .iter()
        .map(|n| {
            let node_path = if n.parent.is_none() {
                ".".to_string()
            } else if n.parent.as_deref() == Some(".") {
                n.name.clone()
            } else {
                format!("{}/{}", n.parent.as_deref().unwrap_or("."), n.name)
            };

            SceneNode {
                name: n.name.clone(),
                r#type: n.node_type.clone(),
                path: node_path,
                properties: n
                    .properties
                    .iter()
                    .map(|(k, v)| Property {
                        name: k.clone(),
                        value: v.clone(),
                        property_type: None,
                    })
                    .collect(),
                children: vec![], // Filled later if needed
                script: None,     // TODO: Parse script reference
                groups: vec![],   // TODO: Parse groups
                signals: vec![],  // TODO: Parse signal connections
            }
        })
        .collect();

    // Root is the first node (no parent)
    let root = all_nodes.first().cloned().unwrap_or_else(|| SceneNode {
        name: "Root".to_string(),
        r#type: "Node".to_string(),
        path: ".".to_string(),
        properties: vec![],
        children: vec![],
        script: None,
        groups: vec![],
        signals: vec![],
    });

    // External resources
    let external_resources: Vec<ExternalResource> = scene
        .ext_resources
        .iter()
        .map(|r| ExternalResource {
            id: r.id.parse().unwrap_or(0),
            resource_type: r.resource_type.clone(),
            path: r.path.clone(),
        })
        .collect();

    Scene {
        path: path.to_string(),
        root,
        all_nodes,
        external_resources,
    }
}

/// Create a new scene file
pub fn create_scene(ctx: &GqlContext, input: &CreateSceneInput) -> SceneResult {
    let project_path = &ctx.project_path;

    // Convert res:// path to filesystem path
    let file_path = path_utils::to_fs_path_unchecked(project_path, &input.path);

    // Check if file already exists
    if file_path.exists() {
        return SceneResult {
            success: false,
            scene: None,
            message: Some(format!("Scene already exists: {}", input.path)),
        };
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return SceneResult {
                success: false,
                scene: None,
                message: Some(format!("Failed to create directory: {}", e)),
            };
        }
    }

    // Generate minimal tscn content
    let tscn_content = format!(
        r#"[gd_scene format=3]

[node name="{}" type="{}"]
"#,
        input.root_name, input.root_type
    );

    // Write file
    if let Err(e) = fs::write(&file_path, tscn_content) {
        return SceneResult {
            success: false,
            scene: None,
            message: Some(format!("Failed to write scene: {}", e)),
        };
    }

    SceneResult {
        success: true,
        scene: None, // Could load and return the scene
        message: Some(format!("Created scene: {}", input.path)),
    }
}
