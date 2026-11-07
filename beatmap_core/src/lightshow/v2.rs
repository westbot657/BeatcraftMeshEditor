use crate::lightshow::{Point, Track};
use crate::RuntimeData;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ColorBoostState {
    #[default]
    Normal,
    Boosted,
}

#[derive(Copy, Clone, Debug)]
pub struct ColorBoostPoint {
    pub beat: f32,
    pub state: ColorBoostState,
}

impl Point for ColorBoostPoint {
    type Output = ColorBoostState;
    fn startpoint(beat: f32) -> Self {
        Self {
            beat,
            state: Default::default(),
        }
    }

    fn extend(&self, beat: f32) -> Self {
        Self {
            beat,
            state: self.state,
        }
    }

    fn beat(&self) -> f32 {
        self.beat
    }

    /// Color boosts is an on/off state, no interpolation exists
    fn interpolate(&self, _other: &Self, _beat: f32, rt: &RuntimeData) -> Self::Output {
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

#[derive(Copy, Clone, Debug)]
pub struct RotationPoint {
    pub beat: f32,
    pub start: f32,
    pub speed: f32,
    pub direction: RotationDirection,
}

impl Point for RotationPoint {
    type Output = f32;

    fn startpoint(beat: f32) -> Self {
        Self {
            beat,
            start: 0.,
            speed: 0.,
            direction: Default::default(),
        }
    }

    fn extend(&self, beat: f32) -> Self {
        Self {
            beat,
            start: self.start,
            speed: self.speed,
            direction: self.direction,
        }
    }

    fn beat(&self) -> f32 {
        self.beat
    }

    fn interpolate(&self, _other: &Self, beat: f32, rt: &RuntimeData) -> Self::Output {
        let start = rt.beat_to_seconds(self.beat);
        let now = rt.beat_to_seconds(beat);
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
    pub left_rotating_lights: Track<RotationPoint>,
    pub right_rotating_lights: Track<RotationPoint>,
}






