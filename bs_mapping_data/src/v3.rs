use serde::{Deserialize, Serialize};

use crate::convert_int;
#[cfg(feature = "custom_data")]
use crate::custom_v3::{BeatmapFileCustomDataV3, CustomArcDataV3, CustomBombDataV3, CustomChainDataV3, CustomNoteDataV3, CustomObstacleDataV3};
use crate::easing::Easing;
use crate::is_value_f;
use crate::v2_v3_v4::*;
use crate::v3_v4::*;
use crate::{
    ArcMidAnchorMode, Color, CutDirection, MapVersion, Sentinel, SpawnRotationAngle,
    SpawnRotationExecutionTime,
};
use crate::{bool_u8_serde, easing_as_i8};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
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
    #[cfg(feature = "custom_data")]
    #[serde(rename = "customData")]
    #[serde(skip_serializing_if = "CustomNoteDataV3::is_empty")]
    pub custom_data: CustomNoteDataV3,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct BombNoteV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "x")]
    pub line_index: f32,
    #[serde(rename = "y")]
    pub line_layer: f32,
    #[cfg(feature = "custom_data")]
    #[serde(rename = "customData")]
    #[serde(skip_serializing_if = "CustomBombDataV3::is_empty")]
    pub custom_data: CustomBombDataV3,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
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
    #[cfg(feature = "custom_data")]
    #[serde(rename = "customData")]
    #[serde(skip_serializing_if = "CustomObstacleDataV3::is_empty")]
    pub custom_data: CustomObstacleDataV3,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
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
    #[cfg(feature = "custom_data")]
    #[serde(rename = "customData")]
    #[serde(skip_serializing_if = "CustomArcDataV3::is_empty")]
    pub custom_data: CustomArcDataV3,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
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
    #[cfg(feature = "custom_data")]
    #[serde(rename = "customData")]
    #[serde(skip_serializing_if = "CustomChainDataV3::is_empty")]
    pub custom_data: CustomChainDataV3,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacySpawnRotationEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "i")]
    pub rotation_angle: SpawnRotationAngle,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRotationEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "e")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "r")]
    pub magnitude: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyBPMEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    et: Sentinel<100>,
    #[serde(rename = "f")]
    pub value: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BPMEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "m")]
    pub bpm: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BpmEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "m")]
    pub bpm: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotationEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "e")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "r")]
    pub rotation: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BasicBeatmapEventV3 {
    SpawnRotation(LegacySpawnRotationEventV3),
    Light(LightEventV3),
    ColorBoost(LegacyColorBoostV3),
    Ring(RingLightEventV3),
    RotatingLights(SpinningLaserEventV3),
    Hydraulics(HydraulicsEventV3),
    Gaga(GagaEventV3),
    BPM(LegacyBPMEventV3),
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: LightEventTypeV2,
    #[serde(rename = "i")]
    pub value: LightEventValueV2,
    /// float value is unused
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyColorBoostV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    typ: Sentinel<5>,
    #[serde(rename = "i")]
    pub boost: ColorBoostValueV2,
    /// float value is unused
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RingLightEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: RingLightEventTypeV2,
    #[serde(rename = "i")]
    pub value: u32,
    /// float value is unused
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpinningLaserEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: SpinningLaserSideV2,
    #[serde(rename = "i")]
    pub value: u32,
    /// float value is unused
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HydraulicsEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: HydraulicsTypeV2,
    #[serde(rename = "i")]
    pub value: u32,
    /// float value is unused
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GagaEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: GagaSideV2,
    #[serde(rename = "i")]
    pub value: u32,
    /// float value is unused
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorBoostV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "o")]
    pub boost: bool, // rare non-numeric bool appearance
}


