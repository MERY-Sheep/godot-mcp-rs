//! Live manipulation tools - Direct communication with Godot Editor plugin

use super::{
    GodotTools, LiveAddAnimationKeyRequest, LiveAddAnimationTrackRequest, LiveAddNodeRequest,
    LiveAddToGroupRequest, LiveClearEditorLogRequest, LiveConnectSignalRequest,
    LiveCreateAnimationRequest, LiveDisconnectSignalRequest, LiveGetEditorLogRequest,
    LiveGetGroupNodesRequest, LiveGetTreeRequest, LiveInstantiateSceneRequest,
    LiveListAnimationsRequest, LiveListGroupsRequest, LiveListSignalsRequest, LiveOpenSceneRequest,
    LivePingRequest, LivePlayAnimationRequest, LiveReloadPluginRequest, LiveRemoveFromGroupRequest,
    LiveRemoveNodeRequest, LiveSaveSceneRequest, LiveSetPropertyRequest, LiveStopAnimationRequest,
};
use crate::godot::commands::{
    AddAnimationKeyParams, AddAnimationTrackParams, AddNodeParams, CreateAnimationParams,
    GetLogParams, GodotCommand, GroupNameParams, GroupParams, InstantiateSceneParams,
    NodePathParams, OpenSceneParams, PlayAnimationParams, Position3D, RemoveNodeParams,
    SetPropertyParams, SignalParams, StopAnimationParams,
};
use rmcp::{model::CallToolResult, model::Content, ErrorData as McpError};

impl GodotTools {
    /// Helper to run a live command to the Godot plugin
    ///
    /// Tries WebSocket first (port 6061), falls back to HTTP (port 6060) on failure.
    async fn execute_live(
        &self,
        port: Option<u16>,
        command: GodotCommand,
    ) -> Result<CallToolResult, McpError> {
        let base_port = port.unwrap_or(6060);
        let ws_port = port.unwrap_or(6061); // WebSocket uses 6061 by default

        // Try WebSocket first
        match self.execute_live_ws(ws_port, &command).await {
            Ok(result) => {
                tracing::debug!("WebSocket request succeeded");
                return Ok(result);
            }
            Err(_ws_err) => {
                tracing::debug!("WebSocket failed, falling back to HTTP");
                // Fall through to HTTP
            }
        }

        // Fallback to HTTP
        self.execute_live_http(base_port, &command).await
    }

    /// Execute command via WebSocket
    async fn execute_live_ws(
        &self,
        port: u16,
        command: &GodotCommand,
    ) -> Result<CallToolResult, McpError> {
        use crate::ws::WsClient;

        let client = WsClient::new(port);
        let response = client
            .send_command(command)
            .await
            .map_err(|e| McpError::internal_error(format!("WebSocket error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    /// Execute command via HTTP (legacy fallback)
    async fn execute_live_http(
        &self,
        port: u16,
        command: &GodotCommand,
    ) -> Result<CallToolResult, McpError> {
        let url = format!("http://localhost:{}", port);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(command)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to connect to Godot plugin: {}", e), None)
            })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if status.is_success() {
            Ok(CallToolResult::success(vec![Content::text(text)]))
        } else {
            Err(McpError::internal_error(
                format!("Godot plugin error ({}): {}", status, text),
                None,
            ))
        }
    }

    pub async fn handle_live_ping(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LivePingRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();
        self.execute_live(req.port, GodotCommand::Ping).await
    }

    pub async fn handle_live_add_node(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveAddNodeRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::AddNode(AddNodeParams {
                parent: req.parent,
                name: req.name,
                node_type: req.node_type,
            }),
        )
        .await
    }

    pub async fn handle_live_remove_node(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveRemoveNodeRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::RemoveNode(RemoveNodeParams {
                node_path: req.node_path,
            }),
        )
        .await
    }

    pub async fn handle_live_set_property(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveSetPropertyRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let value =
            serde_json::from_str(&req.value).unwrap_or(serde_json::Value::String(req.value));
        self.execute_live(
            req.port,
            GodotCommand::SetProperty(SetPropertyParams {
                node_path: req.node_path,
                property: req.property,
                value,
            }),
        )
        .await
    }

    pub async fn handle_live_get_tree(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveGetTreeRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();
        self.execute_live(req.port, GodotCommand::GetTree).await
    }

    pub async fn handle_live_save_scene(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveSaveSceneRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();
        self.execute_live(req.port, GodotCommand::SaveScene).await
    }

