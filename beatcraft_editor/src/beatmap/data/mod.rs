
use std::fmt::{Debug, Display};

use glam::{Quat, Vec4};
use num_traits::{Num, Zero};
use serde::{Deserialize, Serialize};

use crate::easing::Easing;

pub mod v2;
pub mod v3;
pub mod v4;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VersionClass {
    V2,
    V3,
    V4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapVersion {
    #[serde(rename = "2.0.0")]
    V2_0_0,
    #[serde(rename = "2.2.0")]
    V2_2_0,
    #[serde(rename = "2.4.0")]
    V2_4_0,
    #[serde(rename = "2.5.0")]
    V2_5_0,
    #[serde(rename = "2.6.0")]
    V2_6_0,

    #[serde(rename = "3.0.0")]
    V3_0_0,
    #[serde(rename = "3.1.0")]
    V3_1_0,
    #[serde(rename = "3.2.0")]
    V3_2_0,
    #[serde(rename = "3.3.0")]
    V3_3_0,

    #[serde(rename = "4.0.0")]
    V4_0_0,
    #[serde(rename = "4.1.0")]
    V4_1_0,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InfoVersion {
    #[serde(rename = "2.0.0")]
    V2_0_0,
    #[serde(rename = "2.1.0")]
    V2_1_0,

    #[serde(rename = "4.0.0")]
    V4_0_0,
    #[serde(rename = "4.0.1")]
    V4_0_1,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapCharacteristic {
    Standard,
    NoArrows,
    OneSaber,
    #[serde(rename = "360Degree")]
    Degree360,
    #[serde(rename = "90Degree")]
    Degree90,
    Legacy,

    Lightshow,
    Lawless,

    #[serde(untagged)]
    Unknown(String)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
    ExpertPlus,

    #[serde(untagged)]
    Unknown(String),
}

// Color notes

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum Color {
    #[default]
    Red = 0,
    Blue = 1,
}
impl Color {
    pub fn is_red(&self) -> bool {
        *self == Self::Red
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RGBAColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum CutDirection {
    #[default]
    Up        = 0,
    Down      = 1,
    Left      = 2,
    Right     = 3,
    UpLeft    = 4,
    UpRight   = 5,
    DownLeft  = 6,
    DownRight = 7,
    Dot       = 8,
}
impl CutDirection {
    pub fn is_default(&self) -> bool {
        *self == Self::Up
    }
}

impl CutDirection {
    pub fn to_quat(&self) -> Quat {
        Quat::from_rotation_z(match self {
            CutDirection::Up => 180f32,
            CutDirection::Down => 0.,
            CutDirection::Left => 90.,
            CutDirection::Right => -90.,
            CutDirection::UpLeft => 135.,
            CutDirection::UpRight => -135.,
            CutDirection::DownLeft => 45.,
            CutDirection::DownRight => -45.,
            CutDirection::Dot => 0.,
        }.to_radians())
    }
}


pub(crate) fn is_value_i<const N: i8>(v: &(impl Num + From<i8>)) -> bool {
    *v == N.into()
}
pub(crate) fn is_value_u<const N: u8>(v: &(impl Num + From<u8>)) -> bool {
    *v == N.into()
}

pub(crate) fn is_value_f<const N: u32>(v: &f32) -> bool {
    *v == f32::from_bits(N)
}

pub(crate) fn default_i<T: Num + From<i8>, const N: i8>() -> T {
    N.into()
}

pub(crate) fn default_u<T: Num + From<u8>, const N: u8>() -> T {
    N.into()
}

pub(crate) fn default_f<const N: u32>() -> f32 {
    f32::from_bits(N)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorNoteV2 {
    #[serde(rename = "_time")]
    pub time: f32,
    #[serde(rename = "_lineIndex")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub line_index: f32,
    #[serde(rename = "_lineLayer")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub line_layer: f32,
    #[serde(rename = "_type")]
    pub typ: Color,
    #[serde(rename = "_cutDirection")]
    pub cut_direction: CutDirection,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorNoteV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "x")]
    pub line_index: f32,
    #[serde(rename = "y")]
    pub line_layer: f32,
    #[serde(rename = "c")]
    pub color: Color,
    #[serde(rename = "d")]
    pub cut_direction: CutDirection,
    #[serde(rename = "a")]
    pub angle_offset: i32,
}

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

// Bomb notes

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BombNoteV2 {
    #[serde(rename = "_time")]
    pub beat: f32,
    #[serde(rename = "_lineIndex")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub line_index: f32,
    #[serde(rename = "_lineLayer")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub line_layer: f32,
    _type: Sentinel<3>,
    #[serde(rename = "_cutDirection")]
    pub cut_direction: CutDirection,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BombNoteV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "x")]
    pub line_index: f32,
    #[serde(rename = "y")]
    pub line_layer: f32,
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

// Obstacles

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum ObstacleV2Type {
    FullHeight = 0,
    Crouch     = 1,
    Free       = 2,
}

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
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub line_index: f32,
    #[serde(rename = "_lineLayer")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub line_layer: f32,
    #[serde(rename = "_width")]
    pub width: f32,
    #[serde(rename = "_height")]
    #[serde(default="default_f::<{ 5f32.to_bits() }>")]
    #[serde(skip_serializing_if="is_value_f::<{ 5f32.to_bits() }>")]
    pub height: f32,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "d")]
    pub duration: f32,
    #[serde(rename = "x")]
    pub line_index: f32,
    #[serde(rename = "y")]
    pub line_layer: f32,
    #[serde(rename = "w")]
    pub width: f32,
    #[serde(rename = "h")]
    pub height: f32,
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

// Arcs

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum ArcMidAnchorMode {
    #[default]
    Straight         = 0,
    Clockwise        = 1,
    CounterClockwise = 2,
}
impl ArcMidAnchorMode {
    pub fn is_default(&self) -> bool {
        *self == Self::Straight
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcV2 {
    #[serde(rename = "_colorType")]
    pub color: Color,
    #[serde(rename = "_headTime")]
    pub head_beat: f32,
    #[serde(rename = "_headLineIndex")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub head_line_index: f32,
    #[serde(rename = "_headLineLayer")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub head_line_layer: f32,
    #[serde(rename = "_headCutDirection")]
    pub head_cut_direction: CutDirection,
    #[serde(rename = "_headControlPointLengthMultiplier")]
    pub head_ctrl_magnitude: f32,
    #[serde(rename = "_tailTime")]
    pub tail_beat: f32,
    #[serde(rename = "_tailLineIndex")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub tail_line_index: f32,
    #[serde(rename = "_tailLineLayer")]
    #[serde(default, skip_serializing_if="Zero::is_zero")]
    pub tail_line_layer: f32,
    #[serde(rename = "_tailCutDirection")]
    pub tail_cut_direction: CutDirection,
    #[serde(rename = "_tailControlPointLengthMultiplier")]
    pub tail_ctrl_magnitude: f32,
    #[serde(rename = "_sliderMidAnchorMode")]
    pub mid_anchor_mode: ArcMidAnchorMode,
    #[serde(rename = "_customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcV3 {
    #[serde(rename = "c")]
    pub color: Color,
    #[serde(rename = "b")]
    pub head_beat: f32,
    #[serde(rename = "x")]
    pub head_line_index: f32,
    #[serde(rename = "y")]
    pub head_line_layer: f32,
    #[serde(rename = "d")]
    pub head_cut_direction: CutDirection,
    #[serde(rename = "mu")]
    pub head_ctrl_magnitude: f32,
    #[serde(rename = "tb")]
    pub tail_beat: f32,
    #[serde(rename = "tx")]
    pub tail_line_index: f32,
    #[serde(rename = "ty")]
    pub tail_line_layer: f32,
    #[serde(rename = "tc")]
    pub tail_cut_direction: CutDirection,
    #[serde(rename = "tmu")]
    pub tail_ctrl_magnitude: f32,
    #[serde(rename = "m")]
    pub mid_anchor_mode: ArcMidAnchorMode,
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

// Chains

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainV3 {
    #[serde(rename = "c")]
    pub color: Color,
    #[serde(rename = "b")]
    pub head_beat: f32,
    #[serde(rename = "x")]
    pub head_line_index: f32,
    #[serde(rename = "y")]
    pub head_line_layer: f32,
    #[serde(rename = "d")]
    pub head_cut_direction: CutDirection,
    #[serde(rename = "tb")]
    pub tail_beat: f32,
    #[serde(rename = "tx")]
    pub tail_line_index: f32,
    #[serde(rename = "ty")]
    pub tail_line_layer: f32,
    #[serde(rename = "sc")]
    pub slice_count: u8,
    #[serde(rename = "s")]
    pub squish_factor: f32,
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
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub head_rotation_lane: f32,
    #[serde(rename = "tr")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub tail_rotation_lane: f32,
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

// Spawn rotations

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum SpawnRotationExecutionTime {
    Early       = 0,
    Late        = 1,
    LegacyEarly = 14,
    LegacyLate  = 15,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum SpawnRotationAngle {
    CCW60 = 0,
    CCW45 = 1,
    CCW30 = 2,
    CCW15 = 3,
    CW15  = 4,
    CW30  = 5,
    CW45  = 6,
    CW60  = 7,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
pub struct LegacySpawnRotationEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "i")]
    pub rotation_angle: SpawnRotationAngle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "e")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "r")]
    pub magnitude: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventV4 {
    #[serde(rename = "b")]
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

// BPM events
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyBPMEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    et: Sentinel<100>,
    #[serde(rename = "f")]
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BPMEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "m")]
    pub bpm: f32,
}

