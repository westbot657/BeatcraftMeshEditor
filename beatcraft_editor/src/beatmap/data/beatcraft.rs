use serde::{Deserialize, Serialize};

use crate::VERSION;

pub const APP_NAME: &str = "BeatcraftEditor";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct BeatcraftEditorInfoV2 {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_file: Option<String>,

    #[serde(flatten)]
    catchall: Option<serde_json::Map<String, serde_json::Value>>,
}

impl Default for BeatcraftEditorInfoV2 {
    fn default() -> Self {
        Self {
            version: VERSION.to_string(),
            data_file: None,

            catchall: None,
        }
    }
}
