//! Live operation resolvers
//!
//! Connects GraphQL queries/mutations to Godot editor plugin via HTTP.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::context::GqlContext;
use super::types::*;

// ======================
// HTTP Client
// ======================

/// Execute a command to the Godot editor plugin
pub async fn execute_live_command(
    ctx: &GqlContext,
    command: GodotLiveCommand,
) -> Result<Value, LiveError> {
    let url = format!("http://localhost:{}", ctx.godot_port);
    let timeout = Duration::from_millis(ctx.timeout_ms);

    let client = Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| LiveError::Connection(e.to_string()))?;

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&command)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                LiveError::Timeout
            } else if e.is_connect() {
                LiveError::Connection(format!("Failed to connect to Godot plugin: {}", e))
            } else {
                LiveError::Connection(e.to_string())
            }
        })?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    if status.is_success() {
        serde_json::from_str(&text)
            .unwrap_or(Value::String(text))
            .pipe(Ok)
    } else {
        Err(LiveError::HttpError {
            status: status.as_u16(),
            message: text,
        })
    }
}

/// Helper trait for pipe syntax
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

// ======================
// Error Types
// ======================

#[derive(Debug)]
pub enum LiveError {
    Connection(String),
    Timeout,
    HttpError { status: u16, message: String },
}

impl std::fmt::Display for LiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiveError::Connection(msg) => write!(f, "Connection error: {}", msg),
            LiveError::Timeout => write!(f, "Request timed out"),
            LiveError::HttpError { status, message } => {
                write!(f, "HTTP error ({}): {}", status, message)
            }
        }
    }
}

// ======================
// Godot Commands
// ======================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "params")]
pub enum GodotLiveCommand {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "get_tree")]
    GetTree,
    #[serde(rename = "add_node")]
    AddNode {
        parent: String,
        name: String,
        node_type: String,
    },
    #[serde(rename = "remove_node")]
    RemoveNode { node_path: String },
    #[serde(rename = "set_property")]
    SetProperty {
        node_path: String,
        property: String,
        value: Value,
    },
    #[serde(rename = "connect_signal")]
    ConnectSignal {
        source: String,
        signal: String,
        target: String,
        method: String,
    },
    #[serde(rename = "disconnect_signal")]
    DisconnectSignal {
        source: String,
        signal: String,
        target: String,
        method: String,
    },
    #[serde(rename = "add_to_group")]
    AddToGroup { node_path: String, group: String },
    #[serde(rename = "remove_from_group")]
    RemoveFromGroup { node_path: String, group: String },
    #[serde(rename = "save_scene")]
    SaveScene,
    #[serde(rename = "open_scene")]
    OpenScene { scene_path: String },
}

// ======================
// Query Resolvers
// ======================

/// Resolve currentScene query
pub async fn resolve_current_scene(ctx: &GqlContext) -> Option<LiveScene> {
    let result = execute_live_command(ctx, GodotLiveCommand::GetTree).await;

    match result {
        Ok(value) => parse_live_scene_from_tree(&value),
        Err(_) => None,
    }
}

/// Resolve node query
pub async fn resolve_node(ctx: &GqlContext, path: String) -> Option<LiveNode> {
    let result = execute_live_command(ctx, GodotLiveCommand::GetTree).await;

    match result {
        Ok(value) => find_node_in_tree(&value, &path),
        Err(_) => None,
    }
}

// ======================
// Mutation Resolvers
// ======================

