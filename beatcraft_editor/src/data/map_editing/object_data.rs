use crate::data::map_editing::DataElement;
use crate::data::map_editing::resolver::Value;
use crate::data::map_editing::values::{ArcMidAnchorModeValue, BoolValue, CutDirectionValue, F32Value, I32Value, NoteTypeValue, OptVec2Value, OptVec3Value, OptionalBoolValue, OptionalColorValue, OptionalF32Value, U8Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use bs_mapping_data::custom_v2::{CommonCustomDataV2, CustomArcDataV2, CustomBombDataV2, CustomNoteDataV2, CustomObstacleDataV2};
use bs_mapping_data::custom_v3::{CommonCustomDataV3, CustomBombDataV3, CustomChainDataV3, CustomNoteDataV3, CustomObstacleDataV3};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectSource {
    Element {
        index: usize,
    },
    TemplatePlacement {
        index_of_placement: usize,
        index_of_element: usize,
    },
    TemplateDefinition {
        name: String,
        index: usize,
    },
}

#[derive(Serialize, Deserialize)]
pub struct TemplatePlacement {
    pub template: String,
    pub beat: F32Value,
    pub rotation_lane: I32Value,
    pub inputs: HashMap<String, Value>,
    pub mirror_y: bool,
    pub swap_colors: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub elements: Vec<DataElement>,
    pub values: HashMap<String, Value>,
    pub inputs: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CustomData {
    // noodle
    pub coordinates: OptVec2Value,
    pub world_rotation: OptVec3Value,
    pub local_rotation: OptVec3Value,
    pub scale: OptVec3Value,
    pub custom_njs: OptionalF32Value,
    pub custom_njs_offset: OptionalF32Value,
    pub uninteractable: BoolValue,
    // chroma
    pub color: OptionalColorValue,

    pub animation: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct NoteCustomData {
    pub base: CustomData,
    // noodle
    // pub flip: ?,
    pub disable_note_gravity: BoolValue,
    pub disable_note_look: BoolValue,
    // chroma
    pub spawn_effect: OptionalBoolValue,
    pub disable_debris: BoolValue,
}
impl Deref for NoteCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct BombCustomData {
    pub base: CustomData,
    // pub flip: ?,
    pub disable_note_gravity: BoolValue,
}
impl Deref for BombCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ObstacleCustomData {
    #[serde(flatten)]
    pub base: CustomData,
    pub size: OptVec3Value,
}
impl Deref for ObstacleCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ChainCustomData {
    #[serde(flatten)]
    pub base: CustomData,
    
    pub disable_note_gravity: BoolValue,
    pub disable_note_look: BoolValue,
    
    pub spawn_effect: OptionalBoolValue,
    pub disable_debris: BoolValue,
    
    pub tail_coordinates: OptVec2Value,
}
impl Deref for ChainCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ArcCustomData {
    #[serde(flatten)]
    pub base: CustomData,
}
impl Deref for ArcCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Serialize, Deserialize)]
pub struct NoteData {
    pub beat: F32Value,
    pub cut_direction: CutDirectionValue,
    pub x: F32Value,
    pub y: F32Value,
    pub lane_rotation_deg: F32Value,
    pub note_type: NoteTypeValue,
    pub angle_offset_deg: F32Value,
    pub custom_data: NoteCustomData,
}

#[derive(Serialize, Deserialize)]
pub struct BombData {
    pub beat: F32Value,
    pub x: F32Value,
    pub y: F32Value,
    pub lane_rotation_deg: F32Value,
    pub custom_data: BombCustomData,
}

#[derive(Serialize, Deserialize)]
pub struct ObstacleData {
    pub beat: F32Value,
    pub x: F32Value,
    pub y: F32Value,
    pub duration: F32Value,
    pub lane_rotation_deg: F32Value,
    pub width: F32Value,
    pub height: F32Value,
    pub custom_data: ObstacleCustomData,
}

#[derive(Serialize, Deserialize)]
pub struct ObstacleTextData {
    pub beat: F32Value,
    pub text: F32Value,
    pub x: F32Value,
    pub y: F32Value,
    pub duration: F32Value,
    pub lane_rotation_deg: F32Value,
    pub width: F32Value,
    pub height: F32Value,
    /// if set, this overrides the length calculated from the duration
    pub length: OptionalF32Value,
    pub custom_data: CustomData,
}

#[derive(Serialize, Deserialize)]
pub struct ChainData {
    pub beat: F32Value,
    pub cut_direction: CutDirectionValue,
    pub x: F32Value,
    pub y: F32Value,
    pub head_lane_rotation_deg: F32Value,
    pub tail_beat: F32Value,
    pub tx: F32Value,
    pub ty: F32Value,
    pub tail_lane_rotation_deg: F32Value,
    pub squish_factor: F32Value,
    pub link_count: U8Value,
    pub note_type: NoteTypeValue,
    pub custom_data: ChainCustomData,
}

#[derive(Serialize, Deserialize)]
pub struct ArcData {
    pub beat: F32Value,
    pub cut_direction: CutDirectionValue,
    pub x: F32Value,
    pub y: F32Value,
    pub head_ctrl_magnitude: F32Value,
    pub lane_rotation_deg: F32Value,
    pub tail_beat: F32Value,
    pub tail_cut_direction: CutDirectionValue,
    pub tx: F32Value,
    pub ty: F32Value,
    pub tail_ctrl_magnitude: F32Value,
    pub mid_anchor_mode: ArcMidAnchorModeValue,
    pub note_type: NoteTypeValue,
    pub custom_data: CustomData,
}

impl From<&CommonCustomDataV2> for CustomData {
    fn from(value: &CommonCustomDataV2) -> Self {
        Self {
            coordinates: value.position.into(),
            world_rotation: value.rotation.into(),
            local_rotation: value.local_rotation.into(),
            scale: value.scale.into(),
            custom_njs: value.note_jump_movement_speed.into(),
            custom_njs_offset: value.note_jump_start_beat_offset.into(),
            uninteractable: value.uninteractable.into(),
            color: value.color.into(),
            animation: value.animation.clone(),
        }
    }
}

impl From<&CustomNoteDataV2> for NoteCustomData {
    fn from(value: &CustomNoteDataV2) -> Self {
        Self {
            base: (&value.base).into(),
            disable_note_gravity: value.disable_note_gravity.into(),
            disable_note_look: value.disable_note_look.into(),
            spawn_effect: Default::default(),
            disable_debris: Default::default(),
        }
    }
}

impl From<&CustomBombDataV2> for BombCustomData {
    fn from(value: &CustomBombDataV2) -> Self {
        Self {
            base: (&value.base).into(),
            disable_note_gravity: value.disable_note_gravity.into(),
        }
    }
}

impl From<&CustomObstacleDataV2> for ObstacleCustomData {
    fn from(value: &CustomObstacleDataV2) -> Self {
        Self {
            base: (&value.base).into(),
            size: value.size.into(),
        }
    }
}

impl From<&CustomArcDataV2> for ArcCustomData {
    fn from(value: &CustomArcDataV2) -> Self {
        Self {
            base: (&value.base).into()
        }
    }
}

impl From<&CommonCustomDataV3> for CustomData {
    fn from(value: &CommonCustomDataV3) -> Self {
        Self {
            coordinates: value.coordinates.into(),
            world_rotation: value.world_rotation.into(),
            local_rotation: value.local_rotation.into(),
            scale: value.scale.into(),
            custom_njs: value.note_jump_movement_speed.into(),
            custom_njs_offset: value.note_jump_start_beat_offset.into(),
            uninteractable: value.uninteractable.into(),
            color: value.color.into(),
            animation: value.animation.clone(),
        }
    }
}

impl From<&CustomNoteDataV3> for NoteCustomData {
    fn from(value: &CustomNoteDataV3) -> Self {
        Self {
            base: (&value.base).into(),
            disable_note_gravity: value.disable_note_gravity.into(),
            disable_note_look: value.disable_note_look.into(),
            spawn_effect: Default::default(),
            disable_debris: Default::default(),
        }
    }
}

impl From<&CustomBombDataV3> for BombCustomData {
    fn from(value: &CustomBombDataV3) -> Self {
        Self {
            base: (&value.base).into(),
            disable_note_gravity: value.disable_note_gravity.into(),
        }
    }
}

impl From<&CustomObstacleDataV3> for ObstacleCustomData {
    fn from(value: &CustomObstacleDataV3) -> Self {
        Self {
            base: (&value.base).into(),
            size: value.size.into(),
        }
    }
}

impl From<&CustomChainDataV3> for ChainCustomData {
    fn from(value: &CustomChainDataV3) -> Self {
        Self {
            base: (&value.base).into(),
            disable_note_gravity: value.disable_note_gravity.into(),
            disable_note_look: value.disable_note_look.into(),
            spawn_effect: value.spawn_effect.into(),
            disable_debris: value.disable_debris.into(),
            tail_coordinates: value.tail_coordinates.into(),
        }
    }
}


