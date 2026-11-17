use glam::Vec4;
use crate::tracks::points::F32Point;
use crate::tracks::{points::Angle, CacheableTimed, Now, Timed, Track, TrackSet};
use points::{ColorBoostPoint, LightPoint, RotationPoint};
use crate::lightshow::v2::points::RingRotationEvent;
use crate::RuntimeData;

pub mod points;

#[derive(Clone, Debug)]
pub struct RingLightFrame {
    pub position_offset: Angle,
    pub rotation: Angle,
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
    pub left_laser_count: usize,
    pub left_spinning_count: usize,

    pub right_laser_count: usize,
    pub right_spinning_count: usize,

    pub back_laser_count: usize,
    pub center_laser_count: usize,

    pub outer_ring_count: usize,
    pub inner_ring_count: usize,

    pub ring_light_count: usize,

}

impl V2StateFrame {
    fn new(info: &V2EnvironmentInfo) -> Self {
        Self {
            left_laser_colors: Vec::with_capacity(info.left_laser_count),
            left_laser_rotations: Vec::with_capacity(info.left_spinning_count),

            right_laser_colors: Vec::with_capacity(info.right_laser_count),
            right_laser_rotations: Vec::with_capacity(info.right_spinning_count),

            back_lasers: Vec::with_capacity(info.back_laser_count),
            center_lasers: Vec::with_capacity(info.center_laser_count),

            outer_rings: Vec::with_capacity(info.outer_ring_count),
            inner_rings: Vec::with_capacity(info.inner_ring_count),

            ring_lights: Vec::with_capacity(info.ring_light_count),
        }
    }
}

pub struct LightshowV2Controller {
    pub info: V2EnvironmentInfo,

    pub color_boosts: Track<Timed<ColorBoostPoint>>,

    pub left_rotations: TrackSet<Timed<RotationPoint>>,
    pub right_rotations: TrackSet<Timed<RotationPoint>>,

    pub left_lasers: TrackSet<Timed<LightPoint>>,
    pub right_lasers: TrackSet<Timed<LightPoint>>,
    pub back_lasers: TrackSet<Timed<LightPoint>>,
    pub center_lasers: TrackSet<Timed<LightPoint>>,

    pub ring_zoom: TrackSet<Timed<F32Point>>,
    pub ring_rotation: TrackSet<CacheableTimed<RingRotationEvent>>,
}


impl LightshowV2Controller {

    pub fn animate(&self, beat: f32, rt: &RuntimeData) -> V2StateFrame {
        let info = &self.info;
        let second = rt.beat_to_seconds(beat);
        let now = Now { beat, second };

        let mut frame = V2StateFrame::new(info);

        let boost_state = self.color_boosts.animate(now, rt);

        self.left_lasers.iter_animate(now, rt, info.left_laser_count, &mut frame.left_laser_colors, |c| c.resolve(boost_state));
        self.left_rotations.iter_animate(now, rt, info.left_spinning_count, &mut frame.left_laser_rotations, |r| r);

        self.right_lasers.iter_animate(now, rt, info.right_laser_count, &mut frame.right_laser_colors, |c| c.resolve(boost_state));
        self.right_rotations.iter_animate(now, rt, info.right_spinning_count, &mut frame.right_laser_rotations, |r| r);

        self.back_lasers.iter_animate(now, rt, info.back_laser_count, &mut frame.back_lasers, |c| c.resolve(boost_state));
        self.center_lasers.iter_animate(now, rt, info.center_laser_count, &mut frame.center_lasers, |c| c.resolve(boost_state));

        // TODO: ring lights

        Default::default()
    }

}





