use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::easing::Easing;

use super::v2::{ColorBoostValueV2, GagaSideV2, HydraulicsTypeV2, LightEventTypeV2, LightEventValueV2, RingLightEventTypeV2, SpinningLaserSideV2};
use super::{ArcV3, BeatmapDataError, BombNoteV3, ChainV3, ColorNoteV3, LegacyBPMEventV3, LegacySpawnRotationEventV3, MapVersion, ObstacleV3, Sentinel, SpawnRotationExecutionTime, convert_u8};
use super::{bool_u8_serde, easing_as_i8};
use super::{default_u, is_value_u, is_value_f};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BpmEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "m")]
    pub bpm: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotationEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "e")]
    pub execution_time: SpawnRotationExecutionTime,
    #[serde(rename = "r")]
    pub rotation: f32,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: LightEventTypeV2,
    #[serde(rename = "i")]
    pub value: LightEventValueV2,
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyColorBoostV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    typ: Sentinel<5>,
    #[serde(rename = "i")]
    pub boost: ColorBoostValueV2,
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RingLightEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: RingLightEventTypeV2,
    #[serde(rename = "i")]
    pub value: u32,
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpinningLaserEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: SpinningLaserSideV2,
    #[serde(rename = "i")]
    pub value: u32,
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HydraulicsEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: HydraulicsTypeV2,
    #[serde(rename = "i")]
    pub value: u32,
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GagaEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "et")]
    pub typ: GagaSideV2,
    #[serde(rename = "i")]
    pub value: u32,
    f: f32,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorBoostV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "o")]
    pub boost: bool, // rare non-numeric bool appearence
}


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum IndexFilterType {
    Division = 1,
    StepAndOffset = 2,
}
convert_u8! { IndexFilterType : 1 | 2 }

