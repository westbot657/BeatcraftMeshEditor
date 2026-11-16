use glam::{FloatExt, Vec2, Vec3, Vec4};
use bs_mapping_data::easing::Easing;
use crate::RuntimeData;
use crate::tracks::{BeatSpan, Point, PointAnimator, Timeable, TimedData};

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

impl TimedData for F32Point {
    type Animator<T: Timeable<Data = Self>> = PointAnimator<Self>;
    type State = f32;
}
impl Point for F32Point {
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}
impl TimedData for Vec2Point {
    type Animator<T: Timeable<Data = Self>> = PointAnimator<Self>;
    type State = Vec2;
}
impl Point for Vec2Point {
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

impl TimedData for Vec3Point {
    type Animator<T: Timeable<Data = Self>> = PointAnimator<Self>;
    type State = Vec3;
}
impl Point for Vec3Point {
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

impl TimedData for Vec4Point {
    type Animator<T: Timeable<Data = Self>> = PointAnimator<Self>;
    type State = Vec4;
}
impl Point for Vec4Point {
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

