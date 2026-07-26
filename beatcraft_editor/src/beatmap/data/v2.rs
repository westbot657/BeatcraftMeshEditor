use serde::{Deserialize, Serialize};

use super::{ArcV2, BPMEventV2, convert_u8, BeatmapDataError, BombNoteV2, ColorNoteV2, InfoVersion, MapCharacteristic, MapVersion, ObstacleV2, RGBAColor, Sentinel, SpawnRotationEventV2};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum V2Note {
    Note(ColorNoteV2),
    Bomb(BombNoteV2),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum V2Event {
    SpawnRotation(SpawnRotationEventV2),
    Light(LightEventV2),
    ColorBoost(ColorBoostV2),
    Ring(RingLightEventV2),
    RotatingLights(SpiningLaserEventV2),
    Hydraulics(HydraulicsEventV2),
    Gaga(GagaEventV2),
    BPM(BPMEventV2),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum LightEventTypeV2 {
    BackLasers   = 0,
    RingLights   = 1,
    LeftLasers   = 2,
    RightLasers  = 3,
    CenterLasers = 4,

    LeftExtra    = 6,
    RightExtra   = 7,

    BillieLeft   = 10,
    BillieRight  = 11,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum RingLightEventTypeV2 {
    Spin = 8,
    Zoom = 9,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum SpinningLaserSideV2 {
    Left  = 12,
    Right = 13,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum HydraulicsTypeV2 {
    Lower = 16,
    Raise = 17,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum GagaSideV2 {
    Left  = 18,
    Right = 19,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum LightEventValueV2 {
    Off                 = 0,
    StaticSecondary     = 1,
    FlashSecondary      = 2,
    FadeSecondary       = 3,
    TransitionSecondary = 4,
    StaticPrimary       = 5,
    FlashPrimary        = 6,
    FadePrimary         = 7,
    TransitionPrimary   = 8,
    StaticWhite         = 9,
    FlashWhite          = 10,
    FadeWhite           = 11,
    TransitionWhite     = 12,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightEventV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_type")]
    pub typ: LightEventTypeV2,
    #[serde(rename = "_value")]
    pub value: LightEventValueV2,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RingLightEventV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_type")]
    pub typ: RingLightEventTypeV2,
    #[serde(rename = "_value")]
    pub value: u32,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpiningLaserEventV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_type")]
    pub typ: SpinningLaserSideV2,
    #[serde(rename = "_value")]
    pub value: u32,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HydraulicsEventV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_type")]
    pub typ: HydraulicsTypeV2,
    #[serde(rename = "_value")]
    pub value: u32,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GagaEventV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_type")]
    pub typ: GagaSideV2,
    #[serde(rename = "_value")]
    pub value: u32,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorBoostV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_type")]
    typ: Sentinel<5>,
    #[serde(rename = "_value")]
    value: u32,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatmapFileV2 {
    #[serde(rename = "Stats")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<serde_json::Value>,
    #[serde(rename = "_version")]
    pub version: MapVersion,
    #[serde(rename = "_notes")]
    pub notes: Vec<V2Note>,
    #[serde(rename = "_obstacles")]
    pub obstacles: Vec<ObstacleV2>,
    #[serde(rename = "_sliders")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arcs: Vec<ArcV2>,
    #[serde(rename = "_events")]
    pub events: Vec<V2Event>,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,

    // private as I don't care to implement these
    // but it still needs to be preserved from loading
    #[serde(rename = "_waypoints")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    waypoints: Option<serde_json::Value>,
    #[serde(rename = "_specialEventsKeywordFilters")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    special_events: Option<serde_json::Value>,
}

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
    pub environment_360: String,
    #[serde(rename = "_environmentNames")]
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub environment_names: Vec<String>,
    #[serde(rename = "_colorSchemes")]
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub color_schemes: Vec<ColorSchemeV2>,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
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
    #[serde(rename = "_beatmapCharacteristicName")]
    pub beatmap_characteristic_name: MapCharacteristic,
    #[serde(rename = "_difficultyBeatmaps")]
    pub difficulty_beatmaps: Vec<DifficultyBeatmapV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DifficultyBeatmapV2 {
    #[serde(rename = "_difficulty")]
    pub difficulty: String,
    #[serde(rename = "_difficultyRank")]
    pub difficulty_rank: u8,
    #[serde(rename = "_beatmapFilename")]
    pub beatmap_filename: String,
    #[serde(rename = "_noteJumpMovementSpeed")]
    pub note_jump_movement_speed: f32,
    #[serde(rename = "_noteJumpStartBeatOffset")]
    pub note_jump_start_beat_offset: f32,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}


// implementations


convert_u8! { LightEventTypeV2: 0..=4 | 6 | 7 | 10 | 11 }
convert_u8! { LightEventValueV2: 0..=12 }
convert_u8! { RingLightEventTypeV2: 8 | 9 }
convert_u8! { SpinningLaserSideV2: 12 | 13 }
convert_u8! { HydraulicsTypeV2: 16 | 17 }
convert_u8! { GagaSideV2: 18 | 19 }

