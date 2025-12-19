//! .tscn File Parser
//!
//! Parse and generate Godot text scene format

use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TscnError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
}

/// Godot Scene
#[derive(Debug, Clone)]
pub struct GodotScene {
    /// Scene header information
    pub format: u32,
    pub uid: Option<String>,

    /// External resources
    pub ext_resources: Vec<ExtResource>,

    /// Sub-resources
    pub sub_resources: Vec<SubResource>,

    /// Node list
    pub nodes: Vec<SceneNode>,
}

/// External Resource Reference
#[derive(Debug, Clone)]
pub struct ExtResource {
    pub id: String,
    pub resource_type: String,
    pub path: String,
}

/// Sub-Resource
#[derive(Debug, Clone)]
pub struct SubResource {
    pub id: String,
    pub resource_type: String,
    pub properties: HashMap<String, String>,
}

/// Scene Node
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub name: String,
    pub node_type: String,
    pub parent: Option<String>,
    pub properties: HashMap<String, String>,
}

impl GodotScene {
    /// Create new scene
    pub fn new(root_name: &str, root_type: &str) -> Self {
        Self {
            format: 3,
            uid: None,
            ext_resources: Vec::new(),
            sub_resources: Vec::new(),
            nodes: vec![SceneNode {
                name: root_name.to_string(),
                node_type: root_type.to_string(),
                parent: None,
                properties: HashMap::new(),
            }],
        }
    }

    /// Parse .tscn file
    pub fn parse(content: &str) -> Result<Self, TscnError> {
        let mut scene = GodotScene {
            format: 3,
            uid: None,
            ext_resources: Vec::new(),
            sub_resources: Vec::new(),
            nodes: Vec::new(),
        };

        let mut current_section: Option<&str> = None;
        let mut current_node: Option<SceneNode> = None;
        let mut current_properties: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                // Save previous node
                if let Some(mut node) = current_node.take() {
                    node.properties = current_properties.clone();
                    scene.nodes.push(node);
                    current_properties.clear();
                }

                let section_content = &line[1..line.len() - 1];

                if section_content.starts_with("gd_scene") {
                    // Parse header
                    if let Some(format) = extract_attr(section_content, "format") {
                        scene.format = format.parse().unwrap_or(3);
                    }
                    if let Some(uid) = extract_attr(section_content, "uid") {
                        scene.uid = Some(uid.to_string());
                    }
                    current_section = Some("gd_scene");
                } else if section_content.starts_with("ext_resource") {
                    let res = parse_ext_resource(section_content)?;
                    scene.ext_resources.push(res);
                    current_section = Some("ext_resource");
                } else if section_content.starts_with("sub_resource") {
                    current_section = Some("sub_resource");
                } else if section_content.starts_with("node") {
                    let node = parse_node_header(section_content)?;
                    current_node = Some(node);
                    current_section = Some("node");
                }
            } else if current_section == Some("node") {
                // Property line
                if let Some((key, value)) = line.split_once(" = ") {
                    current_properties.insert(key.to_string(), value.to_string());
                }
            }
        }

        // Save last node
        if let Some(mut node) = current_node.take() {
            node.properties = current_properties;
            scene.nodes.push(node);
        }

        Ok(scene)
    }

    /// Convert to .tscn format
    pub fn to_tscn(&self) -> String {
        let mut output = String::new();

        // Header
        let load_steps = 1 + self.ext_resources.len() + self.sub_resources.len();
        output.push_str(&format!(
            "[gd_scene load_steps={} format={}",
            load_steps, self.format
        ));
        if let Some(ref uid) = self.uid {
            output.push_str(&format!(" uid=\"{}\"", uid));
        }
        output.push_str("]\n\n");

        // External resources
        for res in &self.ext_resources {
            output.push_str(&format!(
                "[ext_resource type=\"{}\" path=\"{}\" id=\"{}\"]\n",
                res.resource_type, res.path, res.id
            ));
        }
        if !self.ext_resources.is_empty() {
            output.push('\n');
        }

        // Nodes
        for node in &self.nodes {
            output.push_str(&format!(
                "[node name=\"{}\" type=\"{}\"",
                node.name, node.node_type
            ));
            if let Some(ref parent) = node.parent {
                output.push_str(&format!(" parent=\"{}\"", parent));
            }
            output.push_str("]\n");

            for (key, value) in &node.properties {
                output.push_str(&format!("{} = {}\n", key, value));
            }
            output.push('\n');
        }

        output
    }

    /// Add node
    pub fn add_node(&mut self, node: SceneNode) {
        self.nodes.push(node);
    }

    /// Add external resource
    pub fn add_ext_resource(&mut self, id: &str, resource_type: &str, path: &str) {
        self.ext_resources.push(ExtResource {
            id: id.to_string(),
            resource_type: resource_type.to_string(),
            path: path.to_string(),
        });
    }

    /// Set property
    pub fn set_property(
        &mut self,
        node_path: &str,
        property: &str,
        value: &str,
    ) -> Result<(), String> {
        let node = if node_path == "." {
            self.nodes.first_mut()
        } else {
            self.nodes.iter_mut().find(|n| {
                let full_path = n
                    .parent
                    .as_ref()
                    .map(|p| format!("{}/{}", p, n.name))
                    .unwrap_or_else(|| n.name.clone());
                full_path == node_path || n.name == node_path
            })
        };

        match node {
            Some(n) => {
                n.properties.insert(property.to_string(), value.to_string());
                Ok(())
            }
            None => Err(format!("Node not found: {}", node_path)),
        }
    }

    /// Remove node
    pub fn remove_node(&mut self, node_path: &str) -> Result<(), String> {
        if node_path == "." {
            return Err("Cannot remove root node".to_string());
        }

        let initial_len = self.nodes.len();

        // Search and remove node
        self.nodes.retain(|n| {
            let full_path = n
                .parent
                .as_ref()
                .map(|p| {
                    if p == "." {
                        n.name.clone()
                    } else {
                        format!("{}/{}", p, n.name)
                    }
                })
                .unwrap_or_else(|| n.name.clone());
            full_path != node_path && n.name != node_path
        });

        // Also remove children (where parent path starts with node_path)
        self.nodes.retain(|n| {
            if let Some(ref parent) = n.parent {
                !parent.starts_with(node_path) && parent != node_path
            } else {
                true
            }
        });

        if self.nodes.len() == initial_len {
            Err(format!("Node not found: {}", node_path))
        } else {
            Ok(())
        }
    }

    /// Format version
    pub fn format_version(&self) -> u32 {
        self.format
    }

    /// Load steps
    pub fn load_steps(&self) -> usize {
        1 + self.ext_resources.len() + self.sub_resources.len()
    }

    /// External resources list (for JSON)
    pub fn external_resources(&self) -> Vec<serde_json::Value> {
        self.ext_resources
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "type": r.resource_type,
                    "path": r.path,
                })
            })
            .collect()
    }
}