/// Resolve addNode mutation
pub async fn resolve_add_node(ctx: &GqlContext, input: AddNodeInput) -> NodeResult {
    let command = GodotLiveCommand::AddNode {
        parent: input.parent.clone(),
        name: input.name.clone(),
        node_type: input.node_type.clone(),
    };

    match execute_live_command(ctx, command).await {
        Ok(_) => {
            // Construct node path
            let node_path = if input.parent == "." || input.parent == "/root" {
                format!("/root/{}", input.name)
            } else {
                format!("{}/{}", input.parent, input.name)
            };

            NodeResult {
                success: true,
                node: Some(LiveNode {
                    name: input.name,
                    r#type: input.node_type,
                    path: node_path,
                    global_position: None,
                    global_position_2d: None,
                    properties: vec![],
                    children: vec![],
                    available_signals: vec![],
                    connected_signals: vec![],
                }),
                message: None,
            }
        }
        Err(e) => NodeResult {
            success: false,
            node: None,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve removeNode mutation
pub async fn resolve_remove_node(ctx: &GqlContext, path: String) -> OperationResult {
    let command = GodotLiveCommand::RemoveNode { node_path: path };

    match execute_live_command(ctx, command).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve setProperty mutation
pub async fn resolve_set_property(ctx: &GqlContext, input: SetPropertyInput) -> OperationResult {
    let value = serde_json::from_str(&input.value).unwrap_or(Value::String(input.value.clone()));

    let command = GodotLiveCommand::SetProperty {
        node_path: input.node_path,
        property: input.property,
        value,
    };

    match execute_live_command(ctx, command).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve connectSignal mutation
pub async fn resolve_connect_signal(
    ctx: &GqlContext,
    input: ConnectSignalInput,
) -> OperationResult {
    let command = GodotLiveCommand::ConnectSignal {
        source: input.from_node,
        signal: input.signal,
        target: input.to_node,
        method: input.method,
    };

    match execute_live_command(ctx, command).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve disconnectSignal mutation
pub async fn resolve_disconnect_signal(
    ctx: &GqlContext,
    input: DisconnectSignalInput,
) -> OperationResult {
    let command = GodotLiveCommand::DisconnectSignal {
        source: input.from_node,
        signal: input.signal,
        target: input.to_node,
        method: input.method,
    };

    match execute_live_command(ctx, command).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve addToGroup mutation
pub async fn resolve_add_to_group(
    ctx: &GqlContext,
    node_path: String,
    group: String,
) -> OperationResult {
    let command = GodotLiveCommand::AddToGroup { node_path, group };

    match execute_live_command(ctx, command).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve removeFromGroup mutation
pub async fn resolve_remove_from_group(
    ctx: &GqlContext,
    node_path: String,
    group: String,
) -> OperationResult {
    let command = GodotLiveCommand::RemoveFromGroup { node_path, group };

    match execute_live_command(ctx, command).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve saveScene mutation
pub async fn resolve_save_scene(ctx: &GqlContext) -> OperationResult {
    match execute_live_command(ctx, GodotLiveCommand::SaveScene).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

/// Resolve openScene mutation
pub async fn resolve_open_scene(ctx: &GqlContext, path: String) -> OperationResult {
    let command = GodotLiveCommand::OpenScene { scene_path: path };

    match execute_live_command(ctx, command).await {
        Ok(_) => OperationResult {
            success: true,
            message: None,
        },
        Err(e) => OperationResult {
            success: false,
            message: Some(e.to_string()),
        },
    }
}

// ======================
// Helper Functions
// ======================

/// Parse LiveScene from GetTree response
fn parse_live_scene_from_tree(value: &Value) -> Option<LiveScene> {
    // GetTree response structure varies, but typically:
    // { "scene_path": "...", "root": { "name": "...", "type": "...", "children": [...] } }

    let scene_path = value
        .get("scene_path")
        .and_then(|v| v.as_str())
        .map(String::from);

    let root = value
        .get("root")
        .or_else(|| Some(value)) // If no root key, treat whole value as root
        .and_then(|v| parse_live_node(v, ".".to_string()))?;

    Some(LiveScene {
        path: scene_path,
        root,
        selected_nodes: vec![],
    })
}

/// Find a specific node in the tree
fn find_node_in_tree(value: &Value, path: &str) -> Option<LiveNode> {
    // Start from root and search for the path
    let root = value.get("root").unwrap_or(value);
    find_node_recursive(root, path, ".".to_string())
}

fn find_node_recursive(value: &Value, target_path: &str, current_path: String) -> Option<LiveNode> {
    if current_path == target_path {
        return parse_live_node(value, current_path);
    }

    // Search children
    if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
        for child in children {
            if let Some(name) = child.get("name").and_then(|v| v.as_str()) {
                let child_path = if current_path == "." {
                    name.to_string()
                } else {
                    format!("{}/{}", current_path, name)
                };

                if let Some(node) = find_node_recursive(child, target_path, child_path) {
                    return Some(node);
                }
            }
        }
    }

    None
}

/// Parse a single LiveNode from JSON
fn parse_live_node(value: &Value, path: String) -> Option<LiveNode> {
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Root")
        .to_string();

    let node_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Node")
        .to_string();

    // Parse children recursively
    let children: Vec<LiveNode> = value
        .get("children")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|child| {
                    let child_name = child.get("name").and_then(|v| v.as_str())?;
                    let child_path = if path == "." {
                        child_name.to_string()
                    } else {
                        format!("{}/{}", path, child_name)
                    };
                    parse_live_node(child, child_path)
                })
                .collect()
        })
        .unwrap_or_default();

    // Parse properties
    let properties: Vec<Property> = value
        .get("properties")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| Property {
                    name: k.clone(),
                    value: v.to_string(),
                    property_type: None,
                })
                .collect()
        })
        .unwrap_or_default();

    Some(LiveNode {
        name,
        r#type: node_type,
        path,
        global_position: None,
        global_position_2d: None,
        properties,
        children,
        available_signals: vec![],
        connected_signals: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_live_node() {
        let json = serde_json::json!({
            "name": "Player",
            "type": "CharacterBody3D",
            "children": [
                {
                    "name": "Camera",
                    "type": "Camera3D"
                }
            ]
        });

        let node = parse_live_node(&json, ".".to_string()).unwrap();
        assert_eq!(node.name, "Player");
        assert_eq!(node.r#type, "CharacterBody3D");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "Camera");
    }

    #[test]
    fn test_find_node_in_tree() {
        let json = serde_json::json!({
            "root": {
                "name": "Root",
                "type": "Node",
                "children": [
                    {
                        "name": "Player",
                        "type": "CharacterBody3D"
                    }
                ]
            }
        });

        let node = find_node_in_tree(&json, "Player").unwrap();
        assert_eq!(node.name, "Player");
    }
}