bitflags! {
    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    pub struct RandomizationBehavior: u8 {
        const NoRandom       = 0;
        const KeepOrder      = 1;
        const RandomElements = 2;
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    pub struct LimitBehavior: u8 {
        const None         = 0;
        const Duration     = 1;
        const Distribution = 2;
    }

}

macro_rules! flags_as_u8 {
    ( $val:ty ) => {
        impl From<u8> for $val {
            fn from(value: u8) -> Self {
                Self::from_bits_truncate(value)
            }
        }
        impl From<$val> for u8 {
            fn from(value: $val) -> Self {
                value.bits()
            }
        }
    };
}
flags_as_u8! { RandomizationBehavior }
flags_as_u8! { LimitBehavior }

impl Default for RandomizationBehavior {
    fn default() -> Self {
        Self::empty()
    }
}
impl Default for LimitBehavior {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexFilterV3 {
    #[serde(rename = "c")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunks: Option<u32>,
    #[serde(rename = "f")]
    pub typ: IndexFilterType,
    #[serde(rename = "p")]
    pub param_0: u32,
    #[serde(rename = "t")]
    pub param_1: u32,
    #[serde(rename = "r")]
    #[serde(with = "bool_u8_serde")]
    pub reverse: bool,
    #[serde(rename = "n")]
    #[serde(default, skip_serializing_if = "RandomizationBehavior::is_empty")]
    pub randomization_behavior: RandomizationBehavior,
    #[serde(rename = "s")]
    #[serde(default = "default_u::<_, 0>", skip_serializing_if = "is_value_u::<0>")]
    pub randomization_seed: u64,
    #[serde(rename = "l")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub limit_percentage: f32,
    #[serde(rename = "d")]
    #[serde(default, skip_serializing_if = "LimitBehavior::is_empty")]
    pub limit_behavior: LimitBehavior,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventBoxGroupV3<E>
where
    E: Clone + std::fmt::Debug
{
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "g")]
    pub group: u32,
    #[serde(rename = "e")]
    pub events: Vec<E>,
}


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum DistributionType {
    Wave = 1,
    Step = 2,
}


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum EventAxis {
    X = 0,
    Y = 1,
    Z = 2,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum TransitionType {
    Transition = 0,
    Extend     = 1,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum RotationDirection {
    Automatic        = 0,
    Clockwise        = 1,
    CounterClockwise = 2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(default, skip_serializing_if = "Easing::is_default")]
    pub brightness_distribution_easing: Easing,
    #[serde(rename = "e")]
    pub events: Vec<LightColorEventV3>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightColorEventV3 {
    #[serde(rename = "b")]
    pub beat_offset: f32,
    #[serde(rename = "i")]
    pub transition_type: TransitionType,
    #[serde(rename = "c")]
    pub color: LightEventColor,
    #[serde(rename = "s")]
    pub brightness: f32,
    #[serde(rename = "f")]
    pub strobe_frequency: f32,
    #[serde(rename = "sb")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 1f32.to_bits() }>")]
    pub strobe_brightness: f32,
    #[serde(rename = "sf")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub strobe_fade: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum LightEventColor {
    Primary   = 0,
    Secondary = 1,
    White     = 2,
}
convert_u8! { LightEventColor : 0..=2 }


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(default, skip_serializing_if = "Easing::is_default")]
    pub rotation_distribution_easing: Easing,
    #[serde(rename = "a")]
    pub axis: EventAxis,
    #[serde(rename = "r")]
    #[serde(with = "bool_u8_serde")]
    pub invert_axis: bool,
    #[serde(rename = "l")]
    pub events: Vec<LightRotationEventV3>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(default, skip_serializing_if = "Easing::is_default")]
    pub gap_distribution_easing: Easing,
    #[serde(rename = "a")]
    pub axis: EventAxis,
    #[serde(rename = "invert_axis")]
    #[serde(with = "bool_u8_serde")]
    pub invert_axis: bool,
    #[serde(rename = "l")]
    pub events: Vec<LightTranslationEventV3>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FxEventsCollectionV3 {
    #[serde(rename = "_fl")]
    pub float_fx_event_collection: Vec<VfxEventV3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    _il: Option<serde_json::Value>,
}
impl FxEventsCollectionV3 {
    pub fn is_empty(&self) -> bool {
        self.float_fx_event_collection.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatmapFileV3 {
    pub version: MapVersion,
    #[serde(rename = "bpmEvents")]
    pub bpm_events: Vec<BpmEventV3>,
    #[serde(rename = "rotationEvents")]
    pub rotation_events: Vec<RotationEventV3>,
    #[serde(rename = "colorNotes")]
    pub color_notes: Vec<ColorNoteV3>,
    #[serde(rename = "bombNotes")]
    pub bomb_notes: Vec<BombNoteV3>,
    pub obstacles: Vec<ObstacleV3>,
    #[serde(rename = "sliders")]
    pub arcs: Vec<ArcV3>,
    #[serde(rename = "burstSliders")]
    pub chains: Vec<ChainV3>,
    #[serde(rename = "basicBeatmapEvents")]
    pub basic_beatmap_events: Vec<BasicBeatmapEventV3>,
    #[serde(rename = "colorBoostBeatmapEvents")]
    pub color_boost_events: Vec<ColorBoostV3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    waypoints: Option<serde_json::Value>,
    #[serde(rename = "basicEventTypesWithKeywords")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    idc: Option<serde_json::Value>,
    #[serde(rename = "lightColorEventBoxGroups")]
    pub light_color_event_box_groups: Vec<EventBoxGroupV3<LightColorEventBoxV3>>,
    #[serde(rename = "lightRotationEventBoxGroups")]
    pub light_rotation_event_box_groups: Vec<EventBoxGroupV3<LightRotationEventBoxV3>>,
    #[serde(rename = "lightTranslationEventBoxGroups")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub light_translation_event_box_groups: Vec<EventBoxGroupV3<LightTranslationEventBoxV3>>,
    #[serde(rename = "vfxEventBoxGroups")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vfx_event_box_groups: Vec<EventBoxGroupV3<VfxEventBoxV3>>,
    #[serde(rename = "_fxEventsCollection")]
    #[serde(default, skip_serializing_if = "FxEventsCollectionV3::is_empty")]
    pub vfx_events_collections: FxEventsCollectionV3,
    #[serde(rename = "useNormalEventsAsCompatibleEvents")]
    pub use_normal_events_as_compatible_events: bool,
    #[serde(rename = "customData")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<serde_json::Value>,
}

convert_u8! { DistributionType : 1 | 2 }
convert_u8! { EventAxis : 0..=2 }
convert_u8! { TransitionType : 0 | 1 }
convert_u8! { RotationDirection : 0..=2 }

