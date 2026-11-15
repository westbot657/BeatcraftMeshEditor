use glam::{FloatExt, Vec2, Vec3, Vec4};
use beatcraft_editor_proc::point;
use bs_mapping_data::easing::Easing;
use crate::RuntimeData;
use crate::tracks::BeatSpan;

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

point! {
    F32Point: f32 {
        fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
            let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
            let dt = other.easing_from_last.apply(dt);
            self.v.lerp(other.v, dt)
        }
    }
}
point! {
    Vec2Point: Vec2 {
        fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
            let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
            let dt = other.easing_from_last.apply(dt);
            self.v.lerp(other.v, dt)
        }
    }
}

point! {
    Vec3Point: Vec3 {
        fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
            let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
            let dt = other.easing_from_last.apply(dt);
            self.v.lerp(other.v, dt)
        }
    }
}

point! {
    Vec4Point: Vec4 {
        fn interpolate(&self, other: &Self, mut span: BeatSpan, runtime_data: &RuntimeData) -> Self::State {
            let dt = span.inverse_lerp_seconds(runtime_data); // TODO: check if this should lerp in seconds or beats
            let dt = other.easing_from_last.apply(dt);
            self.v.lerp(other.v, dt)
        }
    }
}

