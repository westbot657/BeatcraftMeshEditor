use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeatcraftEditorInfo {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_file: Option<String>,

    #[serde(flatten)]
    catchall: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct BeatmapDifficultyCustomData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_file: Option<String>,
}
