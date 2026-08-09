use std::f32;
use std::marker::PhantomData;
use std::ops::{Add, Div, Mul, Sub};

use glam::{Mat4, Quat, Vec2, Vec3, Vec3Swizzles, Vec4};
use rand::{rngs::ThreadRng, RngExt};

use crate::easing::Easing;
use crate::render::GameObjectInstanceData;

use super::BeatmapProjectDiff;
use super::data::{BeatmapDataError, BeatmapFile, Color, CutDirection, InfoFile, v2};
use super::render::BeatmapRenderer;

pub struct RuntimeData {
    pub njs: f32,
    pub bpm: f32,
    pub hjd: f32,
    pub jd: f32,
    pub color_scheme: ColorScheme,
}

pub struct BeatmapController {
    pub runtime_data: RuntimeData,
    pub color_notes: Vec<ColorNote>,
    pub bomb_notes: Vec<BombNote>,
    pub obstacles: Vec<Obstacle>,
    pub chain_notes: Vec<ChainNote>,
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
            jd,
            color_scheme: Default::default(),
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

    fn get_instance(&self, clipping_plane: Vec4, model: Mat4, cs: &ColorScheme) -> GameObjectInstanceData;

    fn upcast_chain_head(&self) -> Option<&ChainNote> { None }

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


#[derive(Clone, Debug, PartialEq)]
pub struct LightColors {
    pub primary: Vec4,
    pub secondary: Vec4,
    pub white: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorScheme {
    pub left_note: Vec4,
    pub right_note: Vec4,
    pub obstacle: Vec4,
    pub lights: LightColors,
    pub boost: LightColors,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            left_note: Vec4::new(0.749, 0.184, 0.184, 1.),
            right_note: Vec4::new(0.122, 0.388, 0.655, 1.),
            obstacle: Vec4::new(1., 0.184, 0.184, 1.),
            lights: LightColors {
                primary: Vec4::new(0.749, 0.184, 0.184, 1.),
                secondary: Vec4::new(0.122, 0.388, 0.655, 1.),
                white: Vec4::splat(1.),
            },
            boost: LightColors {
                primary: Vec4::new(0.749, 0.184, 0.184, 1.),
                secondary: Vec4::new(0.122, 0.388, 0.655, 1.),
                white: Vec4::splat(1.),
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NoteColor {
    Red,
    Blue,
    Custom(Vec4),
}
impl NoteColor {
    pub fn color(&self, cs: &ColorScheme) -> Vec4 {
        match self {
            Self::Red => cs.left_note,
            Self::Blue => cs.right_note,
            Self::Custom(c) => *c,
        }
    }
}
impl From<Color> for NoteColor {
    fn from(value: Color) -> Self {
        match value {
            Color::Red => Self::Red,
            Color::Blue => Self::Blue,
        }
    }
}

pub trait ColorableObject where Self: Sized {
    fn color(col: &ObjectColor<Self>, cs: &ColorScheme) -> Vec4;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ObjectColor<O: ColorableObject> {
    Default(PhantomData<O>),
    Custom(Vec4),
}
impl<O: ColorableObject> ObjectColor<O> {
    pub fn color(&self, cs: &ColorScheme) -> Vec4 {
        O::color(self, cs)
    }
}
impl<O: ColorableObject> Default for ObjectColor<O> {
    fn default() -> Self {
        Self::Default(PhantomData)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ObjectType {
    ColorNote,
    BombNote,
    Obstacle,
    ChainHead,
    ChainLink,
    ArcHead,
    ArcTail,
}

pub struct ColorNote {
    pub spawn_orientation: Quat,
    pub beat: f32,
    pub color: NoteColor,
    pub cut_direction: CutDirection,
    pub angle_offset: f32,
    pub grid_pos: Vec2,

    pub dissolve: f32,
    pub index: u32,
}

pub struct BombNote {
    pub beat: f32,
    pub color: ObjectColor<Self>,
    pub grid_pos: Vec2,

    pub dissolve: f32,
    pub index: u32,
}
impl ColorableObject for BombNote {
    fn color(col: &ObjectColor<Self>, _cs: &ColorScheme) -> Vec4 {
        match col {
            ObjectColor::Default(_) => Vec4::new(0.2, 0.2, 0.2, 1.),
            ObjectColor::Custom(vec4) => *vec4,
        }
    }
}

pub struct Obstacle {
    pub beat: f32,
    pub color: ObjectColor<Self>,
    pub grid_pos: Vec2,
    pub size: Vec3,

    pub dissolve: f32,
    pub index: u32,
}
impl ColorableObject for Obstacle {
    fn color(col: &ObjectColor<Self>, cs: &ColorScheme) -> Vec4 {
        match col {
            ObjectColor::Default(_) => cs.obstacle,//Vec4::new(1., 0.184, 0.184, 1.),
            ObjectColor::Custom(vec4) => *vec4,
        }
    }
}

pub struct ChainNoteLinkData {
    pub spawn_orientation: Quat,
    pub index: u32,
}

pub struct ChainNote {
    pub spawn_orientation: Quat,
    pub head_beat: f32,
    pub tail_beat: f32,
    pub cut_direction: CutDirection,
    pub color: NoteColor,
    pub head_grid_pos: Vec2,
    pub tail_grid_pos: Vec2,
    pub squish_factor: f32,
    pub links: Vec<ChainNoteLinkData>,

    pub dissolve: f32,
    pub index: u32,
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
    fn get_instance(&self, clipping_plane: Vec4, model: Mat4, cs: &ColorScheme) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
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
    fn get_instance(&self, clipping_plane: Vec4, model: Mat4, cs: &ColorScheme) -> GameObjectInstanceData {
        GameObjectInstanceData::bomb_note(
            clipping_plane,
            model,
            self.color.color(cs),
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
    fn get_instance(&self, clipping_plane: Vec4, model: Mat4, cs: &ColorScheme) -> GameObjectInstanceData {
        GameObjectInstanceData::obstacle(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            self.size
        )
    }
}

pub struct ChainNoteLink {
    grid_pos: Vec2,
    beat: f32,
    orientation: Quat,
    spawn_orientation: Quat,
    color: NoteColor,
    index: u32,
    dissolve: f32,
}
impl GameObject for ChainNote {
    fn beat(&self) -> f32 { self.head_beat }
    fn grid_pos(&self) -> Vec2 { self.head_grid_pos }
    fn get_orientation(&self) -> Quat { self.cut_direction.to_quat() }
    fn do_gravity(&self) -> bool { true }
    fn do_look(&self) -> bool { true }
    fn do_spawn_rotation(&self) -> bool { true }
    fn spawn_orientation(&self) -> Quat { self.spawn_orientation }
    fn upcast_chain_head(&self) -> Option<&ChainNote> { Some(self) }
    fn get_instance(&self, clipping_plane: Vec4, model: Mat4, cs: &ColorScheme) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}
impl ChainNote {
    pub fn get_links(&self) -> Vec<ChainNoteLink> {
        let slice_count = self.links.len();
        if slice_count == 0 {
            return Vec::new();
        }

        let head_pos = self.head_grid_pos.extend(0.);
        let tail_offset = self.tail_grid_pos.extend(0.) - head_pos;
        let mag = tail_offset.length();
        let f = self.cut_direction.world_angle_radians() - 90f32.to_radians();
        let ctrl = Vec3::new(f.cos() * 0.5 * mag, f.sin() * 0.5 * mag, 0.);

        let spline = BezierCurve { p0: Vec3::ZERO, p1: ctrl, p2: tail_offset };

        let gap = self.squish_factor / slice_count as f32;
        let beat_span = self.tail_beat - self.head_beat;

        let mut links = Vec::with_capacity(self.links.len());
        for (i, data) in self.links.iter().enumerate() {
            let i = i+1;
            // interpolate spline

            let grid_pos = (spline.position(gap * i as f32) + head_pos).xy();
            let angle = spline.derivative(gap * i as f32).xy().to_angle() - 90f32.to_radians();

            let orientation = Quat::from_rotation_z(-angle);

            links.push(ChainNoteLink {
                grid_pos,
                beat: self.head_beat + (beat_span * (gap * i as f32)),
                orientation,
                spawn_orientation: data.spawn_orientation,
                color: self.color,
                index: self.index * 5 + i as u32 * 2,
                dissolve: self.dissolve,
            });
        }
        links
    }
}
impl GameObject for ChainNoteLink {
    fn beat(&self) -> f32 { self.beat }
    fn grid_pos(&self) -> Vec2 { self.grid_pos }
    fn get_orientation(&self) -> Quat { self.orientation }
    fn do_gravity(&self) -> bool { true }
    fn do_look(&self) -> bool { true }
    fn do_spawn_rotation(&self) -> bool { true }
    fn spawn_orientation(&self) -> Quat { self.spawn_orientation }
    fn get_instance(&self, clipping_plane: Vec4, model: Mat4, cs: &ColorScheme) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

struct BezierCurve {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
}
impl BezierCurve {
    fn position(&self, t: f32) -> Vec3 {
        let n = 1. - t;
        let x = n * n * self.p0.x + 2. * n * t * self.p1.x + t * t * self.p2.x;
        let y = n * n * self.p0.y + 2. * n * t * self.p1.y + t * t * self.p2.y;
        let z = n * n * self.p0.z + 2. * n * t * self.p1.z + t * t * self.p2.z;
        Vec3::new(x, y, z)
    }
    fn derivative(&self, t: f32) -> Vec3 {
        let n = 1. - t;
        let x = 2. * n * (self.p1.x - self.p0.x) + 2. * t * (self.p2.x - self.p1.x);
        let y = 2. * n * (self.p1.y - self.p0.y) + 2. * t * (self.p2.y - self.p1.y);
        let z = 2. * n * (self.p1.z - self.p0.z) + 2. * t * (self.p2.z - self.p1.z);
        Vec3::new(x, y, z)
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
        let mut chain_notes = Vec::new();

        #[allow(clippy::single_match)]
        match self {
            Self::V2(v2) => {
                for note in v2.notes.iter() {
                    match note {
                        v2::V2Note::Note(color_note) => {
                            let index = color_notes.len() as u32;
                            let color: NoteColor = color_note.typ.into();
                            color_notes.push(ColorNote {
                                spawn_orientation: get_random_spawn_quat(&mut rng),
                                beat: color_note.time,
                                color,
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
                                color: ObjectColor::default(),
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
                            Vec2::new(obst.line_index + ((obst.width / 2.) - 0.5), obst.line_layer - 0.8),
                            Vec3::new(obst.width, 5., obst.duration),
                        ),
                        super::data::ObstacleV2Type::Crouch => (
                            Vec2::new(obst.line_index + ((obst.width / 2.) - 0.5), obst.line_layer + 1.2),
                            Vec3::new(obst.width, 3., obst.duration)
                        ),
                        super::data::ObstacleV2Type::Free => (
                            Vec2::new(obst.line_index + ((obst.width / 2.) - 0.5), obst.line_layer - 0.8),
                            Vec3::new(obst.width, obst.height, obst.duration)
                        ),
                    };
                    obstacles.push(Obstacle {
                        beat: obst.beat,
                        color: ObjectColor::default(),
                        grid_pos,
                        size,
                        dissolve: 0.,
                        index,
                    })
                }
            },
            Self::V3(v3) => {
                for chain in v3.chains.iter() {
                    let index = chain_notes.len() as u32;
                    let mut links = Vec::with_capacity(chain.slice_count as usize);
                    for i in 0..chain.slice_count {
                        links.push(ChainNoteLinkData {
                            spawn_orientation: get_random_spawn_quat(&mut rng),
                            index: i as u32,
                        });
                    }
                    chain_notes.push(ChainNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        head_beat: chain.head_beat,
                        tail_beat: chain.tail_beat,
                        cut_direction: chain.head_cut_direction,
                        color: chain.color.into(),
                        head_grid_pos: Vec2::new(chain.head_line_index, chain.head_line_layer),
                        tail_grid_pos: Vec2::new(chain.tail_line_index, chain.tail_line_layer),
                        squish_factor: chain.squish_factor,
                        links,
                        dissolve: 0.,
                        index,
                    });
                }
                for note in v3.color_notes.iter() {
                    let index = color_notes.len() as u32;
                    let color: NoteColor = note.color.into();
                    let grid_pos = Vec2::new(note.line_index, note.line_layer);

                    if chain_notes.iter().any(|c| c.color == color && c.head_grid_pos == grid_pos && c.head_beat == note.beat) {
                        continue;
                    }

                    color_notes.push(ColorNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        beat: note.beat,
                        color,
                        cut_direction: note.cut_direction,
                        angle_offset: 0.,
                        grid_pos,
                        dissolve: 0.,
                        index,
                    });
                }
                for bomb in v3.bomb_notes.iter() {
                    let index = bomb_notes.len() as u32;
                    bomb_notes.push(BombNote {
                        beat: bomb.beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(bomb.line_index, bomb.line_layer),
                        dissolve: 0.,
                        index,
                    });
                }
                for obst in v3.obstacles.iter() {
                    let index = obstacles.len() as u32;
                    obstacles.push(Obstacle {
                        beat: obst.beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(obst.line_index + ((obst.width / 2.) - 0.5), obst.line_layer - 0.8),
                        size: Vec3::new(obst.width, obst.height, obst.duration),
                        dissolve: 0.,
                        index,
                    });
                }
            },
            Self::V4(v4) => {
                for chain in v4.chains.iter() {
                    let index = chain_notes.len() as u32;
                    let Some(head_data) = v4.color_notes_data.get(chain.head_note_metadata_index as usize) else { continue };
                    let Some(data) = v4.chains_data.get(chain.metadata_index as usize) else { continue };
                    let mut links = Vec::new();
                    for i in 0..data.slice_count {
                        links.push(ChainNoteLinkData {
                            spawn_orientation: get_random_spawn_quat(&mut rng),
                            index: i as u32,
                        });
                    }

                    chain_notes.push(ChainNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        head_beat: chain.head_beat,
                        tail_beat: chain.tail_beat,
                        cut_direction: head_data.cut_direction,
                        color: head_data.color.into(),
                        head_grid_pos: Vec2::new(head_data.line_index, head_data.line_layer),
                        tail_grid_pos: Vec2::new(data.tail_line_index, data.tail_line_layer),
                        squish_factor: data.squish_factor,
                        links,
                        dissolve: 0.,
                        index,
                    });
                }
                for note in v4.color_notes.iter() {
                    let index = color_notes.len() as u32;
                    let Some(data) = v4.color_notes_data.get(note.metadata_index as usize) else { continue };
                    let grid_pos = Vec2::new(data.line_index, data.line_layer);
                    let color: NoteColor = data.color.into();

                    if chain_notes.iter().any(|c| c.color == color && c.head_grid_pos == grid_pos && c.head_beat == note.beat) {
                        continue;
                    }
                    color_notes.push(ColorNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        beat: note.beat,
                        color,
                        cut_direction: data.cut_direction,
                        angle_offset: data.angle_offset as f32,
                        grid_pos,
                        dissolve: 0.,
                        index,
                    });
                }
                for bomb in v4.bomb_notes.iter() {
                    let index = bomb_notes.len() as u32;
                    let Some(data) = v4.bomb_notes_data.get(bomb.metadata_index as usize) else { continue };
                    bomb_notes.push(BombNote {
                        beat: bomb.beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(data.line_index, data.line_layer),
                        dissolve: 0.,
                        index
                    });
                }
                for obst in v4.obstacles.iter() {
                    let index = obstacles.len() as u32;
                    let Some(data) = v4.obstacles_data.get(obst.metadata_index as usize) else { continue };
                    obstacles.push(Obstacle {
                        beat: obst.beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(data.line_index + ((data.width / 2.) - 0.5), data.line_layer - 0.8),
                        size: Vec3::new(data.width, data.height, data.duration),
                        dissolve: 0.,
                        index,
                    });
                }
            }
        }

        Ok(BeatmapController {
            runtime_data: RuntimeData::new(diff_data.njs, info.bpm(), diff_data.njs_offset),
            color_notes,
            bomb_notes,
            obstacles,
            chain_notes,
        })
    }
}

