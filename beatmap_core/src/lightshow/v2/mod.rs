use crate::tracks::points::F32Point;
use crate::tracks::{Track, TrackSet};
use points::{ColorBoostPoint, LightPoint, RotationPoint};

pub mod points;



pub struct LightshowV2Controller {
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






