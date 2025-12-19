use serde::{Deserialize, Serialize};

/// Command sent to the Godot Editor plugin
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum GodotCommand {
    Ping,
    AddNode(AddNodeParams),
    RemoveNode(RemoveNodeParams),
    SetProperty(SetPropertyParams),
    GetTree,
    SaveScene,
    OpenScene(OpenSceneParams),
    ConnectSignal(SignalParams),
    DisconnectSignal(SignalParams),
    ListSignals(NodePathParams),
    GetProperties(NodePathParams),
    DuplicateNode(DuplicateNodeParams),
    RenameNode(RenameNodeParams),
    ReparentNode(ReparentNodeParams),
    // Animation
    CreateAnimation(CreateAnimationParams),
    AddAnimationTrack(AddAnimationTrackParams),
    AddAnimationKey(AddAnimationKeyParams),
    PlayAnimation(PlayAnimationParams),
    StopAnimation(StopAnimationParams),
    ListAnimations(StopAnimationParams), // Uses same struct for player path
    // Debug
    GetEditorLog(GetLogParams),
    ClearEditorLog,
    // Instantiate
    InstantiateScene(InstantiateSceneParams),
    // Reload
    ReloadPlugin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddNodeParams {
    pub parent: String,
    pub name: String,
    pub node_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveNodeParams {
    pub node_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetPropertyParams {
    pub node_path: String,
    pub property: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodePathParams {
    pub node_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalParams {
    pub source: String,
    pub signal: String,
    pub target: String,
    pub method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateNodeParams {
    pub node_path: String,
    pub new_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameNodeParams {
    pub node_path: String,
    pub new_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReparentNodeParams {
    pub node_path: String,
    pub new_parent: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAnimationParams {
    pub player: String,
    pub name: String,
    pub length: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAnimationTrackParams {
    pub player: String,
    pub animation: String,
    pub track_path: String,
    pub track_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAnimationKeyParams {
    pub player: String,
    pub animation: String,
    pub track: usize,
    pub time: f32,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayAnimationParams {
    pub player: String,
    pub animation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopAnimationParams {
    pub player: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLogParams {
    pub lines: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSceneParams {
    pub scene_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstantiateSceneParams {
    pub scene_path: String,
    pub parent: String,
    pub name: Option<String>,
    pub position: Option<Position3D>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
