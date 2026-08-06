use std::ops::{Add, Div, Mul, Sub};

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use rand::{rngs::ThreadRng, RngExt};

use crate::easing::Easing;
use crate::render::GameObjectInstanceData;

use super::BeatmapProjectDiff;
use super::data::{BeatmapDataError, BeatmapFile, CutDirection, InfoFile, v2};
use super::render::BeatmapRenderer;

pub struct RuntimeData {
    njs: f32,
    bpm: f32,
    hjd: f32,
    jd: f32,
}

pub struct BeatmapController {
    pub runtime_data: RuntimeData,
    pub color_notes: Vec<ColorNote>,
    pub bomb_notes: Vec<BombNote>,
    pub obstacles: Vec<Obstacle>,
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
            hjd,
            jd
        }
    }

    fn calc_hjd(njs: f32, bpm: f32, spawn_offset: f32) -> (f32, f32) {
        let mut hjd = 4.;
        let spb = 60. / bpm;

        let n2 = njs * spb;
        let mut n3 = n2 * hjd;
        while n3 >= 18. {
            hjd /= 2.;
            n3 = n2 * hjd;
        }

        hjd += spawn_offset;
        if hjd < 0.25 {
            hjd = 0.25;
        }

        let jd = hjd * 2. * spb * njs;

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

pub const HEAD_POS: Vec3 = Vec3::new(0., 1.62, 0.);

pub fn get_random_spawn_quat(rng: &mut ThreadRng) -> Quat {
    let get_c = |rng: &mut ThreadRng| -> f32 {
        rng.random::<f32>() * 0.6f32 * 2. - 0.6
    };
    Quat::from_euler(glam::EulerRot::ZYX, get_c(rng), get_c(rng), get_c(rng))
}

pub trait GameObject {

    fn beat(&self) -> f32;
    fn grid_pos(&self) -> Vec2;
    fn get_orientation(&self) -> Quat;
    fn spawn_orientation(&self) -> Quat { Quat::IDENTITY }
    fn do_gravity(&self) -> bool { false }
    fn do_look(&self) -> bool { false }
    fn do_spawn_rotation(&self) -> bool { false }
    fn duration(&self) -> f32 { 0. }
    fn arrow_type(&self) -> ArrowType { ArrowType::None }

    fn get_instance(&self, clipping_plane: Vec4, model: Mat4) -> GameObjectInstanceData;

    fn animate_simple(&self, mut m: Mat4, beat: f32, _data: &RuntimeData, renderer: &BeatmapRenderer) -> Option<Mat4> {
        let b = self.beat();
        let pre = renderer.beats_before as f32;
        let post = renderer.visible_beat_count as f32 - pre;
        let s = b - post;
        let d = b + self.duration() + pre;

        if (s..d).contains(&beat) {
            let gp = self.grid_pos();

            let gp = Vec2::new(1.5 - gp.x, gp.y + 0.5) * 0.6;

            m *= Mat4::from_translation(gp.extend((b - beat) * renderer.beat_spacing));
            m *= Mat4::from_quat(self.get_orientation());
            Some(m)
        } else {
            None
        }

    }

    fn animate_complex(&self, mut m: Mat4, beat: f32, data: &RuntimeData) -> Option<Mat4> {
        let b = self.beat();

        let dur = self.duration();
        let s = b - data.hjd;
        let d = b + dur + data.hjd;

        if (s..d).contains(&beat) {
            let ji = b - data.hjd / 2.;
            let jo = (b + dur) + data.hjd / 2.;
            let jip = data.jd / 2.;

            let njs_length = data.njs * (60. / data.bpm);

            let jop = if dur > 0. {
                -(njs_length * dur)
            } else {
                data.jd * -0.25
            };

            let mut gp = self.grid_pos();

            gp = Vec2::new(1.5 - gp.x, gp.y + 0.5) * 0.6;

            let lifetime = f32::inv_lerp(s, d, beat).clamp(0., 1.);
            let spawn_lifetime = (lifetime * 2.).clamp(0., 1.);

            let rst = 1. - (lifetime - 0.5).abs() * 2.;
            let jump_time = Easing::easeOutQuad.apply(rst);
            gp.y = f32::lerp(if self.do_gravity() { -0.3 } else { gp.y - 0.3 }, gp.y, jump_time);

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

            let jump_mat = Mat4::from_translation(gp.extend(z));
            m *= Mat4::from_translation(Vec3::new(0., 0.8, 0.));

            let ori = if self.do_look()
            && lifetime < 0.5 {
                let mi = m.inverse();
                let mut hp = mi.transform_point3(HEAD_POS);
                hp = jump_mat.transform_point3(hp * -1.).normalize();
                let target = Quat::from_rotation_arc(Vec3::Z, hp);
                Quat::IDENTITY.slerp(target, spawn_lifetime)
            } else { Quat::IDENTITY };

            m *= jump_mat;

            m *= Mat4::from_quat(ori);

            let local_rot = if lifetime < 0.5
            && spawn_lifetime != 0. {
                let rot_lifetime = (spawn_lifetime / 0.3).clamp(0., 1.);
                let rt = Easing::easeOutQuad.apply(rot_lifetime);
                let so = self.spawn_orientation();
                let or = self.get_orientation();
                so.slerp(or, rt)
            } else {
                self.get_orientation()
            };

            m *= Mat4::from_quat(local_rot);
            Some(m)
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
    pub dissolve: f32,
    pub index: u32,
}

pub struct BombNote {
    pub beat: f32,
    pub color: Vec4,
    pub grid_pos: Vec2,
    pub dissolve: f32,
    pub index: u32,
}

pub struct Obstacle {
    pub beat: f32,
    pub color: Vec4,
    pub grid_pos: Vec2,
    pub dissolve: f32,
    pub index: u32,
    pub size: Vec3,
}

impl GameObject for ColorNote {
    fn beat(&self) -> f32 { self.beat }
    fn grid_pos(&self) -> Vec2 { self.grid_pos }
    fn get_orientation(&self) -> Quat {
        self.cut_direction.to_quat() * Quat::from_rotation_z(self.angle_offset)
    }
    fn do_gravity(&self) -> bool { true }
    fn do_look(&self) -> bool { true }
    fn do_spawn_rotation(&self) -> bool { true }
    fn spawn_orientation(&self) -> Quat { self.spawn_orientation }
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

impl GameObject for BombNote {
    fn beat(&self) -> f32 { self.beat }
    fn grid_pos(&self) -> Vec2 { self.grid_pos }
    fn get_orientation(&self) -> Quat { Quat::IDENTITY }
    fn do_gravity(&self) -> bool { true }
    fn do_spawn_rotation(&self) -> bool { true }
    fn get_instance(&self, clipping_plane: Vec4, model: Mat4) -> GameObjectInstanceData {
        GameObjectInstanceData::bomb_note(
            clipping_plane,
            model,
            self.color,
            self.dissolve,
            self.index,
            Vec4::ZERO
        )
    }
}

impl GameObject for Obstacle {
    fn beat(&self) -> f32 { self.beat }
    fn grid_pos(&self) -> Vec2 { self.grid_pos }
    fn get_orientation(&self) -> Quat { Quat::IDENTITY }
    fn duration(&self) -> f32 { self.size.z }
    fn get_instance(&self, clipping_plane: Vec4, model: Mat4) -> GameObjectInstanceData {
        GameObjectInstanceData::obstacle(
            clipping_plane,
            model,
            self.color,
            self.dissolve,
            self.index,
            self.size
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

        let mut rng = rand::rng();
        let mut color_notes = Vec::new();
        let mut bomb_notes = Vec::new();
        let mut obstacles = Vec::new();

        #[allow(clippy::single_match)]
        match self {
            Self::V2(v2) => {
                for note in v2.notes.iter() {
                    match note {
                        v2::V2Note::Note(color_note) => {
                            let index = color_notes.len() as u32;
                            color_notes.push(ColorNote {
                                spawn_orientation: get_random_spawn_quat(&mut rng),
                                beat: color_note.time,
                                color: color_note.typ.to_default_color(),
                                cut_direction: color_note.cut_direction,
                                angle_offset: 0.,
                                grid_pos: Vec2::new(color_note.line_index, color_note.line_layer),
                                dissolve: 0.,
                                index
                            })
                        },
                        v2::V2Note::Bomb(bomb_note) => {
                            let index = bomb_notes.len() as u32;
                            bomb_notes.push(BombNote {
                                beat: bomb_note.beat,
                                color: Vec4::new(0.2, 0.2, 0.2, 1.),
                                grid_pos: Vec2::new(bomb_note.line_index, bomb_note.line_layer),
                                dissolve: 0.,
                                index,
                            })
                        },
                    }
                }
                for obst in v2.obstacles.iter() {
                    let index = obstacles.len() as u32;
                    let (grid_pos, size) = match obst.typ {
                        super::data::ObstacleV2Type::FullHeight => (
                            Vec2::new(obst.line_index, obst.line_layer - 0.8),
                            Vec3::new(obst.width, 5., obst.duration),
                        ),
                        super::data::ObstacleV2Type::Crouch => (
                            Vec2::new(obst.line_index, obst.line_layer + 1.2),
                            Vec3::new(obst.width, 3., obst.duration)
                        ),
                        super::data::ObstacleV2Type::Free => (
                            Vec2::new(obst.line_index, obst.line_layer - 0.8),
                            Vec3::new(obst.width, obst.height, obst.duration)
                        ),
                    };
                    obstacles.push(Obstacle {
                        beat: obst.beat,
                        color: Vec4::new(1., 0.184, 0.184, 1.),
                        grid_pos,
                        dissolve: 0.,
                        index,
                        size,
                    })
                }
            }
        }

        Ok(BeatmapController {
            runtime_data: RuntimeData::new(diff_data.njs, info.bpm(), diff_data.njs_offset),
            color_notes,
            bomb_notes,
            obstacles,
        })
    }
}

