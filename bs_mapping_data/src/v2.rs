use num_traits::Zero;
use serde::{Deserialize, Serialize};

use super::convert_u8;
use super::{
    ArcMidAnchorMode, Color, CutDirection, Sentinel, SpawnRotationAngle, SpawnRotationExecutionTime,
};
use super::{default_f, is_value_f};
use crate::MapVersion;
use crate::v2_v3::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorNoteV2 {
    #[serde(rename = "_time")]
    pub time: f32,
    #[serde(rename = "_lineIndex")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub line_index: f32,
    #[serde(rename = "_lineLayer")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub line_layer: f32,
    #[serde(rename = "_type")]
    pub typ: Color,
    #[serde(rename = "_cutDirection")]
    pub cut_direction: CutDirection,
    #[cfg(feature = "custom_data")]
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BombNoteV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_lineIndex")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub line_index: f32,
    #[serde(rename = "_lineLayer")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub line_layer: f32,
    _type: Sentinel<3>,
    #[serde(rename = "_cutDirection")]
    pub cut_direction: CutDirection,
    #[cfg(feature = "custom_data")]
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum ObstacleV2Type {
    FullHeight = 0,
    Crouch = 1,
    Free = 2,
}
convert_u8! { ObstacleV2Type: 0..=2 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleV2 {
    #[serde(rename = "_type")]
    pub typ: ObstacleV2Type,
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_duration")]
    pub duration: f32,
    #[serde(rename = "_lineIndex")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub line_index: f32,
    #[serde(rename = "_lineLayer")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub line_layer: f32,
    #[serde(rename = "_width")]
    pub width: f32,
    #[serde(rename = "_height")]
    #[serde(default = "default_f::<{ 5f32.to_bits() }>")]
    #[serde(skip_serializing_if = "is_value_f::<{ 5f32.to_bits() }>")]
    pub height: f32,
    #[cfg(feature = "custom_data")]
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcV2 {
    #[serde(rename = "_colorType")]
    pub color: Color,
    #[serde(rename = "_headTime")]
    pub head_beat: f32,
    #[serde(rename = "_headLineIndex")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub head_line_index: f32,
    #[serde(rename = "_headLineLayer")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub head_line_layer: f32,
    #[serde(rename = "_headCutDirection")]
    pub head_cut_direction: CutDirection,
    #[serde(rename = "_headControlPointLengthMultiplier")]
    pub head_ctrl_magnitude: f32,
    #[serde(rename = "_tailTime")]
    pub tail_beat: f32,
    #[serde(rename = "_tailLineIndex")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub tail_line_index: f32,
    #[serde(rename = "_tailLineLayer")]
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub tail_line_layer: f32,
    #[serde(rename = "_tailCutDirection")]
    pub tail_cut_direction: CutDirection,
    #[serde(rename = "_tailControlPointLengthMultiplier")]
    pub tail_ctrl_magnitude: f32,
    #[serde(rename = "_sliderMidAnchorMode")]
    pub mid_anchor_mode: ArcMidAnchorMode,
    #[cfg(feature = "custom_data")]
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_type")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "_value")]
    pub rotation_angle: SpawnRotationAngle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BPMEventV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    _type: Sentinel<100>,
    #[serde(rename = "_value")]
    value: Sentinel<0>,
    #[serde(rename = "_floatValue")]
    float_value: f32,
}

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
    RotatingLights(SpinningLaserEventV2),
    Hydraulics(HydraulicsEventV2),
    Gaga(GagaEventV2),
    BPM(BPMEventV2),
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
pub struct SpinningLaserEventV2 {
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
    value: ColorBoostValueV2,
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
    #[serde(rename = "_BPMChanges")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bpm_changes: Option<serde_json::Value>,
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

    #[serde(rename = "_bookmarks")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bookmarks: Option<serde_json::Value>,

    // private as I don't care to implement these
    // but it still needs to be preserved from loading
    #[serde(rename = "_waypoints")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    waypoints: Option<serde_json::Value>,
    #[serde(rename = "_specialEventsKeywordFilters")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    special_events: Option<serde_json::Value>,
}
