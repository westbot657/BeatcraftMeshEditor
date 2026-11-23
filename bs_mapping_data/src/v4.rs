use crate::default_f;
use serde::{Deserialize, Serialize};

use super::easing::Easing;
use super::{ArcMidAnchorMode, Color, CutDirection, SpawnRotationExecutionTime};
use super::{bool_u8_serde, easing_as_i8};
use super::{is_value_f, is_value_i, is_value_u};
use crate::v2_v3_v4::{LightEventTypeV2, LightEventValueV2};
use crate::v3_v4::DistributionType;
use crate::v3_v4::*;
use crate::{LightshowVersion, MapVersion, convert_int};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BombNoteDataV4 {
    #[serde(rename = "x")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_index: f32,
    #[serde(rename = "y")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub line_layer: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NJSEventV4 {
    #[serde(rename = "b")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub beat: f32,
    #[serde(rename = "i")]
    #[serde(default, skip_serializing_if = "is_value_u::<0>")]
    pub metadata_index: u32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventV4 {
    #[serde(rename = "b")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub beat: f32,
    #[serde(rename = "i")]
    pub metadata_index: u32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventDataV4 {
    #[serde(rename = "t")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "r")]
    pub magnitude: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
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


#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct BasicLightEventV4 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "i")]
    pub metadata_index: u32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct BasicLightEventDataV4 {
    #[serde(rename = "t")]
    pub typ: LightEventTypeV2,
    #[serde(rename = "i")]
    pub value: LightEventValueV2,
    #[serde(rename = "f")]
    pub float_value: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ColorBoostEventV4 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "i")]
    pub metadata_index: u32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ColorBoostEventDataV4 {
    #[serde(rename = "b")]
    #[serde(with = "bool_u8_serde")]
    pub boost: bool,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct WaypointV4 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "i")]
    pub metadata_index: u32,
}


