//! リソース操作ツール - .tres読み取り

use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};
use std::path::{Path, PathBuf};

use super::{GodotTools, ListResourcesRequest, ReadResourceRequest};
use crate::godot::tres::GodotResource;

impl GodotTools {
    /// list_resources - プロジェクト内のリソース一覧
    pub(super) async fn handle_list_resources(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: ListResourcesRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();

        let base = self.get_base_path();
        let mut resources = Vec::new();

        // 再帰的にtresファイルを探索
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
                    // フィルタチェック
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

    /// read_resource - リソースファイルを読み取り
    pub(super) async fn handle_read_resource(
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
}
