use crate::tracks::{Angle, BeatSpan, Point};
use crate::{ColorScheme, RuntimeData};
use glam::{FloatExt, Vec4};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ColorBoostState {
    #[default]
    Normal,
    Boosted,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ColorBoostPoint {
    pub state: ColorBoostState,
}

impl Point for ColorBoostPoint {
    type Output = ColorBoostState;

    /// Color boosts is an on/off state, no interpolation exists
    fn interpolate(&self, _other: &Self, _span: BeatSpan, _rt: &RuntimeData) -> Self::Output {
        self.state
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum RotationDirection {
    #[default]
    /// angle will increase towards +INFINITY
    Clockwise,
    /// angle will decrease towards -INFINITY
    CounterClockwise,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RotationPoint {
    pub start: Angle,
    pub speed: Angle,
    pub direction: RotationDirection,
}

impl Point for RotationPoint {
    type Output = Angle;

    fn interpolate(&self, _other: &Self, span: BeatSpan, rt: &RuntimeData) -> Self::Output {
        let start = rt.beat_to_seconds(span.start);
        let now = span.now_seconds;
        let delta = now - start;
        let mut delta_rotation = delta * self.speed;
        if self.direction == RotationDirection::CounterClockwise {
            delta_rotation = -delta_rotation;
        }
        self.start + delta_rotation
    }
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

impl Default for BoostableColor {
    fn default() -> Self {
        Self::Unboostable(Vec4::ZERO)
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

    pub fn resolve(self, boost_state: ColorBoostState) -> Vec4 {
        match self {
            Self::Unboostable(v4) => v4,
            Self::Boostable { normal, boosted } => match boost_state {
                ColorBoostState::Normal => normal,
                ColorBoostState::Boosted => boosted,
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum LightAction {
    #[default]
    /// Instantly turns off the light
    Off,
    /// Instantly turns on the light
    On(LightColor),
    /// Flashes the given color and then fades to off over the next second
    Fade(LightColor),
    /// Flashes the given color and then fades to the set brightness over the next second
    Flash(LightColor),
    /// Interpolates to this color during the span between the last point and this one
    Transition(LightColor),
}

impl LightAction {
    pub fn get_color(&self) -> Option<LightColor> {
        match self {
            Self::Off => None,
            Self::On(c)
            | Self::Fade(c)
            | Self::Flash(c)
            | Self::Transition(c) => Some(*c)
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct LightPoint {
    pub action: LightAction,
    pub brightness: f32,
}

impl Point for LightPoint {
    type Output = BoostableColor;

    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output {
        let color = self.action.get_color()
            .map(|c| c.resolve(&runtime_data.color_scheme));
        if let Some(mut color) = color {
            let brightness = if let LightAction::Transition(c2) = other.action {
                let dt = span.inverse_lerp_seconds(runtime_data);
                color = color.lerp(c2.resolve(&runtime_data.color_scheme), dt);
                self.brightness.lerp(other.brightness, dt)
            } else {
                self.brightness
            };
            match self.action {
                LightAction::On(_) | LightAction::Transition(_) => color.lerp_brightness(brightness),
                LightAction::Fade(_) => {
                    let elapsed = span.elapsed_seconds(runtime_data);
                    let b = f32::lerp(1.2, 0., elapsed).clamp(0., 1.);
                    color.lerp_brightness(b)
                }
                LightAction::Flash(_) => {
                    let elapsed = span.elapsed_seconds(runtime_data);
                    let b = f32::lerp(1.2, brightness, elapsed).clamp(0., 1.);
                    color.lerp_brightness(b)
                }
                LightAction::Off => unreachable!("LightAction::Off is inaccessible if a color is attached")
            }
        } else {
            BoostableColor::Unboostable(Vec4::ZERO)
        }
    }
}
