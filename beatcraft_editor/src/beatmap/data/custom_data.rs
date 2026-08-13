use std::ops::{Deref, DerefMut};

use glam::{Vec2, Vec4};
use serde::{Deserialize, Serialize};

use super::noodle::AnimationDataV2;
use super::vec4_array_opt;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptVec2(pub [Option<f32>; 2]);
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptVec3(pub [Option<f32>; 3]);
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptVec4(pub [Option<f32>; 4]);

// V2

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonCustomDataV2 {
    #[serde(rename = "_noteJumpStartBeatOffset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_jump_start_beat_offset: Option<f32>,
    #[serde(rename = "_noteJumpMovementSpeed")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_jump_movement_speed: Option<f32>,
    #[serde(rename = "_rotation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<OptVec3>,
    #[serde(rename = "_localRotation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_rotation: Option<OptVec3>,
    #[serde(rename = "_coordinates")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<OptVec2>,
    #[serde(rename = "_position")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<OptVec2>,
    #[serde(rename = "_track")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(rename = "_animation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<AnimationDataV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomNoteDataV2 {
    #[serde(flatten)]
    base: CommonCustomDataV2,
    #[serde(rename = "_color")]
    #[serde(with = "vec4_array_opt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Vec4>,
    #[serde(rename = "_disableNoteLook")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_note_look: Option<bool>,
    #[serde(rename = "_disableNoteGravity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_note_gravity: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomBombDataV2 {
    #[serde(flatten)]
    base: CommonCustomDataV2,
    #[serde(rename = "_color")]
    #[serde(with = "vec4_array_opt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Vec4>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomObstacleSizeV2 {
    X([f32; 1]),
    Xy([f32; 2]),
    Xyz([f32; 3]),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomObstacleDataV2 {
    #[serde(flatten)]
    base: CommonCustomDataV2,
    #[serde(rename = "_color")]
    #[serde(with = "vec4_array_opt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Vec4>,
    #[serde(rename = "_scale")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<CustomObstacleSizeV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomArcDataV2 {
    #[serde(flatten)]
    base: CommonCustomDataV2,

}

impl Deref for CustomNoteDataV2 {
    type Target = CommonCustomDataV2;
    fn deref(&self) -> &Self::Target { &self.base }
}
impl DerefMut for CustomNoteDataV2 {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}

impl Deref for CustomBombDataV2 {
    type Target = CommonCustomDataV2;
    fn deref(&self) -> &Self::Target { &self.base }
}
impl DerefMut for CustomBombDataV2 {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}

impl Deref for CustomObstacleDataV2 {
    type Target = CommonCustomDataV2;
    fn deref(&self) -> &Self::Target { &self.base }
}
impl DerefMut for CustomObstacleDataV2 {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}

impl Deref for CustomArcDataV2 {
    type Target = CommonCustomDataV2;
    fn deref(&self) -> &Self::Target { &self.base }
}
impl DerefMut for CustomArcDataV2 {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}


// V3




