use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InfoCustomDataV2 {
    #[serde(rename = "_contributors")]
    pub contributors: Vec<ContributorV2>,
    #[serde(rename = "_customEnvironment")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_environment: Option<String>,
    #[serde(rename = "_customEnvironmentHash")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_environment_hash: Option<String>,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContributorV2 {
    #[serde(rename = "_role")]
    pub role: String,
    #[serde(rename = "_name")]
    pub name: String,
    #[serde(rename = "_iconPath")]
    pub icon_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RGBCustomColorDataV2 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionModName {
    Chroma,

    #[serde(untagged)]
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementModName {
    Chroma,
    #[serde(rename = "Noodle Extensions")]
    Noodle,
    Vivify,
    //////,
    #[serde(rename = "Mapping Extensions")]
    MappingExtensions,

    #[serde(untagged)]
    Custom(String),
}

impl SuggestionModName {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Chroma => "Chroma",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl RequirementModName {
    pub fn display_name(&self) -> &str {
        match self {
            RequirementModName::Chroma => "Chroma",
            RequirementModName::Noodle => "Noodle Extensions",
            RequirementModName::Vivify => "Vivify",
            /////////////// => "////////",
            RequirementModName::MappingExtensions => "Mapping Extensions",
            RequirementModName::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifficultySetCustomDataV2 {
    #[serde(rename = "_characteristicLabel")]
    pub characteristic_label: String,
    #[serde(rename = "_characteristicIconImageFilename")]
    pub characteristic_icon_image_filename: String,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifficultyBeatmapCustomDataV2 {
    #[serde(rename = "_oneSaber")]
    pub one_saber: bool,
    #[serde(rename = "_showRotationNoteSpawnLines")]
    pub show_rotation_note_spawn_lines: bool,
    #[serde(rename = "_difficultyLabel")]
    pub difficulty_label: String,
    #[serde(rename = "_editorOffset")]
    pub editor_offset: f32,
    #[serde(rename = "_editorOldOffset")]
    pub editor_old_offset: f32,
    #[serde(rename = "_colorLeft")]
    pub color_left: RGBCustomColorDataV2,
    #[serde(rename = "_colorRight")]
    pub color_right: RGBCustomColorDataV2,
    #[serde(rename = "_envColorLeft")]
    pub env_color_left: RGBCustomColorDataV2,
    #[serde(rename = "_envColorRight")]
    pub env_color_right: RGBCustomColorDataV2,
    #[serde(rename = "_envColorLeftBoost")]
    pub env_color_left_boost: RGBCustomColorDataV2,
    #[serde(rename = "_envColorRightBoost")]
    pub env_color_right_boost: RGBCustomColorDataV2,
    #[serde(rename = "_obstacleColor")]
    pub obstacle_color: RGBCustomColorDataV2,
    #[serde(rename = "_warnings")]
    pub warnings: Vec<String>,
    #[serde(rename = "_information")]
    pub information: Vec<String>,
    #[serde(rename = "_suggestions")]
    pub suggestions: Vec<SuggestionModName>,
    #[serde(rename = "_requirements")]
    pub requirements: Vec<RequirementModName>,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

