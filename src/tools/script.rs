//! Script Related Tools - GDScript Creation, Attachment & Analysis

use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};

use super::{
    AddExportVarRequest, AddFunctionRequest, AnalyzeScriptRequest, AttachScriptRequest,
    CreateScriptRequest, FunctionParamInput, GodotTools, ReadScriptRequest,
};
use crate::godot::gdscript::{generate_template, ExportVar, Function, FunctionParam, GDScript};
use crate::godot::tscn::GodotScene;

impl GodotTools {
    /// create_script - Create script
    pub(super) async fn handle_create_script(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: CreateScriptRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let full_path = base.join(&req.path);

        let content = req
            .content
            .unwrap_or_else(|| generate_template(&req.extends));

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }
        std::fs::write(&full_path, &content)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created script: {}",
            req.path
        ))]))
    }

    /// attach_script - Attach script
    pub(super) async fn handle_attach_script(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AttachScriptRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let scene_full_path = base.join(&req.scene_path);

        let content = std::fs::read_to_string(&scene_full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read scene: {}", e), None))?;

        let mut scene = GodotScene::parse(&content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse scene: {}", e), None))?;

        // Add script as external resource
        let res_id = format!("{}_{}", req.node_path.replace("/", "_"), "script");
        scene.add_ext_resource(&res_id, "Script", &format!("res://{}", req.script_path));

        // Set script property on node
        scene
            .set_property(
                &req.node_path,
                "script",
                &format!("ExtResource(\"{}\")", res_id),
            )
            .map_err(|e| McpError::internal_error(e, None))?;

        std::fs::write(&scene_full_path, scene.to_tscn())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Attached script '{}' to node '{}'",
            req.script_path, req.node_path
        ))]))
    }

    /// read_script - Read and parse script
    pub(super) async fn handle_read_script(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ReadScriptRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let script = GDScript::parse(&content);

        let result = serde_json::json!({
            "extends": script.extends,
            "class_name": script.class_name,
            "signals": script.signals,
            "exports": script.exports.iter().map(|e| serde_json::json!({
                "name": e.name,
                "type": e.var_type,
                "default": e.default_value,
            })).collect::<Vec<_>>(),
            "variables": script.variables.iter().map(|v| serde_json::json!({
                "name": v.name,
                "type": v.var_type,
                "default": v.default_value,
            })).collect::<Vec<_>>(),
            "functions": script.functions.iter().map(|f| serde_json::json!({
                "name": f.name,
                "params": f.params.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "type": p.param_type,
                })).collect::<Vec<_>>(),
                "return_type": f.return_type,
            })).collect::<Vec<_>>(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// add_function - Add function
    pub(super) async fn handle_add_function(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AddFunctionRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let mut script = GDScript::parse(&content);

        let params: Vec<FunctionParam> = req
            .params
            .unwrap_or_default()
            .into_iter()
            .map(|p: FunctionParamInput| FunctionParam {
                name: p.name,
                param_type: p.param_type,
                default_value: None,
            })
            .collect();

        script.add_function(Function {
            name: req.name.clone(),
            params,
            return_type: req.return_type,
            body: req.body.unwrap_or_else(|| "pass".to_string()),
        });

        std::fs::write(&full_path, script.to_gdscript())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added function '{}' to {}",
            req.name, req.path
        ))]))
    }

    /// add_export_var - Add export variable
    pub(super) async fn handle_add_export_var(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AddExportVarRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let mut script = GDScript::parse(&content);

        script.add_export(ExportVar {
            name: req.name.clone(),
            var_type: req.var_type,
            default_value: req.default_value,
        });

        std::fs::write(&full_path, script.to_gdscript())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added @export var '{}' to {}",
            req.name, req.path
        ))]))
    }

    /// analyze_script - Analyze script
    pub(super) async fn handle_analyze_script(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: AnalyzeScriptRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let base = self.get_base_path();
        let path = req.path.strip_prefix("res://").unwrap_or(&req.path);
        let full_path = base.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let script = GDScript::parse(&content);

        let result = serde_json::json!({
            "path": req.path,
            "summary": {
                "extends": script.extends,
                "class_name": script.class_name,
                "signal_count": script.signals.len(),
                "export_count": script.exports.len(),
                "variable_count": script.variables.len(),
                "function_count": script.functions.len(),
            },
            "signals": script.signals,
            "functions": script.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
            "exports": script.exports.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
            "variables": script.variables.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }
}
