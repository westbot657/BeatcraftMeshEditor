use serde::{Deserialize, Serialize};

use super::{BpmRegion, InfoVersion, MapCharacteristic, MapDifficulty, MapVersion, RGBAColor};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoV2 {
    #[serde(rename = "_version")]
    pub version: InfoVersion,
    #[serde(rename = "_songName")]
    pub song_name: String,
    #[serde(rename = "_songSubName")]
    pub song_sub_name: String,
    #[serde(rename = "_songAuthorName")]
    pub song_author_name: String,
    #[serde(rename = "_levelAuthorName")]
    pub level_author_name: String,
    #[serde(rename = "_beatsPerMinute")]
    pub bpm: f32,
    #[serde(rename = "_songTimeOffset")]
    pub song_time_offset: f32,
    #[serde(rename = "_shuffle")]
    pub shuffle: f32,
    #[serde(rename = "_shufflePeriod")]
    pub shuffle_period: f32,
    #[serde(rename = "_previewStartTime")]
    pub preview_start_time: f32,
    #[serde(rename = "_previewDuration")]
    pub preview_duration: f32,
    #[serde(rename = "_songFilename")]
    pub song_filename: String,
    #[serde(rename = "_coverImageFilename")]
    pub cover_image_filename: String,
    #[serde(rename = "_environmentName")]
    pub environment: String,
    #[serde(rename = "_allDirectionsEnvironmentName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_360: Option<String>,
    #[serde(rename = "_environmentNames")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environment_names: Vec<String>,
    #[serde(rename = "_colorSchemes")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub color_schemes: Vec<ColorSchemeV2>,
    #[cfg(feature = "custom_data")]
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<crate::custom_info_v2::InfoCustomDataV2>,
    #[serde(rename = "_difficultyBeatmapSets")]
    pub difficulty_beatmap_sets: Vec<CharacteristicSetV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorSchemeV2 {
    #[serde(rename = "useOverride")]
    pub use_override: bool,
    #[serde(rename = "colorScheme")]
    pub color_scheme: ColorSchemeInnerV2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorSchemeInnerV2 {
    #[serde(rename = "colorSchemeId")]
    pub color_scheme_id: String,
    #[serde(rename = "saberAColor")]
    pub saber_a_color: RGBAColor,
    #[serde(rename = "saberBColor")]
    pub saber_b_color: RGBAColor,
    #[serde(rename = "environmentColor0")]
    pub environment_color_0: RGBAColor,
    #[serde(rename = "environmentColor1")]
    pub environment_color_1: RGBAColor,
    #[serde(rename = "obstacleColor")]
    pub obstacle_color: RGBAColor,
    #[serde(rename = "environmentColor0Boost")]
    pub environment_color_0_boost: RGBAColor,
    #[serde(rename = "environmentColor1Boost")]
    pub environment_color_1_boost: RGBAColor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacteristicSetV2 {
    #[cfg(feature = "custom_data")]
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<crate::custom_info_v2::DifficultySetCustomDataV2>,
    #[serde(rename = "_beatmapCharacteristicName")]
    pub beatmap_characteristic_name: MapCharacteristic,
    #[serde(rename = "_difficultyBeatmaps")]
    pub difficulty_beatmaps: Vec<DifficultyBeatmapV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DifficultyBeatmapV2 {
    #[serde(rename = "_difficulty")]
    pub difficulty: MapDifficulty,
    #[serde(rename = "_difficultyRank")]
    pub difficulty_rank: u8,
    #[serde(rename = "_beatmapFilename")]
    pub beatmap_filename: String,
    #[serde(rename = "_noteJumpMovementSpeed")]
    pub note_jump_movement_speed: f32,
    #[serde(rename = "_noteJumpStartBeatOffset")]
    pub note_jump_start_beat_offset: f32,
    #[cfg(feature = "custom_data")]
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<crate::custom_info_v2::DifficultyBeatmapCustomDataV2>,

    #[serde(rename = "_beatmapColorSchemeIdx")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beatmap_color_scheme_index: Option<u32>,
    #[serde(rename = "_environmentNameIdx")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_name_index: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioDataFileV2 {
    #[serde(rename = "_version")]
    pub version: MapVersion,
    #[serde(rename = "_songSampleCount")]
    pub sample_count: usize,
    #[serde(rename = "_songFrequency")]
    pub frequency: u32,
    #[serde(rename = "_regions")]
    pub bpm_regions: Vec<BpmRegionV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BpmRegionV2 {
    #[serde(rename = "_startSampleIndex")]
    pub start_index: usize,
    #[serde(rename = "_endSampleIndex")]
    pub end_index: usize,
    #[serde(rename = "_startBeat")]
    pub start_beat: f32,
    #[serde(rename = "_endBeat")]
    pub end_beat: f32,
}

impl From<&BpmRegionV2> for BpmRegion {
    fn from(value: &BpmRegionV2) -> Self {
        Self {
            start_sample: value.start_index,
            end_sample: value.end_index,
            start_beat: value.start_beat,
            end_beat: value.end_beat,
        }
    }
}