// NJS events
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NJSEventV4 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "i")]
    pub metadata_index: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NJSEventDataV4 {
    #[serde(rename = "p", with="bool_u8_serde")]
    pub extend: bool,
    #[serde(rename = "e", with="easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "d")]
    pub njs_diff: f32,
}

pub struct BpmRegion {
    pub start_sample: usize,
    pub end_sample: usize,
    pub start_beat: f32,
    pub end_beat: f32,
}

// Implementations

impl MapCharacteristic {
    pub fn display_name(&self) -> &str {
        match self {
            MapCharacteristic::Standard => "Standard",
            MapCharacteristic::NoArrows => "NoArrows",
            MapCharacteristic::OneSaber => "OneSaber",
            MapCharacteristic::Degree360 => "360Degree",
            MapCharacteristic::Degree90 => "90Degree",
            MapCharacteristic::Legacy => "Legacy",
            MapCharacteristic::Lightshow => "Lightshow",
            MapCharacteristic::Lawless => "Lawless",
            MapCharacteristic::Unknown(s) => s.as_str(),
        }
    }
}

impl Display for MapCharacteristic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl PartialEq<&str> for MapCharacteristic {
    fn eq(&self, other: &&str) -> bool {
        self.display_name() == *other
    }
}

impl PartialEq<MapCharacteristic> for &str {
    fn eq(&self, other: &MapCharacteristic) -> bool {
        other.eq(self)
    }
}

