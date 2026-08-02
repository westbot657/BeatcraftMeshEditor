use glam::{Mat4, Quat, Vec2, Vec4};

use super::data::CutDirection;

pub struct RuntimeData {
    pub njs: f32,
    pub bpm: f32,
    pub spawn_offset: f32,
}

pub struct BeatmapController {
    pub runtime_data: RuntimeData,
}

pub trait GameObject {

    fn beat(&self) -> f32;
    fn grid_pos(&self) -> Vec2;

    fn animate(&self, beat: f32, data: &RuntimeData) -> Option<Mat4> {
        let b = self.beat();

        let s = b - 4.;
        let d = b + 4.;

        if (s..d).contains(&beat) {
            let ji = b - 2.;
            let jo = b + 2.;

            let gp = self.grid_pos();

            Some(Mat4::from_translation((gp * Vec2::new(0.6, 0.6)).extend(beat)))
        } else {
            None
        }

    }

}

pub struct ColorNote {
    pub spawn_orientation: Quat,
    pub beat: f32,
    pub color: Vec4,
    pub cut_direction: CutDirection,
    pub angle_offset: f32,
    pub grid_pos: Vec2,
}



