use glam::Vec4;
use num_traits::Num;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display};
use std::hint::unreachable_unchecked;

use crate::easing::Easing;

#[cfg(not(any(feature = "v2", feature = "v3", feature = "v4")))]
compile_error!("You need v2, v3, and/or v4 enabled as a feature");

pub mod easing;

#[cfg(feature = "beatcraft")]
pub mod beatcraft;

#[cfg(feature = "v2")]
pub mod v2;
#[cfg(feature = "v3")]
pub mod v3;
#[cfg(feature = "v4")]
pub mod v4;

#[cfg(any(feature = "v2", feature = "v3"))]
pub mod v2_v3;

#[cfg(any(feature = "v2", feature = "v3"))]
pub mod info_v2;
#[cfg(feature = "v4")]
pub mod info_v4;

#[cfg(feature = "custom_data")]
pub mod custom_data;
#[cfg(all(any(feature = "v2", feature = "v3"), feature = "custom_data"))]
pub mod custom_info_v2;
#[cfg(all(feature = "v2", feature = "custom_data"))]
pub mod custom_v2;
#[cfg(all(feature = "v3", feature = "custom_data"))]
pub mod custom_v3;

#[cfg(all(any(feature = "v2", feature = "v3"), feature = "settings_setter"))]
pub mod settings_v2;