#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct EventBoxGroupV3<E>
where
    E: Clone + std::fmt::Debug,
{
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "g")]
    pub group: u32,
    #[serde(rename = "e")]
    pub events: Vec<E>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum LightColorTransitionTypeV3 {
    #[default]
    Instant = 0,
    Transition = 1,
    Extend = 2,
}
convert_int! { LightColorTransitionTypeV3 : 0..=2 }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightColorEventBoxV3 {
    #[serde(rename = "f")]
    pub index_filter: IndexFilterV3,
    #[serde(rename = "w")]
    pub beat_distribution_value: f32,
    #[serde(rename = "d")]
    pub beat_distribution_type: DistributionType,
    #[serde(rename = "r")]
    pub brightness_distribution_value: f32,
    #[serde(rename = "t")]
    pub brightness_distribution_type: DistributionType,
    #[serde(rename = "b")]
    #[serde(with = "bool_u8_serde")]
    pub brightness_distribution_affects_first: bool,
    #[serde(rename = "i")]
    #[serde(with = "easing_as_i8")]
    #[serde(skip_serializing_if = "Easing::is_default")]
    pub brightness_distribution_easing: Easing,
    #[serde(rename = "e")]
    pub events: Vec<LightColorEventV3>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightColorEventV3 {
    #[serde(rename = "b")]
    pub beat_offset: f32,
    #[serde(rename = "i")]
    pub transition_type: LightColorTransitionTypeV3,
    #[serde(rename = "c")]
    pub color: LightEventColor,
    #[serde(rename = "s")]
    pub brightness: f32,
    #[serde(rename = "f")]
    pub strobe_frequency: f32,
    #[serde(rename = "sb")]
    #[serde(skip_serializing_if = "is_value_f::<{ 1f32.to_bits() }>")]
    pub strobe_brightness: f32,
    #[serde(rename = "sf")]
    #[serde(skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub strobe_fade: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightRotationEventBoxV3 {
    #[serde(rename = "f")]
    pub index_filter: IndexFilterV3,
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
    pub rotation_distribution_affects_first: bool,
    #[serde(rename = "i")]
    #[serde(with = "easing_as_i8")]
    #[serde(skip_serializing_if = "Easing::is_default")]
    pub rotation_distribution_easing: Easing,
    #[serde(rename = "a")]
    pub axis: EventAxis,
    #[serde(rename = "r")]
    #[serde(with = "bool_u8_serde")]
    pub invert_axis: bool,
    #[serde(rename = "l")]
    pub events: Vec<LightRotationEventV3>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightRotationEventV3 {
    #[serde(rename = "b")]
    pub beat_offset: f32,
    #[serde(rename = "p")]
    pub transition_type: TransitionType,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    /// Rotation in degrees
    #[serde(rename = "r")]
    pub magnitude: f32,
    #[serde(rename = "o")]
    pub direction: RotationDirection,
    #[serde(rename = "l")]
    pub loop_count: u32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightTranslationEventBoxV3 {
    #[serde(rename = "f")]
    pub index_filter: IndexFilterV3,
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
    pub gap_distribution_affects_first: bool,
    #[serde(rename = "i")]
    #[serde(with = "easing_as_i8")]
    #[serde(skip_serializing_if = "Easing::is_default")]
    pub gap_distribution_easing: Easing,
    #[serde(rename = "a")]
    pub axis: EventAxis,
    #[serde(rename = "r")]
    #[serde(with = "bool_u8_serde")]
    pub invert_axis: bool,
    #[serde(rename = "l")]
    pub events: Vec<LightTranslationEventV3>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct LightTranslationEventV3 {
    #[serde(rename = "b")]
    pub beat_offset: f32,
    #[serde(rename = "p")]
    pub transition_type: TransitionType,
    #[serde(rename = "e")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "t")]
    pub magnitude: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct VfxEventBoxV3 {
    #[serde(rename = "f")]
    pub index_filter: IndexFilterV3,
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
    pub fx_distribution_affects_first: bool,
    #[serde(rename = "i")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "l")]
    pub float_fx_event_metadata_indices: Vec<u32>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FxEventsCollectionV3 {
    #[serde(rename = "_fl")]
    pub float_fx_event_collection: Vec<VfxEventV3>,
    /// unused by beat saber itself
    #[serde(default, skip_serializing_if = "Option::is_none")]
    _il: Option<serde_json::Value>,
}
impl FxEventsCollectionV3 {
    pub fn is_empty(&self) -> bool {
        self.float_fx_event_collection.is_empty()
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct VfxEventV3 {
    #[serde(rename = "b")]
    pub beat_offset: f32,
    #[serde(rename = "p")]
    pub transition_type: TransitionType,
    #[serde(rename = "i")]
    #[serde(with = "easing_as_i8")]
    pub easing: Easing,
    #[serde(rename = "v")]
    pub value: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapFileV3 {
    pub version: MapVersion,
    pub bpm_events: Vec<BpmEventV3>,
    pub rotation_events: Vec<RotationEventV3>,
    pub color_notes: Vec<ColorNoteV3>,
    pub bomb_notes: Vec<BombNoteV3>,
    pub obstacles: Vec<ObstacleV3>,
    #[serde(rename = "sliders")]
    pub arcs: Vec<ArcV3>,
    #[serde(rename = "burstSliders")]
    pub chains: Vec<ChainV3>,
    pub basic_beatmap_events: Vec<BasicBeatmapEventV3>,
    #[serde(rename = "colorBoostBeatmapEvents")]
    pub color_boost_events: Vec<ColorBoostV3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waypoints: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_event_types_with_keywords: Option<serde_json::Value>,
    pub light_color_event_box_groups: Vec<EventBoxGroupV3<LightColorEventBoxV3>>,
    pub light_rotation_event_box_groups: Vec<EventBoxGroupV3<LightRotationEventBoxV3>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub light_translation_event_box_groups: Vec<EventBoxGroupV3<LightTranslationEventBoxV3>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vfx_event_box_groups: Vec<EventBoxGroupV3<VfxEventBoxV3>>,
    #[serde(rename = "_fxEventsCollection")]
    #[serde(default, skip_serializing_if = "FxEventsCollectionV3::is_empty")]
    pub vfx_events_collections: FxEventsCollectionV3,
    pub use_normal_events_as_compatible_events: bool,
    #[cfg(feature = "custom_data")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<BeatmapFileCustomDataV3>,
}
