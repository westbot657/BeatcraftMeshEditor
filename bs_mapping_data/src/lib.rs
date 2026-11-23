use glam::Vec4;
use num_traits::Num;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{value::StrDeserializer, IntoDeserializer};
use std::fmt::{Debug, Display};
use std::hint::unreachable_unchecked;
use std::{fs, io};
use std::path::Path;
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

#[cfg(any(feature = "v2", feature = "v3", feature = "v4"))]
pub mod v2_v3_v4;

#[cfg(any(feature = "v3", feature = "v4"))]
pub mod v3_v4;

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

#[cfg(all(any(feature = "v2", feature = "v3"), feature = "tracks"))]
pub mod animation;
#[cfg(all(feature = "v2", feature = "tracks"))]
pub mod animation_v2;
#[cfg(all(feature = "v3", feature = "tracks"))]
pub mod animation_v3;

#[cfg(test)]
pub mod tests;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum VersionClass {
    V2,
    V3,
    V4,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapVersion {
    #[serde(rename = "2.0.0")]
    V2_0_0,
    #[serde(rename = "2.2.0")]
    V2_2_0,
    #[serde(rename = "2.4.0")]
    V2_4_0,
    #[serde(rename = "2.5.0")]
    V2_5_0,
    #[serde(rename = "2.6.0")]
    V2_6_0,

    #[serde(rename = "3.0.0")]
    V3_0_0,
    #[serde(rename = "3.1.0")]
    V3_1_0,
    #[serde(rename = "3.2.0")]
    V3_2_0,
    #[serde(rename = "3.3.0")]
    V3_3_0,

    #[serde(rename = "4.0.0")]
    V4_0_0,
    #[serde(rename = "4.1.0")]
    V4_1_0,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InfoVersion {
    #[serde(rename = "2.0.0")]
    V2_0_0,
    #[serde(rename = "2.1.0")]
    V2_1_0,

    #[serde(rename = "4.0.0")]
    V4_0_0,
    #[serde(rename = "4.0.1")]
    V4_0_1,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightshowVersion {
    #[serde(rename = "4.0.0")]
    V4_0_0,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
    ExpertPlus,

    #[serde(untagged)]
    Unknown(String),
}

macro_rules! environment_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $data:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                #[serde(rename = $data)]
                $variant,
            )*
        }

        impl $name {
            pub fn display_name(&self) -> &'static str {
                match self {
                    $( Self::$variant => stringify!($variant), )*
                }
            }

            pub fn data_name(&self) -> &'static str {
                match self {
                    $( Self::$variant => $data, )*
                }
            }
        }
    };
}

environment_enum! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub enum EnvironmentV2 {
        Default => "DefaultEnvironment",
        Triangle => "TriangleEnvironment",
        Nice => "NiceEnvironment",
        BigMirror => "BigMirrorEnvironment",
        KDA => "KDAEnvironment",
        Monstercat => "MonstercatEnvironment",
        CrabRave => "CrabRaveEnvironment",
        Dragons => "DragonsEnvironment",
        Origins => "OriginsEnvironment",
        Panic => "PanicEnvironment",
        Rocket => "RocketEnvironment",
        GreenDay => "GreenDayEnvironment",
        GreenDayGrenade => "GreenDayGrenadeEnvironment",
        Timbaland => "TimbalandEnvironment",
        FitBeat => "FitBeatEnvironment",
        LinkinPark => "LinkinParkEnvironment",
        BTS => "BTSEnvironment",
        Kaleidoscope => "KaleidoscopeEnvironment",
        Interscope => "InterscopeEnvironment",
        Skrillex => "SkrillexEnvironment",
        Billie => "BillieEnvironment",
        Halloween => "HalloweenEnvironment",
        Gaga => "GagaEnvironment",
        Halloween2 => "Halloween2Environment",
    }
}