#[cfg(all(feature = "v2", feature = "noodle"))]
pub mod noodle_v2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VersionClass {
    #[cfg(any(feature = "v2", feature = "v3"))]
    V2,
    #[cfg(feature = "v3")]
    V3,
    #[cfg(feature = "v4")]
    V4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapVersion {
    #[cfg(feature = "v2")]
    #[serde(rename = "2.0.0")]
    V2_0_0,
    #[cfg(feature = "v2")]
    #[serde(rename = "2.2.0")]
    V2_2_0,
    #[cfg(feature = "v2")]
    #[serde(rename = "2.4.0")]
    V2_4_0,
    #[cfg(feature = "v2")]
    #[serde(rename = "2.5.0")]
    V2_5_0,
    #[cfg(feature = "v2")]
    #[serde(rename = "2.6.0")]
    V2_6_0,

    #[cfg(feature = "v3")]
    #[serde(rename = "3.0.0")]
    V3_0_0,
    #[cfg(feature = "v3")]
    #[serde(rename = "3.1.0")]
    V3_1_0,
    #[cfg(feature = "v3")]
    #[serde(rename = "3.2.0")]
    V3_2_0,
    #[cfg(feature = "v3")]
    #[serde(rename = "3.3.0")]
    V3_3_0,

    #[cfg(feature = "v4")]
    #[serde(rename = "4.0.0")]
    V4_0_0,
    #[cfg(feature = "v4")]
    #[serde(rename = "4.1.0")]
    V4_1_0,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InfoVersion {
    #[cfg(any(feature = "v2", feature = "v3"))]
    #[serde(rename = "2.0.0")]
    V2_0_0,
    #[cfg(any(feature = "v2", feature = "v3"))]
    #[serde(rename = "2.1.0")]
    V2_1_0,

    #[cfg(feature = "v4")]
    #[serde(rename = "4.0.0")]
    V4_0_0,
    #[cfg(feature = "v4")]
    #[serde(rename = "4.0.1")]
    V4_0_1,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapCharacteristic {
    Standard,
    NoArrows,
    OneSaber,
    #[serde(rename = "360Degree")]
    Degree360,
    #[serde(rename = "90Degree")]
    Degree90,
    Legacy,

    Lightshow,
    Lawless,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
    ExpertPlus,

    #[serde(untagged)]
    Unknown(String),
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum Color {
    #[default]
    Red = 0,
    Blue = 1,
}
convert_u8! { Color: 0 | 1 }
impl Color {
    pub fn is_red(&self) -> bool {
        *self == Self::Red
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RGBAColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum CutDirection {
    #[default]
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    UpLeft = 4,
    UpRight = 5,
    DownLeft = 6,
    DownRight = 7,
    Dot = 8,
}
convert_u8! { CutDirection: 0..=8 }
impl CutDirection {
    pub fn is_default(&self) -> bool {
        *self == Self::Up
    }
}

#[inline(always)]
pub(crate) fn is_value_i<const N: i8>(v: &(impl Num + From<i8>)) -> bool {
    *v == N.into()
}

#[inline(always)]
pub(crate) fn is_value_u<const N: u8>(v: &(impl Num + From<u8>)) -> bool {
    *v == N.into()
}

#[inline(always)]
pub(crate) fn is_value_f<const N: u32>(v: &f32) -> bool {
    *v == f32::from_bits(N)
}

// pub(crate) fn default_i<T: Num + From<i8>, const N: i8>() -> T {
//     N.into()
// }

#[inline(always)]
pub(crate) fn default_u<T: Num + From<u8>, const N: u8>() -> T {
    N.into()
}

#[inline(always)]
pub(crate) fn default_f<const N: u32>() -> f32 {
    f32::from_bits(N)
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum ArcMidAnchorMode {
    #[default]
    Straight = 0,
    Clockwise = 1,
    CounterClockwise = 2,
}
convert_u8! { ArcMidAnchorMode: 0..=2 }
impl ArcMidAnchorMode {
    pub fn is_default(&self) -> bool {
        *self == Self::Straight
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum SpawnRotationExecutionTime {
    LegacyEarly = 14,
    LegacyLate = 15,
}
convert_u8! { SpawnRotationExecutionTime: 14 | 15 }
impl SpawnRotationExecutionTime {
    pub fn is_early(&self) -> bool {
        matches!(self, Self::LegacyEarly)
    }
    pub fn is_late(&self) -> bool {
        matches!(self, Self::LegacyLate)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum SpawnRotationAngle {
    CCW60 = 0,
    CCW45 = 1,
    CCW30 = 2,
    CCW15 = 3,
    CW15 = 4,
    CW30 = 5,
    CW45 = 6,
    CW60 = 7,
}
impl TryFrom<u8> for SpawnRotationAngle {
    type Error = BeatmapDataError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0..=7 => unsafe { std::mem::transmute::<u8, Self>(value) },
            _ => {
                return Err(BeatmapDataError::ToEnum {
                    enum_name: stringify!(SpawnRotationAngle),
                    val: value as i32,
                });
            }
        })
    }
}
impl From<SpawnRotationAngle> for u8 {
    fn from(value: SpawnRotationAngle) -> Self {
        unsafe { std::mem::transmute::<SpawnRotationAngle, u8>(value) }
    }
}
impl SpawnRotationAngle {
    pub fn get_degrees(&self) -> i32 {
        match self {
            SpawnRotationAngle::CCW60 => -60,
            SpawnRotationAngle::CCW45 => -45,
            SpawnRotationAngle::CCW30 => -30,
            SpawnRotationAngle::CCW15 => -15,
            SpawnRotationAngle::CW15 => 15,
            SpawnRotationAngle::CW30 => 30,
            SpawnRotationAngle::CW45 => 45,
            SpawnRotationAngle::CW60 => 60,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BpmRegion {
    pub start_sample: usize,
    pub end_sample: usize,
    pub start_beat: f32,
    pub end_beat: f32,
}

// Implementations

impl MapCharacteristic {
    pub fn display_name(&self) -> &str {
        match self {
            MapCharacteristic::Standard => "Standard",
            MapCharacteristic::NoArrows => "NoArrows",
            MapCharacteristic::OneSaber => "OneSaber",
            MapCharacteristic::Degree360 => "360Degree",
            MapCharacteristic::Degree90 => "90Degree",
            MapCharacteristic::Legacy => "Legacy",
            MapCharacteristic::Lightshow => "Lightshow",
            MapCharacteristic::Lawless => "Lawless",
            MapCharacteristic::Unknown(s) => s.as_str(),
        }
    }
}

impl Display for MapCharacteristic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl PartialEq<&str> for MapCharacteristic {
    fn eq(&self, other: &&str) -> bool {
        self.display_name() == *other
    }
}

impl PartialEq<MapCharacteristic> for &str {
    fn eq(&self, other: &MapCharacteristic) -> bool {
        other.eq(self)
    }
}

impl MapDifficulty {
    pub fn display_name(&self) -> &str {
        match self {
            MapDifficulty::Easy => "Easy",
            MapDifficulty::Normal => "Normal",
            MapDifficulty::Hard => "Hard",
            MapDifficulty::Expert => "Expert",
            MapDifficulty::ExpertPlus => "ExpertPlus",
            MapDifficulty::Unknown(s) => s.as_str(),
        }
    }
}

impl Display for MapDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl PartialEq<&str> for MapDifficulty {
    fn eq(&self, other: &&str) -> bool {
        self.display_name() == *other
    }
}

impl PartialEq<MapDifficulty> for &str {
    fn eq(&self, other: &MapDifficulty) -> bool {
        other.eq(self)
    }
}

impl MapVersion {
    pub fn classifier(&self) -> VersionClass {
        #[allow(clippy::single_match, unreachable_patterns)]
        match self {
            #[cfg(feature = "v2")]
            MapVersion::V2_0_0
            | MapVersion::V2_2_0
            | MapVersion::V2_4_0
            | MapVersion::V2_5_0
            | MapVersion::V2_6_0 => VersionClass::V2,
            #[cfg(feature = "v3")]
            MapVersion::V3_0_0 | MapVersion::V3_1_0 | MapVersion::V3_2_0 | MapVersion::V3_3_0 => {
                VersionClass::V3
            }
            #[cfg(feature = "v4")]
            MapVersion::V4_0_0 | MapVersion::V4_1_0 => VersionClass::V4,
            _ => unreachable!("MapVersion classifier should match"),
        }
    }
}

impl PartialEq<VersionClass> for MapVersion {
    fn eq(&self, other: &VersionClass) -> bool {
        self.classifier() == *other
    }
}
impl PartialEq<MapVersion> for VersionClass {
    fn eq(&self, other: &MapVersion) -> bool {
        other.eq(self)
    }
}

impl InfoVersion {
    pub fn classifier(&self) -> VersionClass {
        match self {
            #[cfg(any(feature = "v2", feature = "v3"))]
            InfoVersion::V2_0_0 | InfoVersion::V2_1_0 => VersionClass::V2,
            #[cfg(feature = "v4")]
            InfoVersion::V4_0_0 | InfoVersion::V4_0_1 => VersionClass::V4,
            _ => unreachable!(),
        }
    }
}

impl PartialEq<VersionClass> for InfoVersion {
    fn eq(&self, other: &VersionClass) -> bool {
        self.classifier() == *other
    }
}
impl PartialEq<InfoVersion> for VersionClass {
    fn eq(&self, other: &InfoVersion) -> bool {
        other.eq(self)
    }
}

impl VersionClass {
    pub fn as_map_version(&self) -> MapVersion {
        #[allow(clippy::single_match, unreachable_patterns)]
        match self {
            #[cfg(feature = "v2")]
            VersionClass::V2 => MapVersion::V2_6_0,
            #[cfg(all(feature = "v3", not(feature = "v2")))]
            VersionClass::V2 => MapVersion::V3_3_0,
            #[cfg(feature = "v3")]
            VersionClass::V3 => MapVersion::V3_3_0,
            #[cfg(feature = "v4")]
            VersionClass::V4 => MapVersion::V4_1_0,
            _ => unreachable!(),
        }
    }

    pub fn as_info_version(&self) -> InfoVersion {
        #[allow(clippy::single_match, unreachable_patterns)]
        match self {
            #[cfg(any(feature = "v2", feature = "v3"))]
            VersionClass::V2 => InfoVersion::V2_1_0,
            #[cfg(feature = "v3")]
            VersionClass::V3 => InfoVersion::V2_1_0,
            #[cfg(feature = "v4")]
            VersionClass::V4 => InfoVersion::V4_0_1,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum InfoFile {
    #[cfg(any(feature = "v2", feature = "v3"))]
    V2(info_v2::InfoV2),
    #[cfg(feature = "v4")]
    V4(info_v4::InfoV4),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum BeatmapFile {
    #[cfg(feature = "v2")]
    V2(v2::BeatmapFileV2),
    #[cfg(feature = "v3")]
    V3(v3::BeatmapFileV3),
    #[cfg(feature = "v4")]
    V4(v4::BeatmapFileV4),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum AudioDataFile {
    #[cfg(any(feature = "v2", feature = "v3"))]
    V2(info_v2::AudioDataFileV2),
    #[cfg(feature = "v4")]
    V4(info_v4::AudioDataFileV4),
}

impl InfoFile {
    pub fn bpm(&self) -> f32 {
        #[allow(clippy::single_match, unreachable_patterns)]
        match self {
            #[cfg(any(feature = "v2", feature = "v3"))]
            Self::V2(v2) => v2.bpm,
            #[cfg(feature = "v4")]
            Self::V4(v4) => v4.audio.bpm,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl Color {
    pub fn to_default_color(&self) -> Vec4 {
        match self {
            Color::Red => Vec4::new(0.749, 0.184, 0.184, 1.),
            Color::Blue => Vec4::new(0.122, 0.388, 0.655, 1.),
        }
    }
}

impl AudioDataFile {
    pub fn bpm_regions(&self) -> Vec<BpmRegion> {
        #[allow(clippy::single_match, unreachable_patterns)]
        match self {
            #[cfg(any(feature = "v2", feature = "v3"))]
            AudioDataFile::V2(v2) => v2.bpm_regions.iter().map(Into::into).collect(),
            #[cfg(feature = "v4")]
            AudioDataFile::V4(v4) => v4.bpm_data.iter().map(Into::into).collect(),
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

// extra helpers

macro_rules! convert_u8 {
    ($cl:ty: $values:pat) => {
        impl TryFrom<u8> for $cl {
            type Error = crate::BeatmapDataError;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Ok(match value {
                    $values => unsafe { std::mem::transmute::<u8, Self>(value) },
                    _ => {
                        return Err(crate::BeatmapDataError::ToEnum {
                            enum_name: stringify!($cl),
                            val: value as i32,
                        });
                    }
                })
            }
        }
        impl From<$cl> for u8 {
            fn from(value: $cl) -> Self {
                unsafe { std::mem::transmute::<$cl, u8>(value) }
            }
        }
    };
}
pub(crate) use convert_u8;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Sentinel<const N: u8>;

impl<const N: u8> Default for Sentinel<N> {
    fn default() -> Self {
        Self
    }
}

impl<const N: u8> Debug for Sentinel<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Sentinel").field(&N).finish()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BeatmapDataError {
    #[error("{val} is not a valid value for {enum_name}")]
    ToEnum { enum_name: &'static str, val: i32 },
}

impl<const N: u8> Serialize for Sentinel<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(N)
    }
}

impl<'de, const N: u8> Deserialize<'de> for Sentinel<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u8::deserialize(deserializer)?;
        if val != N {
            return Err(D::Error::custom(format!(
                "expected sentinel value {N}, found {val}"
            )));
        }
        Ok(Sentinel)
    }
}

pub(crate) mod bool_u8_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(val: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*val as u8)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u8::deserialize(deserializer)?;
        match val {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(serde::de::Error::custom(format!(
                "expected 0 or 1 for bool, found {other}"
            ))),
        }
    }
}

pub(crate) mod easing_as_i8 {
    use super::Easing;
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(easing: &Easing, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i8(i8::from(*easing))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Easing, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = i8::deserialize(deserializer)?;
        Easing::try_from(val).map_err(D::Error::custom)
    }
}

// pub(crate) mod vec4_array {
//     use glam::Vec4;
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};
//
//     pub fn serialize<S>(v: &Vec4, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         if v.w == 1.0 {
//             [v.x, v.y, v.z].serialize(serializer)
//         } else {
//             [v.x, v.y, v.z, v.w].serialize(serializer)
//         }
//     }
//
//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec4, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         #[serde(untagged)]
//         enum Repr {
//             Xyz([f32; 3]),
//             Xyzw([f32; 4]),
//         }
//
//         match Repr::deserialize(deserializer)? {
//             Repr::Xyz([x, y, z]) => Ok(Vec4::new(x, y, z, 1.0)),
//             Repr::Xyzw([x, y, z, w]) => Ok(Vec4::new(x, y, z, w)),
//         }
//     }
// }

pub(crate) mod vec4_array_opt {
    use glam::Vec4;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(v: &Option<Vec4>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match v {
            Some(v) if v.w == 1.0 => Some([v.x, v.y, v.z].to_vec()).serialize(serializer),
            Some(v) => Some([v.x, v.y, v.z, v.w].to_vec()).serialize(serializer),
            None => Option::<()>::None.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec4>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Xyz([f32; 3]),
            Xyzw([f32; 4]),
        }

        match Option::<Repr>::deserialize(deserializer)? {
            None => Ok(None),
            Some(Repr::Xyz([x, y, z])) => Ok(Some(Vec4::new(x, y, z, 1.0))),
            Some(Repr::Xyzw([x, y, z, w])) => Ok(Some(Vec4::new(x, y, z, w))),
        }
    }
}
