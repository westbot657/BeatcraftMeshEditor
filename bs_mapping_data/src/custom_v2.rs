use std::ops::{Deref, DerefMut};

use serde::{Serialize, Deserialize};
use glam::Vec4;

use crate::custom_data::*;
use super::vec4_array_opt;

#[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonCustomDataV2 {
    #[cfg(feature = "noodle")]
    #[serde(rename = "_noteJumpStartBeatOffset")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_jump_start_beat_offset: Option<f32>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_noteJumpMovementSpeed")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_jump_movement_speed: Option<f32>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_rotation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<OptVec3>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_localRotation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_rotation: Option<OptVec3>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_coordinates")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<OptVec2>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_position")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<OptVec2>,
    #[cfg(feature = "tracks")]
    #[serde(rename = "_track")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_animation")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<crate::noodle_v2::AnimationDataV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomNoteDataV2 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    base: CommonCustomDataV2,
    #[cfg(feature = "chroma")]
    #[serde(rename = "_color")]
    #[serde(with = "vec4_array_opt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Vec4>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableNoteLook")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_note_look: Option<bool>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableNoteGravity")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_note_gravity: Option<bool>,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomBombDataV2 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    base: CommonCustomDataV2,
    #[cfg(feature = "chroma")]
    #[serde(rename = "_color")]
    #[serde(with = "vec4_array_opt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Vec4>,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
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
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    base: CommonCustomDataV2,
    #[cfg(feature = "chroma")]
    #[serde(rename = "_color")]
    #[serde(with = "vec4_array_opt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Vec4>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_scale")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<CustomObstacleSizeV2>,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomArcDataV2 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    base: CommonCustomDataV2,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

#[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
mod deref_impls {
    use super::*;
    impl Deref for CustomNoteDataV2 {
        type Target = CommonCustomDataV2;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomNoteDataV2 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl Deref for CustomBombDataV2 {
        type Target = CommonCustomDataV2;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomBombDataV2 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl Deref for CustomObstacleDataV2 {
        type Target = CommonCustomDataV2;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomObstacleDataV2 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl Deref for CustomArcDataV2 {
        type Target = CommonCustomDataV2;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomArcDataV2 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }
}
