use serde::{Deserialize, Serialize};

use crate::MapVersion;

use super::easing::Easing;
use super::{ArcMidAnchorMode, Color, CutDirection, SpawnRotationExecutionTime};
use super::{bool_u8_serde, easing_as_i8};
use super::{is_value_f, is_value_i, is_value_u};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorNoteV4 {
    #[serde(rename = "b")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub beat: f32,
    #[serde(rename = "r")]
    #[serde(default, skip_serializing_if = "is_value_i::<0>")]
    pub rotation_lane: i32,
    #[serde(rename = "i")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub metadata_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorNoteDataV4 {
    #[serde(rename = "x")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_index: f32,
    #[serde(rename = "y")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_layer: f32,
    #[serde(rename = "c")]
    #[serde(default, skip_serializing_if = "Color::is_red")]
    pub color: Color,
    #[serde(rename = "d")]
    #[serde(default, skip_serializing_if = "CutDirection::is_default")]
    pub cut_direction: CutDirection,
    #[serde(rename = "a")]
    #[serde(default, skip_serializing_if = "is_value_i::<0>")]
    pub angle_offset: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BombNoteV4 {
    #[serde(rename = "b")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub beat: f32,
    #[serde(rename = "r")]
    #[serde(default, skip_serializing_if = "is_value_i::<0>")]
    pub rotation_lane: i32,
    #[serde(rename = "i")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub metadata_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BombNoteDataV4 {
    #[serde(rename = "x")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_index: f32,
    #[serde(rename = "y")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_layer: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleV4 {
    #[serde(rename = "b")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub beat: f32,
    #[serde(rename = "r")]
    #[serde(default, skip_serializing_if = "is_value_i::<0>")]
    pub rotation_lane: i32,
    #[serde(rename = "i")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub metadata_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleDataV4 {
    #[serde(rename = "d")]
    pub duration: f32,
    #[serde(rename = "x")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_index: f32,
    #[serde(rename = "y")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_layer: f32,
    #[serde(rename = "w")]
    pub width: f32,
    #[serde(rename = "h")]
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcV4 {
    #[serde(rename = "hb")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub head_beat: f32,
    #[serde(rename = "tb")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub tail_beat: f32,
    #[serde(rename = "hr")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub head_rotation_lane: f32,
    #[serde(rename = "tr")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub tail_rotation_lane: f32,
    #[serde(rename = "hi")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub head_note_metadata_index: u32,
    #[serde(rename = "ti")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub tail_note_metadata_index: u32,
    #[serde(rename = "ai")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub metadata_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcDataV4 {
    #[serde(rename = "m")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub head_ctrl_magnitude: f32,
    #[serde(rename = "tm")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub tail_ctrl_magnitude: f32,
    #[serde(rename = "a")]
    #[serde(default, skip_serializing_if = "ArcMidAnchorMode::is_default")]
    pub mid_anchor_mode: ArcMidAnchorMode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainV4 {
    #[serde(rename = "hb")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub head_beat: f32,
    #[serde(rename = "tb")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub tail_beat: f32,
    #[serde(rename = "hr")]
    #[serde(default, skip_serializing_if = "is_value_i::<0>")]
    pub head_rotation_lane: i32,
    #[serde(rename = "tr")]
    #[serde(default, skip_serializing_if = "is_value_i::<0>")]
    pub tail_rotation_lane: i32,
    #[serde(rename = "i")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub head_note_metadata_index: u32,
    #[serde(rename = "ci")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub metadata_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainDataV4 {
    #[serde(rename = "tx")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub tail_line_index: f32,
    #[serde(rename = "ty")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub tail_line_layer: f32,
    #[serde(rename = "c")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub slice_count: u8,
    #[serde(rename = "s")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub squish_factor: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NJSEventV4 {
    #[serde(rename = "b")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub beat: f32,
    #[serde(rename = "i")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub metadata_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NJSEventDataV4 {
    #[serde(rename = "p", with = "bool_u8_serde")]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub extend: bool,
    #[serde(rename = "e", with = "easing_as_i8")]
    #[serde(default, skip_serializing_if = "Easing::is_default")]
    pub easing: Easing,
    #[serde(rename = "d")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub njs_diff: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventV4 {
    #[serde(rename = "b")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub beat: f32,
    #[serde(rename = "i")]
    pub metadata_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventDataV4 {
    #[serde(rename = "t")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "r")]
    pub magnitude: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapFileV4 {
    pub version: MapVersion,
    pub color_notes: Vec<ColorNoteV4>,
    pub color_notes_data: Vec<ColorNoteDataV4>,
    pub bomb_notes: Vec<BombNoteV4>,
    pub bomb_notes_data: Vec<BombNoteDataV4>,
    pub obstacles: Vec<ObstacleV4>,
    pub obstacles_data: Vec<ObstacleDataV4>,
    pub arcs: Vec<ArcV4>,
    pub arcs_data: Vec<ArcDataV4>,
    pub chains: Vec<ChainV4>,
    pub chains_data: Vec<ChainDataV4>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub njs_events: Option<Vec<NJSEventV4>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub njs_event_data: Option<Vec<NJSEventDataV4>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_rotations: Option<Vec<SpawnRotationEventV4>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_rotations_data: Option<Vec<SpawnRotationEventDataV4>>,
}