/// Extract attribute value
fn extract_attr<'a>(content: &'a str, attr: &str) -> Option<&'a str> {
    let pattern = format!("{}=", attr);
    content.find(&pattern).map(|start| {
        let rest = &content[start + pattern.len()..];
        if rest.starts_with('"') {
            let rest = &rest[1..];
            rest.find('"').map(|end| &rest[..end]).unwrap_or(rest)
        } else {
            rest.split_whitespace().next().unwrap_or("")
        }
    })
}

/// Parse external resource
fn parse_ext_resource(content: &str) -> Result<ExtResource, TscnError> {
    let resource_type = extract_attr(content, "type")
        .ok_or_else(|| TscnError::ParseError("Missing type in ext_resource".into()))?;
    let path = extract_attr(content, "path")
        .ok_or_else(|| TscnError::ParseError("Missing path in ext_resource".into()))?;
    let id = extract_attr(content, "id")
        .ok_or_else(|| TscnError::ParseError("Missing id in ext_resource".into()))?;

    Ok(ExtResource {
        id: id.to_string(),
        resource_type: resource_type.to_string(),
        path: path.to_string(),
    })
}

/// Parse node header
fn parse_node_header(content: &str) -> Result<SceneNode, TscnError> {
    let name = extract_attr(content, "name")
        .ok_or_else(|| TscnError::ParseError("Missing name in node".into()))?;
    let node_type = extract_attr(content, "type").unwrap_or("Node");
    let parent = extract_attr(content, "parent");

    Ok(SceneNode {
        name: name.to_string(),
        node_type: node_type.to_string(),
        parent: parent.map(|s| s.to_string()),
        properties: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_scene() {
        let scene = GodotScene::new("Player", "CharacterBody3D");
        let tscn = scene.to_tscn();

        assert!(tscn.contains("[gd_scene"));
        assert!(tscn.contains("CharacterBody3D"));
        assert!(tscn.contains("Player"));
    }

    #[test]
    fn test_parse_scene() {
        let content = r#"[gd_scene load_steps=1 format=3]

[node name="Root" type="Node3D"]

[node name="Player" type="CharacterBody3D" parent="."]
"#;
        let scene = GodotScene::parse(content).unwrap();

        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.nodes[0].name, "Root");
        assert_eq!(scene.nodes[1].name, "Player");
    }
}
