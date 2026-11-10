use glam::{FloatExt, Vec2, Vec3, Vec4};
use bs_mapping_data::easing::Easing;
use crate::RuntimeData;
use crate::tracks::{BeatSpan, Point};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct F32Point {
    v: f32,
    easing_from_last: Easing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vec2Point {
    v: Vec2,
    easing_from_last: Easing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vec3Point {
    v: Vec3,
    easing_from_last: Easing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vec4Point {
    v: Vec4,
    easing_from_last: Easing,
}

impl Point for F32Point {
    type Output = f32;

    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

impl Point for Vec2Point {
    type Output = Vec2;

    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

impl Point for Vec3Point {
    type Output = Vec3;

    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}

impl Point for Vec4Point {
    type Output = Vec4;

    fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output {
        let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
        let dt = other.easing_from_last.apply(dt);
        self.v.lerp(other.v, dt)
    }
}
