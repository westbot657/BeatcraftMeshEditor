use serde::{Deserialize, Serialize};

use crate::easing::Easing;

use super::{ArcV3, BombNoteV3, ChainV3, Color, ColorNoteV3, MapVersion, ObstacleV3, convert_u8};
use super::{bool_u8_serde, easing_as_i8};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BpmEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "m")]
    pub bpm: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RotationEventV3 {
    #[serde(rename = "b")]
    pub beat: f32,
    #[serde(rename = "e")]
    pub e: u8,
    #[serde(rename = "r")]
    pub rotation: f32,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasicBeatmapEventV3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorBoostV3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexFilterV3 {

}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightColorEventBoxV3 {
    #[serde(rename = "f")]
    pub filter: IndexFilterV3,
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
    pub brightness_distribution_easing: Easing,
    #[serde(rename = "e")]
    pub events: Vec<LightColorEventV3>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightColorEventV3 {
    #[serde(rename = "b")]
    pub beat_offset: f32,
    #[serde(rename = "i")]
    pub transition_type: u8,
    #[serde(rename = "c")]
    pub color: Color,
    #[serde(rename = "s")]
    pub brightness: f32,
    #[serde(rename = "f")]
    pub strobe_frequency: f32,
    #[serde(rename = "sb")]
    pub strobe_brightness: f32,
    #[serde(rename = "sf")]
    pub strobe_fade: f32,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightRotationEventBoxV3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightTranslationEventBoxV3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VfxEventBoxV3;


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
    pub chains: Vec<ChainV3>,
    #[serde(rename = "burstSliders")]
    pub arcs: Vec<ArcV3>,
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
    pub light_translation_event_box_groups: Vec<EventBoxGroupV3<LightTranslationEventBoxV3>>,
    #[serde(rename = "vfxEventBoxGroups")]
    pub vfx_event_box_groups: Vec<EventBoxGroupV3<VfxEventBoxV3>>,
    #[serde(rename = "_fxEventsCollections")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    wtf_is_this: Option<serde_json::Value>,
    #[serde(rename = "useNormalEventsAsCompatibleEvents")]
    pub use_normal_events_as_compatible_events: bool,
}

convert_u8! {  }

