use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

macro_rules! easing {
    (
        $x:tt
        $( $display:literal : $name:tt => $func:expr )*
    ) => {
        #[allow(non_camel_case_types)]
        #[derive(Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq, Copy, Clone)]
        pub enum Easing {
            #[default] $(
            $name,
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
    "Linear         ": easeLinear => x
    "Step           ": easeStep => if x >= 1. { 1. } else { 0. }
    "Sine       (I) ": easeInSine => 1. - ((x * PI) / 2.).cos()
    "Sine        (O)": easeOutSine => ((x * PI) / 2.).sin()
    "Sine       (IO)": easeInOutSine => -((x * PI).cos() - 1.) / 2.
    "Quad       (I) ": easeInQuad => x * x
    "Quad        (O)": easeOutQuad => 1. - (1. - x) * (1. - x)
    "Quad       (IO)": easeInOutQuad => if x < 0.5 { 2. * x*x } else { 1. - (-2. * x + 2.).powi(2) / 2. }
    "Cubic      (I) ": easeInCubic => x*x*x
    "Cubic       (O)": easeOutCubic => 1. - (1. - x).powi(3)
    "Cubid      (IO)": easeInOutCubic => if x < 0.5 { 4. * x*x*x } else { 1. - (-2. * x + 2.).powi(3) / 2. }
    "Quart      (I) ": easeInQuart => x*x*x*x
    "Quart       (O)": easeOutQuart => 1. - (1. - x).powi(4)
    "Quart      (IO)": easeinOutQuart => if x < 0.5 { 8. * x*x*x*x } else { 1. - (-2. * x + 2.).powi(4) / 2. }
    "Quint      (I) ": easeInQuint => x.powi(5)
    "Quint       (O)": easeOutQuint => 1. - (1. - x).powi(5)
    "Quint      (IO)": easeInOutQuint => if x < 0.5 { 16. * x.powi(5) } else { 1. - (-2. * x + 2.).powi(5) / 2. }
    "Expo       (I) ": easeInExpo => if x == 0. { x } else { 2f32.powf(10. * x - 10.) }
    "Expo        (O)": easeOutExpo => if x == 1. { x } else { 1. - 2f32.powf(-10. * x) }
    "Expo       (IO)": easeInOutExpo => if x == 0. || x == 1. { x } else if x < 0.5 { 2f32.powf(20. * x - 10.) / 2. } else { (2. - 2f32.powf(-20. * x + 10.)) / 2. }
    "Circ       (I) ": easeInCirc => 1. - (1. - x.powi(2)).sqrt()
    "Circ        (O)": easeOutCirc => (1. - (x - 1.).powi(2)).sqrt()
    "Circ       (IO)": easeinOutCirc => if x < 0.5 { (1. - (1. - (2. * x).powi(2)).sqrt()) / 2. } else { ((1. - (-2. * x + 2.).powi(2)).sqrt() + 1.) / 2. }
    "Back       (I) ": easeInBack => C3 * x*x*x - C1 * x*x
    "Back        (O)": easeOutBack => 1. + C3 * (x - 1.).powi(3) + C1 * (x - 1.).powi(2)
    "Back       (IO)": easeInOutBack => if x < 0.5 { (2. * x).powi(2) * ((C2 + 1.) * 2. * x - C2) / 2. } else { ((2. * x - 2.).powi(2) * ((C2 + 1.) * (x * 2. - 2.) + C2) + 2.) / 2. }
    "Elastic    (I) ": easeInElastic => if x == 0. || x == 1. { x } else { 2f32.powf(-10. * x) * ((x * 10. - 0.75) * C4).sin() + 1. }
    "Elastic     (O)": easeOutElastic => if x == 0. || x == 1. { x } else { 2f32.powf(-10. * x) * ((x * 10. - 0.75) * C4).sin() + 1. }
    "Elastic    (IO)": easeInOutElastic => { let s = ((20. * x - 11.125) * C5).sin(); if x == 0. || x == 1. { x } else if x < 0.5 { -(2f32.powf(20. * x - 10.) * s) / 2. } else { (2f32.powf(-20. * x + 10.) * s) / 2. + 1. } }
    "Bounce     (I) ": easeInBounce => 1. - Self::easeOutBounce.apply(1. - x)
    "Bounce      (O)": easeOutBounce => if x < 1. / D1 { N1 * x*x } else if x < 2. / D1 { N1 * (x - 1.5 / D1) * (x - 1.5 / D1) + 0.75 } else if x < 2.5 / D1 { N1 * (x - 2.25 / D1) * (x - 2.25 / D1) + 0.9375 } else { N1 * (x - 2.625 / D1) * (x - 2.625 / D1) + 0.984375 }
    "Bounce     (IO)": easeInOutBounce => if x < 0.5 { (1. - Self::easeOutBounce.apply(1. - 2. * x)) / 2. } else { (1. + Self::easeOutBounce.apply(2. * x - 1.)) / 2. }

    "BS Back    (IO)": easeBeatSaberInOutBack => Self::easeInOutBack.apply(x)
    "BS Elastic (IO)": easeBeatSaberInOutElastic => Self::easeInOutElastic.apply(x)
    "BS Bounce  (IO)": easeBeatSaberInOutBounce => Self::easeInOutBounce.apply(x)
}

impl Easing {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::easeLinear)
    }
}
