use glam::{FloatExt, Vec2, Vec3, Vec4};
use bs_mapping_data::easing::Easing;
use crate::{timed, RuntimeData};
use crate::tracks::{TimeSpan, Point};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Angle {
    Radians(f32),
    Degrees(f32),
}

impl Default for Angle {
    fn default() -> Self {
        Self::Radians(0.)
    }
}

impl Angle {
    pub fn to_radians(self) -> f32 {
        match self {
            Self::Radians(r) => r,
            Self::Degrees(r) => r.to_radians(),
        }
    }
    pub fn to_degrees(self) -> f32 {
        match self {
            Self::Radians(r) => r.to_degrees(),
            Self::Degrees(d) => d,
        }
    }
}

impl std::ops::Add for Angle {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(r + rhs.to_radians()),
            Self::Degrees(r) => Self::Degrees(r + rhs.to_degrees()),
        }
    }
}

impl std::ops::Mul<f32> for Angle {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(r * rhs),
            Self::Degrees(d) => Self::Degrees(d * rhs),
        }
    }
}

impl std::ops::Mul<Angle> for f32 {
    type Output = Angle;
    fn mul(self, rhs: Angle) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Sub for Angle {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(r - rhs.to_radians()),
            Self::Degrees(r) => Self::Degrees(r - rhs.to_degrees()),
        }
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(-r),
            Self::Degrees(d) => Self::Degrees(-d),
        }
    }
}


#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct F32Point {
    pub v: f32,
    pub easing_from_last: Easing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vec2Point {
    pub v: Vec2,
    pub easing_from_last: Easing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vec3Point {
    pub v: Vec3,
    pub easing_from_last: Easing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vec4Point {
    pub v: Vec4,
    pub easing_from_last: Easing,
}

timed!(point F32Point: f32);
impl Point for F32Point {
    fn interpolate(&self, other: &Self, mut span: TimeSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

timed!(point Vec2Point: Vec2);
impl Point for Vec2Point {
    fn interpolate(&self, other: &Self, mut span: TimeSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

timed!(point Vec3Point: Vec3);
impl Point for Vec3Point {
    fn interpolate(&self, other: &Self, mut span: TimeSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

timed!(point Vec4Point: Vec4);
impl Point for Vec4Point {
    fn interpolate(&self, other: &Self, mut span: TimeSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