#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum WaypointDirection {
    #[default]
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    UpLeft = 4,
    UpRight = 5,
    DownLeft = 6,
    DownRight = 7,
}
convert_int!(WaypointDirection: 0..=7);

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct WaypointDataV4 {
    #[serde(rename = "x")]
    pub line_index: f32,
    #[serde(rename = "y")]
    pub line_layer: f32,
    #[serde(rename = "d")]
    pub offset_direction: WaypointDirection,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum KeywordEventLane {
    L40 = 40,
    L41 = 41,
    L42 = 42,
    L43 = 43,
}
convert_int!(KeywordEventLane: 40..=43);

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct KeywordEventV4 {
    #[serde(rename = "k")]
    pub keyword: String,
    #[serde(rename = "e")]
    pub event_types: Vec<KeywordEventLane>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct KeywordEventCollectionV4 {
    #[serde(rename = "d")]
    pub data: Vec<KeywordEventV4>,
}
impl KeywordEventCollectionV4 {
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}


#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum EventBoxGroupTypeV4 {
    #[default]
    None = 0,
    LightColor = 1,
    LightRotation = 2,
    LightTranslation = 3,
    FloatFx = 4,
}
convert_int!(EventBoxGroupTypeV4: 0..=4);

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct EventBoxGroupV4 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "g")]
    pub group: u32,
    #[serde(rename = "t")]
    pub typ: EventBoxGroupTypeV4,
    #[serde(rename = "e")]
    pub event_boxes: Vec<EventBoxV4>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct EventBoxV4 {
    #[serde(rename = "f")]
    pub index_filter_metadata_index: u32,
    #[serde(rename = "e")]
    pub metadata_index: u32,
    #[serde(rename = "l")]
    pub events: Vec<EventV4>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct EventV4 {
    #[serde(rename = "b")]
    pub beat_offset: f32,
    #[serde(rename = "i")]
    pub metadata_index: u32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightColorEventBoxV4 {
    #[serde(rename = "w")]
    pub beat_distribution_value: f32,
    #[serde(rename = "d")]
    pub beat_distribution_type: DistributionType,
    #[serde(rename = "s")]
    pub brightness_distribution_value: f32,
    #[serde(rename = "t")]
    pub brightness_distribution_type: DistributionType,
    #[serde(rename = "b")]
    #[serde(with = "bool_u8_serde")]
    pub affects_first: bool,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
}
impl Default for LightColorEventBoxV4 {
    fn default() -> Self {
        Self {
            beat_distribution_value: 1.0,
            beat_distribution_type: Default::default(),
            brightness_distribution_value: 1.0,
            brightness_distribution_type: Default::default(),
            affects_first: false,
            easing: Easing::easeLinear,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightColorEventV4 {
    #[serde(rename = "p")]
    #[serde(default)]
    pub transition_type: TransitionType,
    #[serde(rename = "e")]
    #[serde(default)]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "c")]
    #[serde(default)]
    pub color: LightEventColor,
    #[serde(rename = "b")]
    #[serde(default = "default_f::<{ 1f32.to_bits() }>")]
    pub brightness: f32,
    #[serde(rename = "f")]
    #[serde(default)]
    pub strobe_frequency: f32,
    #[serde(rename = "sb")]
    #[serde(default)]
    pub strobe_brightness: f32,
    #[serde(rename = "sf")]
    #[serde(default)]
    #[serde(with = "bool_u8_serde")]
    pub strobe_fade: bool,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightRotationEventBoxV4 {
    #[serde(rename = "w")]
    pub beat_distribution_value: f32,
    #[serde(rename = "d")]
    pub beat_distribution_type: DistributionType,
    #[serde(rename = "s")]
    pub rotation_distribution_value: f32,
    #[serde(rename = "t")]
    pub rotation_distribution_type: DistributionType,
    #[serde(rename = "b")]
    #[serde(with = "bool_u8_serde")]
    pub affects_first: bool,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "a")]
    pub axis: EventAxis,
    #[serde(rename = "f")]
    #[serde(with = "bool_u8_serde")]
    pub invert_axis: bool,
}
impl Default for LightRotationEventBoxV4 {
    fn default() -> Self {
        Self {
            beat_distribution_value: 1.,
            beat_distribution_type: Default::default(),
            rotation_distribution_value: 1.,
            rotation_distribution_type: Default::default(),
            affects_first: false,
            easing: Default::default(),
            axis: Default::default(),
            invert_axis: false,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightRotationEventV4 {
    #[serde(rename = "p")]
    pub transition_type: TransitionType,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "r")]
    pub magnitude: f32,
    #[serde(rename = "d")]
    pub direction: RotationDirection,
    #[serde(rename = "l")]
    pub loop_count: u32,
}
impl Default for LightRotationEventV4 {
    fn default() -> Self {
        Self {
            transition_type: Default::default(),
            easing: Easing::easeLinear,
            magnitude: 0.0,
            direction: Default::default(),
            loop_count: 0,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightTranslationEventBoxV4 {
    #[serde(rename = "w")]
    pub beat_distribution_value: f32,
    #[serde(rename = "d")]
    pub beat_distribution_type: DistributionType,
    #[serde(rename = "s")]
    pub gap_distribution_value: f32,
    #[serde(rename = "t")]
    pub gap_distribution_type: DistributionType,
    #[serde(rename = "b")]
    #[serde(with = "bool_u8_serde")]
    pub affects_first: bool,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "a")]
    pub axis: EventAxis,
    #[serde(rename = "f")]
    #[serde(with = "bool_u8_serde")]
    pub invert_axis: bool,
}
impl Default for LightTranslationEventBoxV4 {
    fn default() -> Self {
        Self {
            beat_distribution_value: 1.0,
            beat_distribution_type: Default::default(),
            gap_distribution_value: 1.0,
            gap_distribution_type: Default::default(),
            affects_first: false,
            easing: Easing::easeLinear,
            axis: Default::default(),
            invert_axis: false,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightTranslationEventV4 {
    #[serde(rename = "p")]
    pub transition_type: TransitionType,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "t")]
    pub magnitude: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FxEventBoxV4 {
    #[serde(rename = "w")]
    pub beat_distribution_value: f32,
    #[serde(rename = "d")]
    pub beat_distribution_type: DistributionType,
    #[serde(rename = "s")]
    pub fx_distribution_value: f32,
    #[serde(rename = "t")]
    pub fx_distribution_type: DistributionType,
    #[serde(rename = "b")]
    #[serde(with = "bool_u8_serde")]
    pub affects_first: bool,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
}
impl Default for FxEventBoxV4 {
    fn default() -> Self {
        Self {
            beat_distribution_value: 1.0,
            beat_distribution_type: Default::default(),
            fx_distribution_value: 1.0,
            fx_distribution_type: Default::default(),
            affects_first: false,
            easing: Easing::easeLinear,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FloatFxEventV4 {
    #[serde(rename = "p")]
    pub transition_type: TransitionType,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "v")]
    pub value: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct LightshowFileV4 {
    pub version: LightshowVersion,
    pub basic_events: Vec<BasicLightEventV4>,
    pub basic_events_data: Vec<BasicLightEventDataV4>,
    pub color_boost_events: Vec<ColorBoostEventV4>,
    // WHY DOES THIS EXIST??? it'll only ever have 2 elements????????
    pub color_boost_events_data: Vec<ColorBoostEventDataV4>,
    pub waypoints: Vec<WaypointV4>,
    pub waypoints_data: Vec<WaypointDataV4>,
    pub basic_event_types_with_keywords: KeywordEventCollectionV4,
    pub event_box_groups: Vec<EventBoxGroupV4>,
    pub index_filters: Vec<IndexFilterV3>,
    pub light_color_event_boxes: Vec<LightColorEventBoxV4>,
    pub light_color_events: Vec<LightColorEventV4>,
    pub light_rotation_event_boxes: Vec<LightRotationEventBoxV4>,
    pub light_rotation_events: Vec<LightRotationEventV4>,
    pub light_translation_event_boxes: Vec<LightTranslationEventBoxV4>,
    pub light_translation_events: Vec<LightTranslationEventV4>,
    pub fx_event_boxes: Vec<FxEventBoxV4>,
    pub float_fx_events: Vec<FloatFxEventV4>,
    #[serde(default)]
    pub use_normal_events_as_compatible_events: bool,
}
