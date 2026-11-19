use crate::vec_or_string;
use std::ops::Not;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::custom_data::*;

#[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CommonCustomDataV2 {
    #[cfg(feature = "tracks")]
    #[serde(rename = "_track")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(with = "vec_or_string")]
    pub track: Vec<String>,

    #[cfg(feature = "noodle")]
    #[serde(rename = "_position")]
    #[serde(skip_serializing_if = "OptVec2::is_none")]
    pub position: OptVec2,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_rotation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<RotationVec3>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_localRotation")]
    #[serde(skip_serializing_if = "OptVec3::is_none")]
    pub local_rotation: OptVec3,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_scale")]
    #[serde(skip_serializing_if = "OptVec3::is_none")]
    pub scale: OptVec3,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_noteJumpMovementSpeed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_jump_movement_speed: Option<f32>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_noteJumpStartBeatOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_jump_start_beat_offset: Option<f32>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_uninteractable")]
    #[serde(skip_serializing_if = "Not::not")]
    pub uninteractable: bool,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_animation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<crate::noodle_v2::AnimationDataV2>,

    #[cfg(feature = "chroma")]
    #[serde(rename = "_color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorVec4>,

}

#[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
impl CommonCustomDataV2 {
    fn is_empty(&self) -> bool {
        let mut some = false;

        #[cfg(feature = "tracks")]
        {
            some |= !self.track.is_empty();
        }

        #[cfg(feature = "noodle")]
        {
            some |= self.position.is_some()
                || self.rotation.is_some()
                || self.local_rotation.is_some()
                || self.scale.is_some()
                || self.note_jump_movement_speed.is_some()
                || self.note_jump_start_beat_offset.is_some()
                || self.uninteractable
                || self.animation.is_some();
        }

        #[cfg(feature = "chroma")]
        {
            some |= self.color.is_some();
        }

        !some
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomNoteDataV2 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV2,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_flip")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flip: Option<[f32; 2]>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableNoteGravity")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_note_gravity: bool,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableNoteLook")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_note_look: bool,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableBadCutDirection")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_bad_cut_direction: bool,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableBadCutSpeed")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_bad_cut_speed: bool,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableBadCutSaberType")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_bad_cut_saber_type: bool,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_link")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

impl CustomNoteDataV2 {
    pub fn is_empty(&self) -> bool {
        let mut some = !self.extra.as_ref().is_none_or(|v| v.is_empty());
        #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
        {
            some |= !self.base.is_empty();
        }
        #[cfg(feature = "noodle")]
        {
            some |= self.flip.is_some()
                || self.disable_note_gravity
                || self.disable_note_look
                || self.disable_bad_cut_direction
                || self.disable_bad_cut_speed
                || self.disable_bad_cut_saber_type
                || self.link.is_some();
        }

        !some
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CustomBombDataV2 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV2,

    #[cfg(feature = "noodle")]
    #[serde(rename = "_flip")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flip: Option<[f32; 2]>,
    #[cfg(feature = "noodle")]
    #[serde(rename = "_disableNoteGravity")]
    #[serde(default, skip_serializing_if = "Not::not")]
    pub disable_note_gravity: bool,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}
impl CustomBombDataV2 {
    pub fn is_empty(&self) -> bool {
        let mut some = !self.extra.as_ref().is_none_or(|v| v.is_empty());
        #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
        {
            some |= !self.base.is_empty();
        }
        !some
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CustomObstacleDataV2 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV2,

    #[cfg(feature = "noodle")]
    #[serde(rename = "_scale")]
    #[serde(default, skip_serializing_if = "OptVec3::is_none")]
    pub size: OptVec3,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}
impl CustomObstacleDataV2 {
    pub fn is_empty(&self) -> bool {
        let mut some = !self.extra.as_ref().is_none_or(|v| v.is_empty());
        #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
        {
            some |= !self.base.is_empty();
        }
        #[cfg(feature = "noodle")]
        {
            some |= self.size.is_some();
        }
        !some
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CustomArcDataV2 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV2,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}
impl CustomArcDataV2 {
    pub fn is_empty(&self) -> bool {
        let mut some = !self.extra.as_ref().is_none_or(|v| v.is_empty());
        #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
        {
            some |= !self.base.is_empty();
        }
        !some
    }
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
