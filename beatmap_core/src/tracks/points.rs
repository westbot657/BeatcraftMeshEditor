use glam::{FloatExt, Vec2, Vec3, Vec4};
use bs_mapping_data::easing::Easing;
use crate::{timed, RuntimeData};
use crate::tracks::{BeatSpan, Point};

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
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

timed!(point Vec2Point: Vec2);
impl Point for Vec2Point {
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

timed!(point Vec3Point: Vec3);
impl Point for Vec3Point {
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

timed!(point Vec4Point: Vec4);
impl Point for Vec4Point {
    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