environment_enum! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub enum EnvironmentV3 {
        Weave => "WeaveEnvironment",
        Pyro => "PyroEnvironment",
        EDM => "EDMEnvironment",
        TheSecond => "TheSecondEnvironment",
        Lizzo => "LizzoEnvironment",
        TheWeeknd => "TheWeekndEnvironment",
        RockMixtape => "RockMixtapeEnvironment",
        Dragons2 => "Dragons2Environment",
        Panic2 => "Panic2Environment",
        Queen => "QueenEnvironment",
        LinkinPark2 => "LinkinPark2Environment",
        TheRollingStones => "TheRollingStonesEnvironment",
        Lattice => "LatticeEnvironment",
        DaftPunk => "DaftPunkEnvironment",
        HipHop => "HipHopEnvironment",
        Collider => "ColliderEnvironment",
        Britney => "BritneyEnvironment",
        Monstercat2 => "Monstercat2Environment",
        Metallica => "MetallicaEnvironment",
        Grid => "GridEnvironment",
        Coldplay => "ColdplayEnvironment",
    }
}

environment_enum! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub enum Environment360V2 {
        GlassDesert => "GlassDesertEnvironment",
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    #[serde(untagged)]
    V2(EnvironmentV2),

    #[serde(untagged)]
    V3(EnvironmentV3),

    #[serde(untagged)]
    Unknown(String),
}

impl Default for Environment {
    fn default() -> Self {
        Self::V2(EnvironmentV2::Default)
    }
}

