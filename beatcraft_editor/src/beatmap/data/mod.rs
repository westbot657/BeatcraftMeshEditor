#![allow(non_snake_case)]

use std::ops::{Deref, Not};

use glam::{Quat, Vec2, Vec3, Vec4};
use num_traits::{ConstOne, ConstZero};
use serde::{Deserialize, Serialize};

use crate::easing::Easing;

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

#[inline]
fn is_one<T: PartialEq + ConstOne>(t: &T) -> bool {
    *t == T::ONE
}

// V2 --------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Spline {
    #[serde(rename = "splineCatmullRom")]
    SplineCatmullRom
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointV3InnerV2(
    f32, f32, f32,  f32,
    Option<Easing>, Option<Spline>,
);


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PointV3V2 {
    Value(Vec3),
    Lookup(String),
    Inline(Vec<PointV3InnerV2>)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointFV2();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointQV2();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointCV2();

macro_rules! deref_to {
    ( [ $( $typ:path ),* ].$field:tt: $reft:path ) => {
        $(impl Deref for $typ {
            type Target = $reft;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        })*
    };
}

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BeatmapObjectDataV2 {
    pub _time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ColorNoteDataV2 {
    #[serde(flatten)]
    obj: GameplayObjectDataV2,
    pub _cutDirection: u32,
    pub _type: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _customData: Option<ColorNoteCustomDataV2>,
}

// Bomb Note

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BombNoteCustomDataV2 {
    #[serde(flatten)]
    parent: CustomDataV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _color: Option<Color>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BombNoteDataV2 {
    #[serde(flatten)]
    obj: GameplayObjectDataV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _customData: Option<BombNoteCustomDataV2>,
}

// Obstacle
#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

// Chain Note does not exist in V2

deref_to! { [
    ColorNoteDataV2,
    BombNoteDataV2
].obj: GameplayObjectDataV2 }
deref_to! { [
    ColorNoteCustomDataV2,
    BombNoteCustomDataV2,
    ObstacleCustomDataV2
].parent: CustomDataV2 }

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
pub struct BeatmapObjectDataV3 {
    #[serde(skip_serializing_if = "is_zero")]
    pub b: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GameplayObjectDataV3 {
    #[serde(flatten)]
    obj: BeatmapObjectDataV3,
    #[serde(skip_serializing_if = "is_zero")]
    pub x: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub y: f32,
}

impl Deref for GameplayObjectDataV3 {
    type Target = BeatmapObjectDataV3;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CustomDataV3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noteJumpStartBeatOffset: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noteJumpMovementSpeed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worldRotation: Option<Quat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localRotation: Option<Quat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<(Option<f32>, Option<f32>)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<(Option<f32>, Option<f32>)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<AnimationDataV3>,
}

// Color Note
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ColorNoteCustomDataV3 {
    #[serde(flatten)]
    pub parent: CustomDataV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Not::not")]
    pub disableNoteLook: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub disableNoteGravity: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ColorNoteDataV3 {
    #[serde(flatten)]
    obj: GameplayObjectDataV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customData: Option<ColorNoteCustomDataV3>,
}

// Bomb Note
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct BombNoteCustomDataV3 {
    #[serde(flatten)]
    parent: CustomDataV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct BombNoteDataV3 {
    #[serde(flatten)]
    obj: GameplayObjectDataV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customData: Option<BombNoteCustomDataV3>,
}

// Obstacle
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ObstacleCustomDataV3 {
    #[serde(flatten)]
    parent: CustomDataV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<CustomObstacleScale>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ObstacleDataV3 {
    parent: BeatmapObjectDataV3,
    /// duration
    #[serde(skip_serializing_if = "is_zero")]
    pub d: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub x: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub y: f32,
    /// width
    #[serde(skip_serializing_if = "is_zero")]
    pub w: f32,
    /// height
    #[serde(skip_serializing_if = "is_zero")]
    pub h: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customData: Option<ObstacleCustomDataV3>,
}

impl Deref for ObstacleDataV3 {
    type Target = BeatmapObjectDataV3;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}


// Arc
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ArcDataV3 {
    /// Note Type
    #[serde(skip_serializing_if = "is_zero")]
    pub c: u32,
    /// Beat
    #[serde(skip_serializing_if = "is_zero")]
    pub b: f32,
    /// Tail Beat
    #[serde(skip_serializing_if = "is_zero")]
    pub tb: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub x: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub y: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub tx: f32,
    #[serde(skip_serializing_if = "is_zero")]
    pub ty: f32,
    /// Head Cut Direction
    #[serde(skip_serializing_if = "is_zero")]
    pub d: u32,
    /// Tail Cut Direction
    #[serde(skip_serializing_if = "is_zero")]
    pub tc: u32,
    /// Head Magnitude
    #[serde(skip_serializing_if = "is_zero")]
    pub mu: f32,
    /// Tail Magnitude
    #[serde(skip_serializing_if = "is_zero")]
    pub tmu: f32,
    /// Mid Anchor Mode
    #[serde(skip_serializing_if = "is_zero")]
    pub m: u32,
}


// Chain Note
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ChainNoteDataV3 {
    #[serde(flatten)]
    obj: GameplayObjectDataV3,
    /// Cut Direction
    #[serde(skip_serializing_if = "is_zero")]
    pub d: u32,
    /// Note Type
    #[serde(skip_serializing_if = "is_zero")]
    pub c: u32,
    /// Tail Beat
    #[serde(skip_serializing_if = "is_zero")]
    pub tb: f32,
    /// Tail X
    #[serde(skip_serializing_if = "is_zero")]
    pub tx: f32,
    /// Tail Y
    #[serde(skip_serializing_if = "is_zero")]
    pub ty: f32,
    /// Slice Count
    #[serde(skip_serializing_if = "is_zero")]
    pub sc: u32,
    /// Squish Factor
    #[serde(skip_serializing_if = "is_one")]
    pub s: f32,
}

impl Default for ChainNoteDataV3 {
    fn default() -> Self {
        Self {
            obj: Default::default(),
            d: Default::default(),
            c: Default::default(),
            tb: Default::default(),
            tx: Default::default(),
            ty: Default::default(),
            sc: Default::default(),
            s: 1.,
        }
    }
}

deref_to! { [
    ColorNoteDataV3,
    BombNoteDataV3,
    ChainNoteDataV3
].obj: GameplayObjectDataV3 }
deref_to! { [
    ColorNoteCustomDataV3,
    BombNoteCustomDataV3,
    ObstacleCustomDataV3
].parent: CustomDataV3 }

// V4 --------------------------------------------------------



















