//! Resource manipulation tools - .tres reading and writing

use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};
use std::path::{Path, PathBuf};

use super::{
    AddExtResourceRequest, AddSubResourceRequest, AssignMaterialRequest, CreateMaterialRequest,
    CreateResourceRequest, GodotTools, ListResourcesRequest, ReadResourceRequest,
    SetResourcePropertyRequest,
};
use crate::godot::tres::GodotResource;
use crate::godot::tscn::GodotScene;

impl GodotTools {
    /// list_resources - List resources in the project
    pub async fn handle_list_resources(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ListResourcesRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();

        let base = self.get_base_path();
        let mut resources = Vec::new();

        // Recursively search for .tres files
        fn find_tres_files(dir: &Path) -> Vec<PathBuf> {
            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(find_tres_files(&path));
                    } else if path.extension().map(|e| e == "tres").unwrap_or(false) {
                        files.push(path);
                    }
                }
            }
            files
        }

        let tres_files = find_tres_files(base);

        for file_path in tres_files {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(resource) = GodotResource::parse(&content) {
                    // Filter check
                    if let Some(ref filter) = req.filter_type {
                        if !resource.resource_type.contains(filter) {
                            continue;
                        }
                    }

                    let rel_path = file_path.strip_prefix(base).unwrap_or(&file_path);
                    let res_path =
                        format!("res://{}", rel_path.to_string_lossy().replace('\\', "/"));

                    resources.push(serde_json::json!({
                        "path": res_path,
                        "type": resource.resource_type,
                        "ext_resource_count": resource.ext_resources.len(),
                        "sub_resource_count": resource.sub_resources.len(),
                    }));
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&serde_json::json!({ "resources": resources }))
                .unwrap_or_default(),
        )]))
    }

    /// read_resource - Read a resource file
    pub async fn handle_read_resource(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ReadResourceRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let resource = GodotResource::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&resource.to_json()).unwrap_or_default(),
        )]))
    }

    /// create_resource - Create a new resource file
    pub async fn handle_create_resource(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: CreateResourceRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                McpError::internal_error(format!("Failed to create directories: {}", e), None)
            })?;
        }

        let resource = GodotResource::new(&req.resource_type);
        let content = resource.to_tres();

        std::fs::write(&full_path, &content).map_err(|e| {
            McpError::internal_error(format!("Failed to write file: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created resource '{}' at {}",
            req.resource_type,
            full_path.display()
        ))]))
    }

    /// set_resource_property - Set a property on a resource
    pub async fn handle_set_resource_property(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: SetResourcePropertyRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let mut resource = GodotResource::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse: {}", e), None))?;

        resource.set_property(&req.property, &req.value);

        let new_content = resource.to_tres();
        std::fs::write(&full_path, &new_content).map_err(|e| {
            McpError::internal_error(format!("Failed to write file: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set property '{}' = {} on {}",
            req.property,
            req.value,
            full_path.display()
        ))]))
    }

    /// add_ext_resource - Add an external resource reference
    pub async fn handle_add_ext_resource(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AddExtResourceRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let mut resource = GodotResource::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse: {}", e), None))?;

        resource.add_ext_resource(&req.id, &req.resource_type, &req.resource_path);

        let new_content = resource.to_tres();
        std::fs::write(&full_path, &new_content).map_err(|e| {
            McpError::internal_error(format!("Failed to write file: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added ext_resource id='{}' type='{}' path='{}' to {}",
            req.id,
            req.resource_type,
            req.resource_path,
            full_path.display()
        ))]))
    }

    /// add_sub_resource - Add a sub-resource
    pub async fn handle_add_sub_resource(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AddSubResourceRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let mut resource = GodotResource::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse: {}", e), None))?;

        let sub = resource.add_sub_resource(&req.id, &req.resource_type);

        // Add properties if provided
        if let Some(props) = req.properties {
            for prop in props {
                sub.set_property(&prop.name, &prop.value);
            }
        }

        let new_content = resource.to_tres();
        std::fs::write(&full_path, &new_content).map_err(|e| {
            McpError::internal_error(format!("Failed to write file: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added sub_resource id='{}' type='{}' to {}",
            req.id,
            req.resource_type,
            full_path.display()
        ))]))
    }

    /// create_material - Create a StandardMaterial3D resource
    pub async fn handle_create_material(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: CreateMaterialRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                McpError::internal_error(format!("Failed to create directories: {}", e), None)
            })?;
        }

        let mut resource = GodotResource::new("StandardMaterial3D");

        // Set optional properties
        if let Some(name) = req.name {
            resource.set_property("resource_name", &format!("\"{}\"", name));
        }

        if let Some(color) = req.albedo_color {
            resource.set_property(
                "albedo_color",
                &format!("Color({}, {}, {}, {})", color[0], color[1], color[2], color[3]),
            );
        }

        if let Some(metallic) = req.metallic {
            resource.set_property("metallic", &metallic.to_string());
        }

        if let Some(roughness) = req.roughness {
            resource.set_property("roughness", &roughness.to_string());
        }

        let content = resource.to_tres();
        std::fs::write(&full_path, &content).map_err(|e| {
            McpError::internal_error(format!("Failed to write file: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created StandardMaterial3D at {}",
            full_path.display()
        ))]))
    }

    /// set_material_property - Set a property on a material
    pub async fn handle_set_material_property(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        // This is essentially the same as set_resource_property
        self.handle_set_resource_property(args).await
    }

    /// assign_material - Assign a material to a node in a scene
    pub async fn handle_assign_material(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AssignMaterialRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let scene_path = req.scene_path.strip_prefix("res://").unwrap_or(&req.scene_path);
        let full_scene_path = base.join(scene_path);

        let content = std::fs::read_to_string(&full_scene_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read scene: {}", e), None))?;

        let mut scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse scene: {}", e), None))?;

        // Add ext_resource for the material if not already present
        let material_id = format!("{}", scene.ext_resources.len() + 1);
        let mut found = false;
        for ext in &scene.ext_resources {
            if ext.path == req.material_path {
                found = true;
                break;
            }
        }

        if !found {
            scene.ext_resources.push(crate::godot::tscn::ExtResource {
                id: material_id.clone(),
                resource_type: "Material".to_string(),
                path: req.material_path.clone(),
            });
        }

        // Find the node and set the material property
        let node_path = if req.node_path == "." {
            String::new()
        } else {
            req.node_path.clone()
        };

        let mut node_found = false;
        for node in &mut scene.nodes {
            let matches = if node_path.is_empty() {
                node.parent.is_none()
            } else {
                // Build the node's full path
                let full_node_path = if let Some(ref parent) = node.parent {
                    if parent == "." {
                        node.name.clone()
                    } else {
                        format!("{}/{}", parent, node.name)
                    }
                } else {
                    node.name.clone()
                };
                full_node_path == node_path || node.name == node_path
            };

            if matches {
                let surface_idx = req.surface_index.unwrap_or(0);
                let prop_name = format!("surface_material_override/{}", surface_idx);
                node.properties
                    .insert(prop_name, format!("ExtResource(\"{}\")", material_id));
                node_found = true;
                break;
            }
        }

        if !node_found {
            return Err(McpError::internal_error(
                format!("Node '{}' not found in scene", req.node_path),
                None,
            ));
        }

        let new_content = scene.to_tscn();
        std::fs::write(&full_scene_path, &new_content).map_err(|e| {
            McpError::internal_error(format!("Failed to write scene: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Assigned material '{}' to node '{}' in {}",
            req.material_path,
            req.node_path,
            full_scene_path.display()
        ))]))
    }
}
