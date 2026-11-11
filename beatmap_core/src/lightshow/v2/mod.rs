use glam::Vec4;
use crate::tracks::points::F32Point;
use crate::tracks::{Angle, Track, TrackSet};
use points::{ColorBoostPoint, LightPoint, RotationPoint};
use crate::RuntimeData;

pub mod points;

#[derive(Clone, Debug)]
pub struct RingLightFrame {
    pub position_offset: f32,
    pub rotation: f32,
}

#[derive(Clone, Debug, Default)]
pub struct V2StateFrame {
    pub left_laser_colors: Vec<Vec4>,
    pub left_laser_rotations: Vec<Angle>,

    pub right_laser_colors: Vec<Vec4>,
    pub right_laser_rotations: Vec<Angle>,

    pub back_lasers: Vec<Vec4>,
    pub center_lasers: Vec<Vec4>,

    pub outer_rings: Vec<RingLightFrame>,
    pub inner_rings: Vec<RingLightFrame>,

    pub ring_lights: Vec<Vec4>,
}

pub struct V2RingInfo {
    pub start_offsets: Vec<Angle>,
    pub offsets: Vec<Angle>,
}

pub struct V2EnvironmentInfo {
    pub left_laser_count: u32,
    pub left_spinning_count: u32,

    pub right_laser_count: u32,
    pub right_spinning_count: u32,

    pub back_laser_count: u32,
    pub center_laser_count: u32,

    pub outer_ring_count: u32,
    pub inner_ring_count: u32,

    pub ring_light_count: u32,

}

impl V2StateFrame {
    fn new(info: &V2EnvironmentInfo) -> Self {
        Self {
            left_laser_colors: Vec::with_capacity(info.left_laser_count as usize),
            left_laser_rotations: Vec::with_capacity(info.left_spinning_count as usize),

            right_laser_colors: Vec::with_capacity(info.right_laser_count as usize),
            right_laser_rotations: Vec::with_capacity(info.right_spinning_count as usize),

            back_lasers: Vec::with_capacity(info.back_laser_count as usize),
            center_lasers: Vec::with_capacity(info.center_laser_count as usize),

            outer_rings: Vec::with_capacity(info.outer_ring_count as usize),
            inner_rings: Vec::with_capacity(info.inner_ring_count as usize),

            ring_lights: Vec::with_capacity(info.ring_light_count as usize),
        }
    }
}

pub struct LightshowV2Controller {
    pub info: V2EnvironmentInfo,

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


impl LightshowV2Controller {

    pub fn animate(&self, beat: f32, rt: &RuntimeData) -> V2StateFrame {
        let info = &self.info;
        let second = rt.beat_to_seconds(beat);

        let mut frame = V2StateFrame::new(info);

        let boost_state = self.color_boosts.animate(beat, second, rt);

        self.left_lasers.iter_animate(beat, second, rt, info.left_laser_count as usize, &mut frame.left_laser_colors, |c| c.resolve(boost_state));
        self.left_rotations.iter_animate(beat, second, rt, info.left_spinning_count as usize, &mut frame.left_laser_rotations, |r| r);

        self.right_lasers.iter_animate(beat, second, rt, info.right_laser_count as usize, &mut frame.right_laser_colors, |c| c.resolve(boost_state));
        self.right_rotations.iter_animate(beat, second, rt, info.right_spinning_count as usize, &mut frame.right_laser_rotations, |r| r);

        self.back_lasers.iter_animate(beat, second, rt, info.back_laser_count as usize, &mut frame.back_lasers, |c| c.resolve(boost_state));
        self.center_lasers.iter_animate(beat, second, rt, info.center_laser_count as usize, &mut frame.center_lasers, |c| c.resolve(boost_state));

        // TODO: ring lights

        Default::default()
    }

}





