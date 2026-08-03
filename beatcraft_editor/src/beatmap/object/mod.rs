use std::ops::{Add, Div, Mul, Sub};

use glam::{Mat4, Quat, Vec2, Vec4};
use num_traits::Num;

use crate::render::GameObjectInstanceData;

use super::BeatmapProjectDiff;
use super::data::{BeatmapDataError, BeatmapFile, CutDirection, InfoFile, v2};
use super::render::BeatmapRenderer;

pub struct RuntimeData {
    njs: f32,
    bpm: f32,
    spawn_offset: f32,
    hjd: f32,
    jd: f32,
}

pub struct BeatmapController {
    pub runtime_data: RuntimeData,
    pub color_notes: Vec<ColorNote>,
}

const JUMP_FAR_Z: f32 = 500.;

pub trait Lerp<T>
where
    T: Copy + Mul<Output=T> + Sub<Output=T> + Add<Output=T> + Div<Output=T>
{
    fn lerp(a: T, b: T, t: T) -> T {
        a + (b - a) * t
    }

    fn inv_lerp(a: T, b: T, x: T) -> T {
        (x - a) / (b - a)
    }
}

impl Lerp<f32> for f32 {}
impl Lerp<f64> for f64 {}

impl RuntimeData {

    pub fn new(njs: f32, bpm: f32, spawn_offset: f32) -> Self {
        let (hjd, jd) = Self::calc_hjd(njs, bpm, spawn_offset);
        Self {
            njs,
            bpm,
            spawn_offset,
            hjd,
            jd
        }
    }

    fn calc_hjd(njs: f32, bpm: f32, spawn_offset: f32) -> (f32, f32) {
        let mut hjd = 4.;
        let spb = 60. / bpm; // seconds per beat

        while njs * spb * hjd >= 18. {
            hjd /= 2.;
        }

        hjd += spawn_offset;

        if hjd < 0.25 {
            hjd = 0.25;
        }
        let jd = hjd * spb * njs * 2.;

        (hjd, jd)
    }

    pub fn beat_to_pos_simple(&self, current_beat: f32, target_beat: f32) -> f32 {
        let bps = self.bpm / 60.;
        let delta_b = target_beat - current_beat;
        let delta_s = delta_b / bps;
        delta_s * self.njs
    }

}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ArrowType {
    None,
    Arrow,
    Dot,
    ChainDot,
}

pub trait GameObject {

    fn beat(&self) -> f32;
    fn grid_pos(&self) -> Vec2;
    fn get_orientation(&self) -> Quat;

    fn arrow_type(&self) -> ArrowType { ArrowType::None }

    fn animate_simple(&self, beat: f32, data: &RuntimeData, renderer: &BeatmapRenderer) -> Option<Mat4> {
        let b = self.beat();
        let pre = renderer.beats_before as f32;
        let post = renderer.visible_beat_count as f32 - pre;
        let s = b - post;
        let d = b + pre;

        if (s..d).contains(&beat) {
            let gp = self.grid_pos();

            let gp = Vec2::new(1.5 - gp.x, gp.y + 0.5) * 0.6;

            let m = Mat4::from_translation(gp.extend((b - beat) * renderer.beat_spacing));
            let m = m * Mat4::from_quat(self.get_orientation());
            Some(m)
        } else {
            None
        }

    }

    fn animate_complex(&self, beat: f32, data: &RuntimeData) -> Option<Mat4> {
        let b = self.beat();

        let s = b - data.hjd;
        let d = b + data.hjd;

        if (s..d).contains(&beat) {
            let ji = b - data.hjd / 2.;
            let jo = b + data.hjd / 2.;
            let jip = data.jd / 2.;
            let jop = data.jd * -0.25;

            let gp = self.grid_pos();

            let gp = Vec2::new(1.5 - gp.x, gp.y + 0.5) * 0.6;

            let z = if beat <= ji {
                let p = (ji - beat) / 2.;
                f32::lerp(jip, JUMP_FAR_Z, p)
            } else if beat < jo {
                let p = f32::inv_lerp(ji, jo, beat);
                f32::lerp(jip, jop, p)
            } else {
                let mut p = f32::inv_lerp(jo, d, beat);
                p *= p;
                f32::lerp(jop, -JUMP_FAR_Z, p)
            };

            let m = Mat4::from_translation(gp.extend(z));
            let m = m * Mat4::from_quat(self.get_orientation());
            Some(m)
        } else {
            None
        }

    }

    fn get_instance(&self, clipping_plane: Vec4, model: Mat4) -> GameObjectInstanceData;

}

pub struct ColorNote {
    pub spawn_orientation: Quat,
    pub beat: f32,
    pub color: Vec4,
    pub cut_direction: CutDirection,
    pub angle_offset: f32,
    pub grid_pos: Vec2,
    pub dissolve: f32,
    pub index: u32,
}

impl GameObject for ColorNote {
    fn beat(&self) -> f32 {
        self.beat
    }
    fn grid_pos(&self) -> Vec2 {
        self.grid_pos
    }
    fn get_orientation(&self) -> Quat {
        self.cut_direction.to_quat() * Quat::from_rotation_z(self.angle_offset)
    }

    fn arrow_type(&self) -> ArrowType {
        if self.cut_direction == CutDirection::Dot {
            ArrowType::Dot
        } else {
            ArrowType::Arrow
        }
    }

    fn get_instance(&self, clipping_plane: Vec4, model: Mat4) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color,
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}


impl BeatmapController {
    pub fn new(info: &InfoFile, diff_data: &BeatmapProjectDiff, diff: &BeatmapFile) -> Result<Self, BeatmapDataError> {

        diff.to_controller(info, diff_data)
    }
}

impl BeatmapFile {
    fn to_controller(&self, info: &InfoFile, diff_data: &BeatmapProjectDiff) -> Result<BeatmapController, BeatmapDataError> {

        let mut color_notes = Vec::new();

        #[allow(clippy::single_match)]
        match self {
            Self::V2(v2) => {
                for note in v2.notes.iter() {
                    match note {
                        v2::V2Note::Note(color_note) => {
                            let index = color_notes.len() as u32;
                            color_notes.push(ColorNote {
                                spawn_orientation: Quat::IDENTITY,
                                beat: color_note.time,
                                color: color_note.typ.to_default_color(),
                                cut_direction: color_note.cut_direction,
                                angle_offset: 0.,
                                grid_pos: Vec2::new(color_note.line_index, color_note.line_layer),
                                dissolve: 0.,
                                index
                            })
                        },
                        v2::V2Note::Bomb(bomb_note) => {},
                    }
                }
            }
        }

        Ok(BeatmapController {
            runtime_data: RuntimeData::new(diff_data.njs, info.bpm(), diff_data.njs_offset),
            color_notes,
        })
    }
}

