use crate::tracks::{BeatSpan, Point, Track, TrackSet};
use crate::RuntimeData;

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
    fn interpolate(&self, _other: &Self, _span: BeatSpan, rt: &RuntimeData) -> Self::Output {
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

pub struct LightshowV2Controller {
    pub color_boosts: Track<ColorBoostPoint>,
    pub left_rotating_lights: TrackSet<RotationPoint>,
    pub right_rotating_lights: TrackSet<RotationPoint>,
}






