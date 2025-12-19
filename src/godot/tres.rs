//! .tres resource file parser
//!
//! Parsing Godot resource files (.tres)

use std::collections::HashMap;

/// Godot Resource (.tres)
#[derive(Debug, Clone)]
pub struct GodotResource {
    /// Resource type (e.g., "Resource", "PackedScene")
    pub resource_type: String,
    /// load_steps
    pub load_steps: Option<i32>,
    /// format version
    pub format: Option<i32>,
    /// External resource referral
    pub ext_resources: Vec<ExtResourceRef>,
    /// Sub-resources
    pub sub_resources: Vec<SubResourceDef>,
    /// Main resource properties
    pub properties: HashMap<String, String>,
}

/// External resource reference
#[derive(Debug, Clone)]
pub struct ExtResourceRef {
    pub id: String,
    pub resource_type: String,
    pub path: String,
}

/// Sub-resource definition
#[derive(Debug, Clone)]
pub struct SubResourceDef {
    pub id: String,
    pub resource_type: String,
    pub properties: HashMap<String, String>,
}

impl GodotResource {
    /// Parse a .tres file
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut resource = GodotResource {
            resource_type: String::new(),
            load_steps: None,
            format: None,
            ext_resources: Vec::new(),
            sub_resources: Vec::new(),
            properties: HashMap::new(),
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_section: Option<String> = None;
        let mut current_props: HashMap<String, String> = HashMap::new();
        let mut current_sub_id = String::new();
        let mut current_sub_type = String::new();

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                i += 1;
                continue;
            }

            // [gd_resource ...] header
            if line.starts_with("[gd_resource") {
                if let Some(rt) = extract_attr(line, "type") {
                    resource.resource_type = rt;
                }
                if let Some(ls) = extract_attr(line, "load_steps") {
                    resource.load_steps = ls.parse().ok();
                }
                if let Some(fmt) = extract_attr(line, "format") {
                    resource.format = fmt.parse().ok();
                }
                current_section = Some("header".to_string());
            }
            // [ext_resource ...]
            else if line.starts_with("[ext_resource") {
                if let (Some(id), Some(rt), Some(path)) = (
                    extract_attr(line, "id"),
                    extract_attr(line, "type"),
                    extract_attr(line, "path"),
                ) {
                    resource.ext_resources.push(ExtResourceRef {
                        id,
                        resource_type: rt,
                        path,
                    });
                }
            }
            // [sub_resource ...]
            else if line.starts_with("[sub_resource") {
                // Save previous sub-resource
                if !current_sub_id.is_empty() {
                    resource.sub_resources.push(SubResourceDef {
                        id: current_sub_id.clone(),
                        resource_type: current_sub_type.clone(),
                        properties: current_props.clone(),
                    });
                    current_props.clear();
                }

                current_sub_id = extract_attr(line, "id").unwrap_or_default();
                current_sub_type = extract_attr(line, "type").unwrap_or_default();
                current_section = Some("sub_resource".to_string());
            }
            // [resource]
            else if line.starts_with("[resource]") {
                // Save previous sub-resource
                if !current_sub_id.is_empty() {
                    resource.sub_resources.push(SubResourceDef {
                        id: current_sub_id.clone(),
                        resource_type: current_sub_type.clone(),
                        properties: current_props.clone(),
                    });
                    current_props.clear();
                    current_sub_id.clear();
                }
                current_section = Some("resource".to_string());
            }
            // Property line
            else if let Some(eq_pos) = line.find(" = ") {
                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 3..].trim().to_string();

                match current_section.as_deref() {
                    Some("resource") => {
                        resource.properties.insert(key, value);
                    }
                    Some("sub_resource") => {
                        current_props.insert(key, value);
                    }
                    _ => {}
                }
            }

            i += 1;
        }

        // Save the last sub-resource
        if !current_sub_id.is_empty() {
            resource.sub_resources.push(SubResourceDef {
                id: current_sub_id,
                resource_type: current_sub_type,
                properties: current_props,
            });
        }

        if resource.resource_type.is_empty() {
            return Err("Failed to parse resource: missing type".to_string());
        }

        Ok(resource)
    }

    /// Convert resource to JSON format
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": self.resource_type,
            "load_steps": self.load_steps,
            "format": self.format,
            "ext_resources": self.ext_resources.iter().map(|r| serde_json::json!({
                "id": r.id,
                "type": r.resource_type,
                "path": r.path,
            })).collect::<Vec<_>>(),
            "sub_resources": self.sub_resources.iter().map(|s| serde_json::json!({
                "id": s.id,
                "type": s.resource_type,
                "properties": s.properties,
            })).collect::<Vec<_>>(),
            "properties": self.properties,
        })
    }
}

/// Extract attribute value (e.g., get "Resource" from type="Resource")
fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=", attr);
    if let Some(start) = line.find(&pattern) {
        let rest = &line[start + pattern.len()..];
        if rest.starts_with('"') {
            let rest = &rest[1..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        } else {
            // Unquoted value until space or closing bracket
            let end = rest
                .find(|c: char| c == ' ' || c == ']')
                .unwrap_or(rest.len());
            return Some(rest[..end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tres() {
        let content = r#"[gd_resource type="Resource" format=3]

[resource]
name = "TestResource"
value = 42
"#;
        let res = GodotResource::parse(content).unwrap();
        assert_eq!(res.resource_type, "Resource");
        assert_eq!(res.format, Some(3));
        assert_eq!(
            res.properties.get("name"),
            Some(&"\"TestResource\"".to_string())
        );
    }
}
