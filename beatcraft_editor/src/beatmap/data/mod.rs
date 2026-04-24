#![allow(non_snake_case)]

use std::ops::{Deref, Not};

use glam::{Quat, Vec2, Vec3, Vec4};
use num_traits::ConstZero;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(untagged)]
pub enum Color {
    V4(Vec4),
    V3(Vec3),
}

#[inline]
fn is_zero<T: PartialEq + ConstZero>(t: &T) -> bool {
    *t == T::ZERO
}

// V2 --------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointV3V2 {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointFV2 {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointQV2 {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointCV2 {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct AnimationDataV2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _position: Option<PointV3V2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _rotation: Option<PointQV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _localRotation: Option<PointQV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _localPosition: Option<PointV3V2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _definitePosition: Option<PointV3V2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _scale: Option<PointV3V2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _dissolve: Option<PointFV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _dissolveArrow: Option<PointFV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _time: Option<PointFV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _color: Option<PointCV2>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CustomDataV2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _noteJumpStartBeatOffset: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _noteJumpMovementSpeed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _rotation: Option<Quat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _localRotation: Option<Quat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _coordinates: Option<Vec2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _position: Option<Vec2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _animation: Option<AnimationDataV2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeatmapObjectDataV2 {
    pub _time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameplayObjectDataV2 {
    #[serde(flatten)]
    obj: BeatmapObjectDataV2,
    pub _lineIndex: f32,
    pub _lineLayer: f32,
}

impl Deref for GameplayObjectDataV2 {
    type Target = BeatmapObjectDataV2;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

// Color Note

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ColorNoteCustomDataV2 {
    #[serde(flatten)]
    parent: CustomDataV2,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _color: Option<Color>,
    #[serde(skip_serializing_if = "Not::not")]
    pub _disableNoteLook: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub _disableNoteGravity: bool,
}

impl Deref for ColorNoteCustomDataV2 {
    type Target = CustomDataV2;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColorNoteDataV2 {
    #[serde(flatten)]
    obj: GameplayObjectDataV2,
    pub _cutDirection: u32,
    pub _type: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _customData: Option<ColorNoteCustomDataV2>,
}

impl Deref for ColorNoteDataV2 {
    type Target = GameplayObjectDataV2;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

// Bomb Note

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BombNoteCustomDataV2 {
    #[serde(flatten)]
    parent: CustomDataV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _color: Option<Color>
}

impl Deref for BombNoteCustomDataV2 {
    type Target = CustomDataV2;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BombNoteDataV2 {
    #[serde(flatten)]
    obj: GameplayObjectDataV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _customData: Option<BombNoteCustomDataV2>,
}

impl Deref for BombNoteDataV2 {
    type Target = GameplayObjectDataV2;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

// Obstacle
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum CustomObstacleScale {
    Bounds([f32; 3]),
    Rect([f32; 2]),
    Width([f32; 1]),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ObstacleCustomDataV2 {
    #[serde(flatten)]
    parent: CustomDataV2,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _scale: Option<CustomObstacleScale>,
}

impl Deref for ObstacleCustomDataV2 {
    type Target = CustomDataV2;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObstacleDataV2 {
    #[serde(flatten)]
    obj: BeatmapObjectDataV2,
    pub _duration: f32,
    pub _lineIndex: f32,
    pub _width: f32,
    pub _type: u32,
}

impl Deref for ObstacleDataV2 {
    type Target = BeatmapObjectDataV2;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

// Arc
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArcDataV2 {
    pub _colorType: u32,
    pub _headTime: f32,
    pub _headLineIndex: f32,
    pub _headLineLayer: f32,
    pub _tailLineIndex: f32,
    pub _tailLineLayer: f32,
    pub _headCutDirection: u32,
    pub _tailCutDirection: u32,
    pub _headControlPointLengthMultiplier: f32,
    pub _tailControlPointLengthMultiplier: f32,
    pub _sliderMidAnchorMode: u32,
}

// V3 --------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointV3V3 {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointFV3 {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointQV3 {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointCV3 {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct AnimationDataV3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offsetPosition: Option<PointV3V3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offsetWorldRotation: Option<PointQV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localRotation: Option<PointQV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localPosition: Option<PointV3V3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definitePosition: Option<PointV3V3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PointQV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<PointQV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<PointV3V3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve: Option<PointFV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolveArrow: Option<PointFV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactable: Option<PointFV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<PointFV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<PointCV3>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct BeatmapObjectV3 {
    #[serde(skip_serializing_if = "is_zero")]
    pub b: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GameplayObjectV3 {
    #[serde(flatten)]
    obj: BeatmapObjectV3,
    #[serde(skip_serializing_if = "is_zero")]
    pub x: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub y: f32,
}

impl Deref for GameplayObjectV3 {
    type Target = BeatmapObjectV3;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

// Color Note
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ColorNoteCustomDataV3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Not::not")]
    pub disableNoteLook: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub disableNoteGravity: bool,
}


















// V4 --------------------------------------------------------