impl MapDifficulty {
    pub fn display_name(&self) -> &str {
        match self {
            MapDifficulty::Easy => "Easy",
            MapDifficulty::Normal => "Normal",
            MapDifficulty::Hard => "Hard",
            MapDifficulty::Expert => "Expert",
            MapDifficulty::ExpertPlus => "ExpertPlus",
            MapDifficulty::Unknown(s) => s.as_str(),
        }
    }
}

impl Display for MapDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl PartialEq<&str> for MapDifficulty {
    fn eq(&self, other: &&str) -> bool {
        self.display_name() == *other
    }
}

impl PartialEq<MapDifficulty> for &str {
    fn eq(&self, other: &MapDifficulty) -> bool {
        other.eq(self)
    }
}

impl MapVersion {
    pub fn classifier(&self) -> VersionClass {
        match self {
            MapVersion::V2_0_0 |
            MapVersion::V2_2_0 |
            MapVersion::V2_4_0 |
            MapVersion::V2_5_0 |
            MapVersion::V2_6_0 => VersionClass::V2,
            MapVersion::V3_0_0 |
            MapVersion::V3_1_0 |
            MapVersion::V3_2_0 |
            MapVersion::V3_3_0 => VersionClass::V3,
            MapVersion::V4_0_0 |
            MapVersion::V4_1_0 => VersionClass::V4,
        }
    }
}

impl PartialEq<VersionClass> for MapVersion {
    fn eq(&self, other: &VersionClass) -> bool {
        self.classifier() == *other
    }
}
impl PartialEq<MapVersion> for VersionClass {
    fn eq(&self, other: &MapVersion) -> bool {
        other.eq(self)
    }
}

