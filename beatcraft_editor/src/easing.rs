use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use crate::beatmap;

macro_rules! easing {
    (
        $x:tt
        $( $display:literal : $sentinel:literal : $name:tt => $func:expr )*
    ) => {
        #[allow(non_camel_case_types)]
        #[derive(Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq, Copy, Clone)]
        #[repr(i8)]
        pub enum Easing {
            #[default] $(
            $name = $sentinel,
        )*}

        impl Easing {
            pub fn apply(&self, $x: f32) -> f32 {
                match self {$(
                    Self::$name => $func,
                )*}
            }

            pub fn iter_all() -> impl Iterator<Item = (&'static str, Self)> {
                [
                    $( ($display, Self::$name) ),*
                ].into_iter()
            }

            pub fn display_name(&self) -> &'static str {
                match self {$(
                    Self::$name => $display,
                )*}
            }

        }
    };
}

static C1: f32 = 1.70158;
static C2: f32 = C1 * 1.525;
static C3: f32 = C1 + 1.;
static C4: f32 = (2. * PI) / 3.;
static C5: f32 = (2. * PI) / 4.5;
static N1: f32 = 7.5625;
static D1: f32 = 2.75;

easing! {x
    "Step           ":  -1 : easeStep => if x >= 1. { 1. } else { 0. }
    "Linear         ":   0 : easeLinear => x
    "Quad       (I) ":   1 : easeInQuad => x * x
    "Quad        (O)":   2 : easeOutQuad => 1. - (1. - x) * (1. - x)
    "Quad       (IO)":   3 : easeInOutQuad => if x < 0.5 { 2. * x*x } else { 1. - (-2. * x + 2.).powi(2) / 2. }
    "Sine       (I) ":   4 : easeInSine => 1. - ((x * PI) / 2.).cos()
    "Sine        (O)":   5 : easeOutSine => ((x * PI) / 2.).sin()
    "Sine       (IO)":   6 : easeInOutSine => -((x * PI).cos() - 1.) / 2.
    "Cubic      (I) ":   7 : easeInCubic => x*x*x
    "Cubic       (O)":   8 : easeOutCubic => 1. - (1. - x).powi(3)
    "Cubid      (IO)":   9 : easeInOutCubic => if x < 0.5 { 4. * x*x*x } else { 1. - (-2. * x + 2.).powi(3) / 2. }
    "Quart      (I) ":  10 : easeInQuart => x*x*x*x
    "Quart       (O)":  11 : easeOutQuart => 1. - (1. - x).powi(4)
    "Quart      (IO)":  12 : easeinOutQuart => if x < 0.5 { 8. * x*x*x*x } else { 1. - (-2. * x + 2.).powi(4) / 2. }
    "Quint      (I) ":  13 : easeInQuint => x.powi(5)
    "Quint       (O)":  14 : easeOutQuint => 1. - (1. - x).powi(5)
    "Quint      (IO)":  15 : easeInOutQuint => if x < 0.5 { 16. * x.powi(5) } else { 1. - (-2. * x + 2.).powi(5) / 2. }
    "Expo       (I) ":  16 : easeInExpo => if x == 0. { x } else { 2f32.powf(10. * x - 10.) }
    "Expo        (O)":  17 : easeOutExpo => if x == 1. { x } else { 1. - 2f32.powf(-10. * x) }
    "Expo       (IO)":  18 : easeInOutExpo => if x == 0. || x == 1. { x } else if x < 0.5 { 2f32.powf(20. * x - 10.) / 2. } else { (2. - 2f32.powf(-20. * x + 10.)) / 2. }
    "Circ       (I) ":  19 : easeInCirc => 1. - (1. - x.powi(2)).sqrt()
    "Circ        (O)":  20 : easeOutCirc => (1. - (x - 1.).powi(2)).sqrt()
    "Circ       (IO)":  21 : easeinOutCirc => if x < 0.5 { (1. - (1. - (2. * x).powi(2)).sqrt()) / 2. } else { ((1. - (-2. * x + 2.).powi(2)).sqrt() + 1.) / 2. }
    "Back       (I) ":  22 : easeInBack => C3 * x*x*x - C1 * x*x
    "Back        (O)":  23 : easeOutBack => 1. + C3 * (x - 1.).powi(3) + C1 * (x - 1.).powi(2)
    "Back       (IO)":  24 : easeInOutBack => if x < 0.5 { (2. * x).powi(2) * ((C2 + 1.) * 2. * x - C2) / 2. } else { ((2. * x - 2.).powi(2) * ((C2 + 1.) * (x * 2. - 2.) + C2) + 2.) / 2. }
    "Elastic    (I) ":  25 : easeInElastic => if x == 0. || x == 1. { x } else { 2f32.powf(-10. * x) * ((x * 10. - 0.75) * C4).sin() + 1. }
    "Elastic     (O)":  26 : easeOutElastic => if x == 0. || x == 1. { x } else { 2f32.powf(-10. * x) * ((x * 10. - 0.75) * C4).sin() + 1. }
    "Elastic    (IO)":  27 : easeInOutElastic => { let s = ((20. * x - 11.125) * C5).sin(); if x == 0. || x == 1. { x } else if x < 0.5 { -(2f32.powf(20. * x - 10.) * s) / 2. } else { (2f32.powf(-20. * x + 10.) * s) / 2. + 1. } }
    "Bounce     (I) ":  28 : easeInBounce => 1. - Self::easeOutBounce.apply(1. - x)
    "Bounce      (O)":  29 : easeOutBounce => if x < 1. / D1 { N1 * x*x } else if x < 2. / D1 { N1 * (x - 1.5 / D1) * (x - 1.5 / D1) + 0.75 } else if x < 2.5 / D1 { N1 * (x - 2.25 / D1) * (x - 2.25 / D1) + 0.9375 } else { N1 * (x - 2.625 / D1) * (x - 2.625 / D1) + 0.984375 }
    "Bounce     (IO)":  30 : easeInOutBounce => if x < 0.5 { (1. - Self::easeOutBounce.apply(1. - 2. * x)) / 2. } else { (1. + Self::easeOutBounce.apply(2. * x - 1.)) / 2. }

    "BS Back    (IO)": 100 : easeBeatSaberInOutBack => Self::easeInOutBack.apply(x)
    "BS Elastic (IO)": 101 : easeBeatSaberInOutElastic => Self::easeInOutElastic.apply(x)
    "BS Bounce  (IO)": 102 : easeBeatSaberInOutBounce => Self::easeInOutBounce.apply(x)
}

impl Easing {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::easeLinear)
    }
}

impl TryFrom<i8> for Easing {
    type Error = beatmap::data::BeatmapDataError;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Ok(match value {
            -1..=30 | 100..=102 => unsafe { std::mem::transmute::<i8, Easing>(value) },
            _ => return Err(beatmap::data::BeatmapDataError::ToEnum {
                enum_name: "Easing",
                val: value as i32
            })
        })
    }
}

impl From<Easing> for i8 {
    fn from(value: Easing) -> Self {
        unsafe { std::mem::transmute::<Easing, i8>(value) }
    }
}
