use crate::RuntimeData;
use crate::lightshow::v2::{BoostableColor, ColorBoostState, LightColor, RotationDirection};
use crate::tracks::{BeatSpan, Point};
use glam::FloatExt;

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

#[derive(Copy, Clone, Debug, Default)]
pub struct RotationPoint {
    pub start: f32,
    pub speed: f32,
    pub direction: RotationDirection,
}

impl Point for RotationPoint {
    type Output = f32;

    fn interpolate(&self, _other: &Self, span: BeatSpan, rt: &RuntimeData) -> Self::Output {
        let start = rt.beat_to_seconds(span.start);
        let now = rt.beat_to_seconds(span.now);
        let delta = now - start;
        let mut delta_rotation = delta * self.speed;
        if self.direction == RotationDirection::CounterClockwise {
            delta_rotation = -delta_rotation;
        }
        self.start + delta_rotation
    }
}

/// LightPoints are significantly different from V2 Light Events since LightEvent's Transition event
/// type is impossible in this stateless system, so you have to convert them into LightPoints beforehand
#[derive(Default, Clone, Debug)]
pub struct LightPoint {
    pub color: LightColor,
    pub brightness: f32,
}

impl Point for LightPoint {
    type Output = BoostableColor;

    fn interpolate(&self, other: &Self, span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output {
        let dt = span.inverse_lerp_seconds(runtime_data);
        let color = self.color.resolve(&runtime_data.color_scheme).lerp(other.color.resolve(&runtime_data.color_scheme), dt);
        let brightness = self.brightness.lerp(other.brightness, dt);

        color.lerp_brightness(brightness * 1.2)
    }
}