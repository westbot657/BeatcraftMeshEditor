use crate::convert_int;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum LightEventTypeV2 {
    #[default]
    BackLasers = 0,
    RingLights = 1,
    LeftLasers = 2,
    RightLasers = 3,
    CenterLasers = 4,

    LeftExtra = 6,
    RightExtra = 7,

    BillieLeft = 10,
    BillieRight = 11,
}
convert_int! { LightEventTypeV2: 0..=4 | 6 | 7 | 10 | 11 }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum RingLightEventTypeV2 {
    Spin = 8,
    Zoom = 9,
}
convert_int! { RingLightEventTypeV2: 8 | 9 }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum SpinningLaserSideV2 {
    Left = 12,
    Right = 13,
}
convert_int! { SpinningLaserSideV2: 12 | 13 }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum HydraulicsTypeV2 {
    Lower = 16,
    Raise = 17,
}
convert_int! { HydraulicsTypeV2: 16 | 17 }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum GagaSideV2 {
    Left = 18,
    Right = 19,
}
convert_int! { GagaSideV2: 18 | 19 }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "i32", into = "u8")]
pub enum LightEventValueV2 {
    #[default]
    Off = 0,
    StaticSecondary = 1,
    FlashSecondary = 2,
    FadeSecondary = 3,
    TransitionSecondary = 4,
    StaticPrimary = 5,
    FlashPrimary = 6,
    FadePrimary = 7,
    TransitionPrimary = 8,
    StaticWhite = 9,
    FlashWhite = 10,
    FadeWhite = 11,
    TransitionWhite = 12,
}
convert_int! { LightEventValueV2: 0..=12, defaults }

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "u8")]
#[repr(u8)]
pub enum ColorBoostValueV2 {
    #[default]
    Disable = 0,
    Enable = 1,
}
convert_int! { ColorBoostValueV2 : 0 | 1, defaults }