impl InfoVersion {
    pub fn classifier(&self) -> VersionClass {
        match self {
            InfoVersion::V2_0_0 |
            InfoVersion::V2_1_0 => VersionClass::V2,
            InfoVersion::V4_0_0 |
            InfoVersion::V4_0_1 => VersionClass::V4,
        }
    }
}

impl PartialEq<VersionClass> for InfoVersion {
    fn eq(&self, other: &VersionClass) -> bool {
        self.classifier() == *other
    }
}
impl PartialEq<InfoVersion> for VersionClass {
    fn eq(&self, other: &InfoVersion) -> bool {
        other.eq(self)
    }
}

impl VersionClass {
    pub fn as_map_version(&self) -> MapVersion {
        match self {
            VersionClass::V2 => MapVersion::V2_6_0,
            VersionClass::V3 => MapVersion::V3_3_0,
            VersionClass::V4 => MapVersion::V4_1_0,
        }
    }

    pub fn as_info_version(&self) -> InfoVersion {
        match self {
            VersionClass::V2 |
            VersionClass::V3 => InfoVersion::V2_1_0,
            VersionClass::V4 => InfoVersion::V4_0_1,
        }
    }
}


#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum InfoFile {
    V2(InfoV2),
    V4(InfoV4),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum BeatmapFile {
    V2(BeatmapFileV2),
    V3(BeatmapFileV3),
    V4(BeatmapFileV4),
}

impl InfoFile {
    pub fn bpm(&self) -> f32 {
        match self {
            Self::V2(v2) => v2.bpm,
            Self::V4(v4) => v4.audio.bpm,
        }
    }
}

impl Color {
    pub fn to_default_color(&self) -> Vec4 {
        match self {
            Color::Red => Vec4::new(0.749, 0.184, 0.184, 1.),
            Color::Blue => Vec4::new(0.122, 0.388, 0.655, 1.),
        }
    }
}


// extra helpers


macro_rules! convert_u8 {
    ($cl:ty: $values:pat) => {
        impl TryFrom<u8> for $cl {
            type Error = BeatmapDataError;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Ok(match value {
                    $values => unsafe { std::mem::transmute::<u8, Self>(value) },
                    _ => return Err(BeatmapDataError::ToEnum {
                        enum_name: stringify!($cl),
                        val: value as i32,
                    })
                })
            }
        }
        impl From<$cl> for u8 {
            fn from(value: $cl) -> Self {
                unsafe { std::mem::transmute::<$cl, u8>(value) }
            }
        }
    };
}
pub(crate) use convert_u8;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Sentinel<const N: u8>;

impl<const N: u8> Debug for Sentinel<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Sentinel").field(&N).finish()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BeatmapDataError {
    #[error("{val} is not a valid value for {enum_name}")]
    ToEnum { enum_name: &'static str, val: i32 },
}

convert_u8! { Color: 0 | 1 }
convert_u8! { CutDirection: 0..=8 }
convert_u8! { ObstacleV2Type: 0..=2 }
convert_u8! { ArcMidAnchorMode: 0..=2 }
convert_u8! { SpawnRotationExecutionTime: 0 | 1 | 14 | 15 }
convert_u8! { SpawnRotationAngle: 0..=7 }


use serde::{Deserializer, Serializer};
use serde::de::Error as _;

use self::v2::{BeatmapFileV2, InfoV2};
use self::v3::BeatmapFileV3;
use self::v4::{BeatmapFileV4, InfoV4};

impl<const N: u8> Serialize for Sentinel<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(N)
    }
}

impl<'de, const N: u8> Deserialize<'de> for Sentinel<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u8::deserialize(deserializer)?;
        if val != N {
            return Err(D::Error::custom(format!(
                "expected sentinel value {N}, found {val}"
            )));
        }
        Ok(Sentinel)
    }
}

pub(crate) mod bool_u8_serde {
    use serde::{Deserializer, Serializer, Deserialize};

    pub fn serialize<S>(val: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*val as u8)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u8::deserialize(deserializer)?;
        match val {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(serde::de::Error::custom(format!(
                "expected 0 or 1 for bool, found {other}"
            ))),
        }
    }
}

pub(crate) mod easing_as_i8 {
    use super::Easing;
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de::Error as _;

    pub fn serialize<S>(easing: &Easing, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i8(i8::from(*easing))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Easing, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = i8::deserialize(deserializer)?;
        Easing::try_from(val).map_err(D::Error::custom)
    }
}
