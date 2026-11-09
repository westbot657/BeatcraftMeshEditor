use crate::tracks::points::F32Point;
use crate::tracks::{Track, TrackSet};
use crate::ColorScheme;
use glam::Vec4;
use points::{ColorBoostPoint, LightPoint, RotationPoint};

pub mod points;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ColorBoostState {
    #[default]
    Normal,
    Boosted,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum RotationDirection {
    #[default]
    /// angle will increase towards +INFINITY
    Clockwise,
    /// angle will decrease towards -INFINITY
    CounterClockwise,
}


#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum LightColor {
    #[default]
    Left,
    Right,
    White,
    Chroma(Vec4),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoostableColor {
    Unboostable(Vec4),
    Boostable {
        normal: Vec4,
        boosted: Vec4,
    }
}

impl From<Vec4> for BoostableColor {
    fn from(value: Vec4) -> Self {
        Self::Unboostable(value)
    }
}

impl From<(Vec4, Vec4)> for BoostableColor {
    fn from(value: (Vec4, Vec4)) -> Self {
        Self::Boostable {
            normal: value.0,
            boosted: value.1,
        }
    }
}

impl LightColor {
    fn resolve(&self, cs: &ColorScheme) -> BoostableColor {
        match self {
            Self::Left => (cs.lights.primary, cs.boost.primary).into(),
            Self::Right => (cs.lights.secondary, cs.boost.secondary).into(),
            Self::White => (cs.lights.white, cs.boost.white).into(),
            Self::Chroma(c) => (*c).into(),
        }
    }
}

fn lerp_brightness(color: Vec4, brightness: f32) -> Vec4 {
    if brightness <= 1. {
        color * brightness
    } else {
        let over_bright = brightness - 1.0;
        color + (1. - color) * over_bright
    }//.clamp(Vec4::ZERO, Vec4::ONE)
}

impl BoostableColor {
    fn lerp(self, other: Self, dt: f32) -> Self {
        match (self, other) {
            (
                Self::Unboostable(v4),
                Self::Unboostable(u4)
            ) => Self::Unboostable(v4.lerp(u4, dt)),
            (
                Self::Unboostable(v4),
                Self::Boostable { normal: un, boosted: ub }
            ) => Self::Boostable { normal: v4.lerp(un, dt), boosted: v4.lerp(ub, dt) },
            (
                Self::Boostable { normal: vn, boosted: vb },
                Self::Unboostable(u4)
            ) => Self::Boostable { normal: vn.lerp(u4, dt), boosted: vb.lerp(u4, dt) },
            (
                Self::Boostable { normal: vn, boosted: vb },
                Self::Boostable { normal: un, boosted: ub }
            ) => Self::Boostable { normal: vn.lerp(un, dt), boosted: vb.lerp(ub, dt) },
        }
    }

    fn lerp_brightness(self, brightness: f32) -> Self {
        match self {
            Self::Unboostable(v4) => lerp_brightness(v4, brightness).into(),
            Self::Boostable { normal, boosted } => (lerp_brightness(normal, brightness), lerp_brightness(boosted, brightness)).into(),
        }
    }
}


pub struct LightshowV2Controller {
    pub color_boosts: Track<ColorBoostPoint>,

    pub left_rotations: TrackSet<RotationPoint>,
    pub right_rotations: TrackSet<RotationPoint>,

    pub left_lasers: TrackSet<LightPoint>,
    pub right_lasers: TrackSet<LightPoint>,
    pub back_lasers: TrackSet<LightPoint>,
    pub center_lasers: TrackSet<LightPoint>,

    pub ring_zoom: TrackSet<F32Point>,
    pub ring_rotation: TrackSet<F32Point>,
}






