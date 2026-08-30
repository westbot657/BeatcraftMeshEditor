use serde::{Serialize, Deserialize};
use crate::convert_u8;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum LightEventTypeV2 {
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
convert_u8! { LightEventTypeV2: 0..=4 | 6 | 7 | 10 | 11 }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum RingLightEventTypeV2 {
    Spin = 8,
    Zoom = 9,
}
convert_u8! { RingLightEventTypeV2: 8 | 9 }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum SpinningLaserSideV2 {
    Left = 12,
    Right = 13,
}
convert_u8! { SpinningLaserSideV2: 12 | 13 }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum HydraulicsTypeV2 {
    Lower = 16,
    Raise = 17,
}
convert_u8! { HydraulicsTypeV2: 16 | 17 }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum GagaSideV2 {
    Left = 18,
    Right = 19,
}
convert_u8! { GagaSideV2: 18 | 19 }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(try_from = "u8", into = "u8")]
pub enum LightEventValueV2 {
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
convert_u8! { LightEventValueV2: 0..=12 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum ColorBoostValueV2 {
    Disable = 0,
    Enable = 1,
}
convert_u8! { ColorBoostValueV2 : 0 | 1 }

