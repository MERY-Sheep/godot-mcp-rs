//! Node Type Resolver
//!
//! Handles node type information from static database.

use super::types::*;

/// Resolve node type information from static database
pub fn resolve_node_type_info(type_name: &str) -> Option<NodeTypeInfo> {
    let data = get_node_type_data(type_name)?;

    // Convert key_properties to NodePropertyInfo
    let properties: Vec<NodePropertyInfo> = data
        .properties
        .iter()
        .map(|(name, type_hint)| NodePropertyInfo {
            name: name.clone(),
            property_type: type_hint.clone(),
            hint: None,
        })
        .collect();

    // Convert key_signals to SignalInfo
    let signals: Vec<SignalInfo> = data
        .signals
        .iter()
        .map(|sig| SignalInfo {
            name: sig.clone(),
            arguments: vec![],
        })
        .collect();

    Some(NodeTypeInfo {
        type_name: type_name.to_string(),
        properties,
        signals,
    })
}

/// Internal node type data
struct NodeTypeData {
    properties: Vec<(String, String)>,
    signals: Vec<String>,
}

/// Get node type data from static database
fn get_node_type_data(type_name: &str) -> Option<NodeTypeData> {
    match type_name {
        "CharacterBody3D" => Some(NodeTypeData {
            properties: vec![
                ("velocity".to_string(), "Vector3".to_string()),
                ("floor_max_angle".to_string(), "float".to_string()),
                ("motion_mode".to_string(), "MotionMode".to_string()),
                ("up_direction".to_string(), "Vector3".to_string()),
                ("slide_on_ceiling".to_string(), "bool".to_string()),
            ],
            signals: vec![],
        }),
        "RigidBody3D" => Some(NodeTypeData {
            properties: vec![
                ("mass".to_string(), "float".to_string()),
                ("gravity_scale".to_string(), "float".to_string()),
                ("linear_velocity".to_string(), "Vector3".to_string()),
                ("angular_velocity".to_string(), "Vector3".to_string()),
                ("freeze".to_string(), "bool".to_string()),
            ],
            signals: vec!["body_entered".to_string(), "body_exited".to_string()],
        }),
        "Area3D" => Some(NodeTypeData {
            properties: vec![
                ("monitoring".to_string(), "bool".to_string()),
                ("monitorable".to_string(), "bool".to_string()),
                ("priority".to_string(), "int".to_string()),
            ],
            signals: vec![
                "body_entered".to_string(),
                "body_exited".to_string(),
                "area_entered".to_string(),
                "area_exited".to_string(),
            ],
        }),
        "Node3D" => Some(NodeTypeData {
            properties: vec![
                ("position".to_string(), "Vector3".to_string()),
                ("rotation".to_string(), "Vector3".to_string()),
                ("scale".to_string(), "Vector3".to_string()),
                ("visible".to_string(), "bool".to_string()),
            ],
            signals: vec!["visibility_changed".to_string()],
        }),
        "Camera3D" => Some(NodeTypeData {
            properties: vec![
                ("current".to_string(), "bool".to_string()),
                ("fov".to_string(), "float".to_string()),
                ("near".to_string(), "float".to_string()),
                ("far".to_string(), "float".to_string()),
            ],
            signals: vec![],
        }),
        "MeshInstance3D" => Some(NodeTypeData {
            properties: vec![
                ("mesh".to_string(), "Mesh".to_string()),
                ("skeleton".to_string(), "NodePath".to_string()),
                ("skin".to_string(), "Skin".to_string()),
            ],
            signals: vec![],
        }),
        "CharacterBody2D" => Some(NodeTypeData {
            properties: vec![
                ("velocity".to_string(), "Vector2".to_string()),
                ("floor_max_angle".to_string(), "float".to_string()),
                ("motion_mode".to_string(), "MotionMode".to_string()),
            ],
            signals: vec![],
        }),
        "Sprite2D" => Some(NodeTypeData {
            properties: vec![
                ("texture".to_string(), "Texture2D".to_string()),
                ("flip_h".to_string(), "bool".to_string()),
                ("flip_v".to_string(), "bool".to_string()),
                ("centered".to_string(), "bool".to_string()),
            ],
            signals: vec!["texture_changed".to_string()],
        }),
        "AnimationPlayer" => Some(NodeTypeData {
            properties: vec![
                ("current_animation".to_string(), "StringName".to_string()),
                ("autoplay".to_string(), "String".to_string()),
                ("speed_scale".to_string(), "float".to_string()),
            ],
            signals: vec![
                "animation_started".to_string(),
                "animation_finished".to_string(),
                "animation_changed".to_string(),
            ],
        }),
        "Timer" => Some(NodeTypeData {
            properties: vec![
                ("wait_time".to_string(), "float".to_string()),
                ("one_shot".to_string(), "bool".to_string()),
                ("autostart".to_string(), "bool".to_string()),
            ],
            signals: vec!["timeout".to_string()],
        }),
        "Button" => Some(NodeTypeData {
            properties: vec![
                ("text".to_string(), "String".to_string()),
                ("disabled".to_string(), "bool".to_string()),
            ],
            signals: vec![
                "pressed".to_string(),
                "button_down".to_string(),
                "button_up".to_string(),
            ],
        }),
        "Label" => Some(NodeTypeData {
            properties: vec![
                ("text".to_string(), "String".to_string()),
                (
                    "horizontal_alignment".to_string(),
                    "HorizontalAlignment".to_string(),
                ),
                (
                    "vertical_alignment".to_string(),
                    "VerticalAlignment".to_string(),
                ),
            ],
            signals: vec![],
        }),
        "Node" => Some(NodeTypeData {
            properties: vec![
                ("name".to_string(), "StringName".to_string()),
                ("process_mode".to_string(), "ProcessMode".to_string()),
            ],
            signals: vec![
                "ready".to_string(),
                "tree_entered".to_string(),
                "tree_exited".to_string(),
            ],
        }),
        _ => None,
    }
}