impl Environment {
    pub fn display_name(&self) -> &str {
        match self {
            Self::V2(v2) => v2.display_name(),
            Self::V3(v3) => v3.display_name(),
            Self::Unknown(s) => s.as_str(),
        }
    }
    pub fn data_name(&self) -> &str {
        match self {
            Self::V2(v2) => v2.data_name(),
            Self::V3(v3) => v3.data_name(),
            Self::Unknown(s) => s.as_str(),
        }
    }
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
    pub fn is_v2(&self) -> bool {
        matches!(self, Self::V2(_))
    }
    pub fn is_v3(&self) -> bool {
        matches!(self, Self::V3(_))
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment360 {
    #[serde(untagged)]
    V2(Environment360V2),

    #[serde(untagged)]
    Unknown(String),
}

impl Environment360 {
    pub fn display_name(&self) -> &str {
        match self {
            Self::V2(v2) => v2.display_name(),
            Self::Unknown(s) => s.as_str(),
        }
    }
    pub fn data_name(&self) -> &str {
        match self {
            Self::V2(v2) => v2.data_name(),
            Self::Unknown(s) => s.as_str(),
        }
    }
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
    pub fn is_v2(&self) -> bool {
        matches!(self, Self::V2(_))
    }
    pub fn is_v3(&self) -> bool {
        false
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub enum AnyEnvironment {
    #[serde(untagged)]
    Standard(Environment),
    #[serde(untagged)]
    Env360(Environment360),
}

impl Default for AnyEnvironment {
    fn default() -> Self {
        Self::Standard(Default::default())
    }
}

impl AnyEnvironment {
    pub fn is_standard(&self) -> bool {
        matches!(self, Self::Standard(_))
    }
    pub fn is_360(&self) -> bool {
        matches!(self, Self::Env360(_))
    }
    pub fn is_v2(&self) -> bool {
        match self {
            Self::Standard(e) => e.is_v2(),
            Self::Env360(e) => e.is_v2(),
        }
    }
    pub fn is_v3(&self) -> bool {
        match self {
            Self::Standard(e) => e.is_v3(),
            Self::Env360(e) => e.is_v3(),
        }
    }
    pub fn is_unknown(&self) -> bool {
        match self {
            Self::Standard(e) => e.is_unknown(),
            Self::Env360(e) => e.is_unknown(),
        }
    }
    pub fn display_name(&self) -> &str {
        match self {
            Self::Standard(e) => e.display_name(),
            Self::Env360(e) => e.display_name(),
        }
    }
    pub fn data_name(&self) -> &str {
        match self {
            Self::Standard(e) => e.data_name(),
            Self::Env360(e) => e.data_name(),
        }
    }
}

impl<'de> Deserialize<'de> for AnyEnvironment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Environment::deserialize(deserializer)? {
            Environment::Unknown(name) => {
                let de: StrDeserializer<serde::de::value::Error> =
                    name.as_str().into_deserializer();

                match Environment360::deserialize(de) {
                    Ok(Environment360::Unknown(_)) | Err(_) => {
                        Ok(AnyEnvironment::Standard(Environment::Unknown(name)))
                    }
                    Ok(env360) => Ok(AnyEnvironment::Env360(env360)),
                }
            }
            known => Ok(AnyEnvironment::Standard(known)),
        }
    }
}

impl From<EnvironmentV2> for Environment {
    fn from(value: EnvironmentV2) -> Self {
        Self::V2(value)
    }
}
impl From<EnvironmentV3> for Environment {
    fn from(value: EnvironmentV3) -> Self {
        Self::V3(value)
    }
}
impl From<EnvironmentV2> for AnyEnvironment {
    fn from(value: EnvironmentV2) -> Self {
        Self::Standard(value.into())
    }
}
impl From<EnvironmentV3> for AnyEnvironment {
    fn from(value: EnvironmentV3) -> Self {
        Self::Standard(value.into())
    }
}
impl From<Environment> for AnyEnvironment {
    fn from(value: Environment) -> Self {
        Self::Standard(value)
    }
}
impl From<Environment360> for AnyEnvironment {
    fn from(value: Environment360) -> Self {
        Self::Env360(value)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum Color {
    #[default]
    Red = 0,
    Blue = 1,
}
convert_int! { Color: 0 | 1 }
impl Color {
    pub fn is_red(&self) -> bool {
        *self == Self::Red
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct RGBAColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
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
convert_int! { CutDirection: 0..=8 }
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

pub(crate) fn default_i<T: Num + From<i8>, const N: i8>() -> T {
    N.into()
}

// #[inline(always)]
// pub(crate) fn default_u<T: Num + From<u8>, const N: u8>() -> T {
//     N.into()
// }

#[inline(always)]
pub(crate) fn default_f<const N: u32>() -> f32 {
    f32::from_bits(N)
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum ArcMidAnchorMode {
    #[default]
    Straight = 0,
    Clockwise = 1,
    CounterClockwise = 2,
}
convert_int! { ArcMidAnchorMode: 0..=2 }
impl ArcMidAnchorMode {
    pub fn is_default(&self) -> bool {
        *self == Self::Straight
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum SpawnRotationExecutionTime {
    LegacyEarly = 14,
    LegacyLate = 15,
}
convert_int! { SpawnRotationExecutionTime: 14 | 15 }
impl SpawnRotationExecutionTime {
    pub fn is_early(&self) -> bool {
        matches!(self, Self::LegacyEarly)
    }
    pub fn is_late(&self) -> bool {
        matches!(self, Self::LegacyLate)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum SpawnRotationAngle {
    #[default]
    CCW60 = 0,
    CCW45 = 1,
    CCW30 = 2,
    CCW15 = 3,
    CW15 = 4,
    CW30 = 5,
    CW45 = 6,
    CW60 = 7,
}
convert_int! { SpawnRotationAngle: 0..=7, defaults }

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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone)]
pub struct BpmRegion {
    pub start_sample: usize,
    pub end_sample: usize,
    pub start_beat: f64,
    pub end_beat: f64,
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
    pub fn rank(&self) -> Option<u8> {
        match self {
            Self::Easy => Some(1),
            Self::Normal => Some(3),
            Self::Hard => Some(5),
            Self::Expert => Some(7),
            Self::ExpertPlus => Some(9),
            Self::Unknown(_) => None,
        }
    }
    pub fn display_name(&self) -> &str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Expert => "Expert",
            Self::ExpertPlus => "ExpertPlus",
            Self::Unknown(s) => s.as_str(),
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
            MapVersion::V2_0_0
            | MapVersion::V2_2_0
            | MapVersion::V2_4_0
            | MapVersion::V2_5_0
            | MapVersion::V2_6_0 => VersionClass::V2,
            MapVersion::V3_0_0 | MapVersion::V3_1_0 | MapVersion::V3_2_0 | MapVersion::V3_3_0 => {
                VersionClass::V3
            }
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
        #[allow(clippy::single_match, unreachable_patterns)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum InfoFile {
    #[cfg(any(feature = "v2", feature = "v3"))]
    V2(info_v2::InfoV2),
    #[cfg(feature = "v4")]
    V4(info_v4::InfoV4),
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum AudioDataFile {
    #[cfg(any(feature = "v2", feature = "v3"))]
    V2(info_v2::AudioDataFileV2),
    #[cfg(feature = "v4")]
    V4(info_v4::AudioDataFileV4),
}

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("{0}")]
    IOError(#[from] io::Error),
    #[error("0")]
    SerdeError(#[from] serde_json::Error),
    #[error("File not found: {0}")]
    FileNotFound(&'static str),
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
    pub fn load_from_folder(folder: &Path) -> Result<Self, DataError> {
        for path in fs::read_dir(folder)? {
            let path = path?;
            let name = path.file_name().to_string_lossy().to_lowercase();
            if name == "info.dat" {
                let data = fs::read(path.path())?;
                let info: Self = serde_json::from_slice(&data)?;
                return Ok(info)
            }
        }
        Err(DataError::FileNotFound("info.dat"))
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

macro_rules! convert_int {
    ($cl:ty: $values:pat) => {
        impl TryFrom<i32> for $cl {
            type Error = $crate::BeatmapDataError;
            fn try_from(value: i32) -> Result<Self, Self::Error> {
                Ok(match value {
                    $values => unsafe { ::std::mem::transmute::<u8, Self>(value as u8) },
                    _ => {
                        return Err($crate::BeatmapDataError::ToEnum {
                            enum_name: stringify!($cl),
                            val: value,
                        });
                    }
                })
            }
        }
        impl TryFrom<u8> for $cl {
            type Error = $crate::BeatmapDataError;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Ok(match value {
                    $values => unsafe { ::std::mem::transmute::<u8, Self>(value) },
                    _ => {
                        return Err($crate::BeatmapDataError::ToEnum {
                            enum_name: stringify!($cl),
                            val: value as i32,
                        })
                    }
                })
            }
        }
        impl From<$cl> for u8 {
            fn from(value: $cl) -> Self {
                unsafe { ::std::mem::transmute::<$cl, u8>(value) }
            }
        }
    };
    ($cl:ty: $values:pat, defaults) => {
        impl From<i32> for $cl {
            fn from(value: i32) -> Self {
                match value {
                    $values => unsafe { ::std::mem::transmute::<u8, Self>(value as u8) },
                    _ => Self::default(),
                }
            }
        }
        impl TryFrom<u8> for $cl {
            type Error = $crate::BeatmapDataError;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Ok(match value {
                    $values => unsafe { ::std::mem::transmute::<u8, Self>(value) },
                    _ => {
                        return Err($crate::BeatmapDataError::ToEnum {
                            enum_name: stringify!($cl),
                            val: value as i32,
                        })
                    }
                })
            }
        }
        impl From<$cl> for u8 {
            fn from(value: $cl) -> Self {
                unsafe { ::std::mem::transmute::<$cl, u8>(value) }
            }
        }
    };
    ($cl:ty: $values:pat, defaults: $def:expr) => {
        impl From<i32> for $cl {
            fn from(value: i32) -> Self {
                match value {
                    $values => unsafe { ::std::mem::transmute::<u8, Self>(value as u8) },
                    _ => $def,
                }
            }
        }
        impl TryFrom<u8> for $cl {
            type Error = $crate::BeatmapDataError;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Ok(match value {
                    $values => unsafe { ::std::mem::transmute::<u8, Self>(value) },
                    _ => {
                        return Err($crate::BeatmapDataError::ToEnum {
                            enum_name: stringify!($cl),
                            val: value as i32,
                        })
                    }
                })
            }
        }
        impl From<$cl> for u8 {
            fn from(value: $cl) -> Self {
                unsafe { ::std::mem::transmute::<$cl, u8>(value) }
            }
        }
    };
}
pub(crate) use convert_int;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Sentinel<const N: u8>;

impl<const N: u8> Default for Sentinel<N> {
    fn default() -> Self {
        Self
    }
}

#[cfg(feature = "debug")]
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

pub(crate) mod vec_or_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum VecOrStr {
        Str(String),
        Vec(Vec<String>),
    }

    pub fn serialize<S>(values: &[String], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match values {
            [single] => serializer.serialize_str(single),
            _ => values.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match VecOrStr::deserialize(deserializer)? {
            VecOrStr::Str(s) => vec![s],
            VecOrStr::Vec(v) => v,
        })
    }
}