    pub async fn handle_live_open_scene(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveOpenSceneRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::OpenScene(OpenSceneParams {
                scene_path: req.scene_path,
            }),
        )
        .await
    }

    pub async fn handle_live_connect_signal(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveConnectSignalRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::ConnectSignal(SignalParams {
                source: req.source,
                signal: req.signal,
                target: req.target,
                method: req.method,
            }),
        )
        .await
    }

    pub async fn handle_live_disconnect_signal(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveDisconnectSignalRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::DisconnectSignal(SignalParams {
                source: req.source,
                signal: req.signal,
                target: req.target,
                method: req.method,
            }),
        )
        .await
    }

    pub async fn handle_live_list_signals(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveListSignalsRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::ListSignals(NodePathParams {
                node_path: req.node_path,
            }),
        )
        .await
    }

    pub async fn handle_live_create_animation(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveCreateAnimationRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::CreateAnimation(CreateAnimationParams {
                player: req.player,
                name: req.name,
                length: req.length,
            }),
        )
        .await
    }

    pub async fn handle_live_add_animation_track(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveAddAnimationTrackRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::AddAnimationTrack(AddAnimationTrackParams {
                player: req.player,
                animation: req.animation,
                track_path: req.track_path,
                track_type: req.track_type.unwrap_or_else(|| "value".to_string()),
            }),
        )
        .await
    }

    pub async fn handle_live_add_animation_key(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveAddAnimationKeyRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let value =
            serde_json::from_str(&req.value).unwrap_or(serde_json::Value::String(req.value));
        self.execute_live(
            req.port,
            GodotCommand::AddAnimationKey(AddAnimationKeyParams {
                player: req.player,
                animation: req.animation,
                track: req.track as usize,
                time: req.time,
                value,
            }),
        )
        .await
    }

    pub async fn handle_live_play_animation(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LivePlayAnimationRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::PlayAnimation(PlayAnimationParams {
                player: req.player,
                animation: req.animation,
            }),
        )
        .await
    }

    pub async fn handle_live_stop_animation(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveStopAnimationRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::StopAnimation(StopAnimationParams { player: req.player }),
        )
        .await
    }

    pub async fn handle_live_list_animations(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveListAnimationsRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::ListAnimations(StopAnimationParams { player: req.player }),
        )
        .await
    }

    pub async fn handle_live_get_editor_log(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveGetEditorLogRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();
        self.execute_live(
            req.port,
            GodotCommand::GetEditorLog(GetLogParams {
                lines: req.lines.unwrap_or(50),
            }),
        )
        .await
    }

    pub async fn handle_live_clear_editor_log(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveClearEditorLogRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();
        self.execute_live(req.port, GodotCommand::ClearEditorLog)
            .await
    }

    pub async fn handle_live_reload_plugin(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveReloadPluginRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .unwrap_or_default();
        self.execute_live(req.port, GodotCommand::ReloadPlugin)
            .await
    }

    // --- Group Management ---

    pub async fn handle_live_add_to_group(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveAddToGroupRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::AddToGroup(GroupParams {
                node_path: req.node_path,
                group: req.group,
            }),
        )
        .await
    }

    pub async fn handle_live_remove_from_group(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveRemoveFromGroupRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::RemoveFromGroup(GroupParams {
                node_path: req.node_path,
                group: req.group,
            }),
        )
        .await
    }

    pub async fn handle_live_list_groups(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveListGroupsRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::ListGroups(NodePathParams {
                node_path: req.node_path,
            }),
        )
        .await
    }

    pub async fn handle_live_get_group_nodes(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveGetGroupNodesRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::GetGroupNodes(GroupNameParams { group: req.group }),
        )
        .await
    }

    pub async fn handle_live_instantiate_scene(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let req: LiveInstantiateSceneRequest =
            serde_json::from_value(serde_json::Value::Object(args.unwrap_or_default()))
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.execute_live(
            req.port,
            GodotCommand::InstantiateScene(InstantiateSceneParams {
                scene_path: req.scene_path,
                parent: req.parent,
                name: req.name,
                position: if req.x.is_some() || req.y.is_some() || req.z.is_some() {
                    Some(Position3D {
                        x: req.x.unwrap_or(0.0),
                        y: req.y.unwrap_or(0.0),
                        z: req.z.unwrap_or(0.0),
                    })
                } else {
                    None
                },
            }),
        )
        .await
    }
}
