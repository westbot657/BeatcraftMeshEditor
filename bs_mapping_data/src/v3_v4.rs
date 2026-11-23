use crate::is_value_i;
use crate::default_i;
use crate::bool_u8_serde;
use crate::convert_int;
use crate::is_value_f;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum IndexFilterType {
    #[default]
    Division = 1,
    StepAndOffset = 2,
}
convert_int! { IndexFilterType : 1 | 2, defaults }
impl IndexFilterType {
    pub fn is_default(&self) -> bool {
        *self == Self::Division
    }
}

bitflags! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    pub struct RandomizationBehavior: u8 {
        const NoRandom       = 0;
        const KeepOrder      = 1;
        const RandomElements = 2;
    }

    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    pub struct LimitBehavior: u8 {
        const None         = 0;
        const Duration     = 1;
        const Distribution = 2;
    }

}

macro_rules! flags_as_u8 {
    ( $val:ty ) => {
        impl From<u8> for $val {
            fn from(value: u8) -> Self {
                Self::from_bits_truncate(value)
            }
        }
        impl From<$val> for u8 {
            fn from(value: $val) -> Self {
                value.bits()
            }
        }
    };
}

impl Default for RandomizationBehavior {
    fn default() -> Self {
        Self::empty()
    }
}
impl Default for LimitBehavior {
    fn default() -> Self {
        Self::empty()
    }
}

flags_as_u8! { RandomizationBehavior }
flags_as_u8! { LimitBehavior }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum DistributionType {
    #[default]
    Wave = 1,
    Step = 2,
}
convert_int! { DistributionType : 1 | 2, defaults }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum TransitionType {
    #[default]
    Transition = 0,
    Extend = 1,
}
convert_int! { TransitionType : 1 | 2, defaults }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum LightEventColor {
    #[default]
    Primary = 0,
    Secondary = 1,
    White = 2,
}
convert_int! { LightEventColor : 0..=2, defaults }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum EventAxis {
    #[default]
    X = 0,
    Y = 1,
    Z = 2,
}
convert_int! { EventAxis : 0..=2, defaults }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum RotationDirection {
    #[default]
    Automatic = 0,
    Clockwise = 1,
    CounterClockwise = 2,
}
convert_int! { RotationDirection : 0..=2, defaults }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexFilterV3 {
    #[serde(rename = "c")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunks: Option<u32>,
    #[serde(rename = "f")]
    #[serde(default, skip_serializing_if = "IndexFilterType::is_default")]
    pub typ: IndexFilterType,
    #[serde(rename = "p")]
    #[serde(default)]
    pub param_0: u32,
    #[serde(rename = "t")]
    #[serde(default)]
    pub param_1: u32,
    #[serde(rename = "r")]
    #[serde(with = "bool_u8_serde")]
    #[serde(default)]
    pub reverse: bool,
    #[serde(rename = "n")]
    #[serde(default, skip_serializing_if = "RandomizationBehavior::is_empty")]
    pub randomization_behavior: RandomizationBehavior,
    #[serde(rename = "s")]
    #[serde(default = "default_i::<_, 0>", skip_serializing_if = "is_value_i::<0>")]
    pub randomization_seed: i64,
    #[serde(rename = "l")]
    #[serde(default, skip_serializing_if = "is_value_f::<{ 0f32.to_bits() }>")]
    pub limit_percentage: f32,
    #[serde(rename = "d")]
    #[serde(default, skip_serializing_if = "LimitBehavior::is_empty")]
    pub limit_behavior: LimitBehavior,
}
