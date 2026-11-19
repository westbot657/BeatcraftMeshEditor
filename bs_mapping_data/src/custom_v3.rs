use crate::vec_or_string;
use std::ops::{Deref, DerefMut, Not};
use serde::{Deserialize, Serialize};
use crate::custom_data::{ColorVec4, OptVec2, OptVec3, RotationVec3};

#[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CommonCustomDataV3 {
    #[cfg(feature = "tracks")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(with = "vec_or_string")]
    pub track: Vec<String>,

    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "OptVec2::is_none")]
    pub coordinates: OptVec2,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_rotation: Option<RotationVec3>,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "OptVec3::is_none")]
    pub local_rotation: OptVec3,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "OptVec3::is_none")]
    pub scale: OptVec3,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_jump_movement_speed: Option<f32>,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_jump_start_beat_offset: Option<f32>,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub uninteractable: bool,
    #[cfg(feature = "tracks")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<crate::animation_v3::AnimationDataV3>,

    #[cfg(feature = "chroma")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorVec4>,
}

#[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
impl CommonCustomDataV3 {
    pub fn is_empty(&self) -> bool {
        let mut some = false;

        #[cfg(feature = "tracks")]
        {
            some |= !self.track.is_empty();
        }

        #[cfg(feature = "noodle")]
        {
            some |= self.coordinates.is_some()
                || self.world_rotation.is_some()
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
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CustomNoteDataV3 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV3,

    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flip: Option<[f32; 2]>,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_note_gravity: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_note_look: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_bad_cut_direction: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_bad_cut_speed: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_bad_cut_saber_type: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_effect: Option<bool>,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_debris: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

impl CustomNoteDataV3 {
    pub fn is_empty(&self) -> bool {
        let mut some = !self.extra.as_ref().is_none_or(|v| v.is_empty());
        #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
        {
            some |= self.base.is_empty();
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
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CustomBombDataV3 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV3,

    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flip: Option<[f32; 2]>,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_note_gravity: bool,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}
impl CustomBombDataV3 {
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
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CustomObstacleDataV3 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV3,

    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "OptVec3::is_none")]
    pub size: OptVec3,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}
impl CustomObstacleDataV3 {
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
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CustomChainDataV3 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV3,

    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_note_gravity: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_note_look: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_effect: Option<bool>,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_debris: bool,
    #[cfg(feature = "noodle")]
    #[serde(skip_serializing_if = "OptVec2::is_none")]
    pub tail_coordinates: OptVec2,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}
impl CustomChainDataV3 {
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
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CustomArcDataV3 {
    #[cfg(any(feature = "noodle", feature = "chroma", feature = "tracks"))]
    #[serde(flatten)]
    pub base: CommonCustomDataV3,

    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}
impl CustomArcDataV3 {
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
    impl Deref for CustomNoteDataV3 {
        type Target = CommonCustomDataV3;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomNoteDataV3 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl Deref for CustomBombDataV3 {
        type Target = CommonCustomDataV3;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomBombDataV3 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl Deref for CustomObstacleDataV3 {
        type Target = CommonCustomDataV3;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomObstacleDataV3 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl Deref for CustomChainDataV3 {
        type Target = CommonCustomDataV3;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomChainDataV3 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl Deref for CustomArcDataV3 {
        type Target = CommonCustomDataV3;
        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
    impl DerefMut for CustomArcDataV3 {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }
}





