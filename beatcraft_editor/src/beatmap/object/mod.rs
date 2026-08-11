use std::f32;
use std::marker::PhantomData;
use std::ops::{Add, Div, Mul, Sub};

use glam::{Mat4, Quat, Vec2, Vec3, Vec3Swizzles, Vec4};
use rand::{rngs::ThreadRng, RngExt};

use crate::easing::Easing;
use crate::render::GameObjectInstanceData;

use super::BeatmapProjectDiff;
use super::data::{BeatmapDataError, BeatmapFile, BpmRegion, Color, CutDirection, InfoFile, v2};
use super::render::BeatmapRenderer;


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

pub struct BeatmapController {
    pub runtime_data: RuntimeData,
    pub color_notes: Vec<ColorNote>,
    pub bomb_notes: Vec<BombNote>,
    pub obstacles: Vec<Obstacle>,
    pub chain_notes: Vec<ChainNote>,
}

pub struct RuntimeData {
    pub njs: f32,
    bpm: f32,
    spawn_offset: f32,
    pub color_scheme: ColorScheme,
    pub bpm_regions: Vec<BpmRegion>,
    pub sample_count: usize,
    pub sample_rate: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeUnit {
    Beat(f32),
    Seconds(f32),
    Sample(usize),
}

impl RuntimeData {

    pub fn new(njs: f32, bpm: f32, spawn_offset: f32, bpm_regions: Vec<BpmRegion>, sample_count: usize, sample_rate: u32) -> Self {
        Self {
            njs,
            bpm,
            spawn_offset,
            color_scheme: Default::default(),
            bpm_regions,
            sample_count,
            sample_rate,
        }
    }

    pub fn base_jumps(&self) -> (f32, f32) {
        Self::calc_jumps(self.njs, self.bpm, self.spawn_offset)
    }

    pub fn jumps(&self, beat: f32) -> (f32, f32) {
        let bpm = self.bpm(TimeUnit::Beat(beat));
        Self::calc_jumps(self.njs, bpm, self.spawn_offset)
    }

    pub fn seconds_to_beat(&self, seconds: f32) -> f32 {
        let sample = (seconds * self.sample_rate as f32) as usize;
        for BpmRegion { start_sample, end_sample, start_beat, end_beat } in self.bpm_regions.iter() {
            if (*start_sample..=*end_sample).contains(&sample) {
                let x = f32::inv_lerp(*start_sample as f32, *end_sample as f32, sample as f32);
                return f32::lerp(*start_beat, *end_beat, x);
            }
        }
        seconds / (60. / self.bpm)
    }

    pub fn beat_to_seconds(&self, beat: f32) -> f32 {
        let sample = self.beat_to_sample(beat);
        sample as f32 / self.sample_rate as f32
    }

    fn beat_to_sample(&self, beat: f32) -> usize {
        for BpmRegion { start_sample, end_sample, start_beat, end_beat } in self.bpm_regions.iter() {
            if (*start_beat..=*end_beat).contains(&beat) {
                let x = f32::inv_lerp(*start_beat, *end_beat, beat);
                return f32::lerp(*start_sample as f32, *end_sample as f32, x) as usize;
            }
        }
        let s = beat * (60. / self.bpm);
        (s * self.sample_rate as f32) as usize
    }

    pub fn bpm(&self, time: TimeUnit) -> f32 {
        let time_samples = match time {
            TimeUnit::Sample(s) => s,
            TimeUnit::Seconds(s) => (s * self.sample_rate as f32) as usize,
            TimeUnit::Beat(b) => self.beat_to_sample(b),
        };

        for BpmRegion { start_sample, end_sample, start_beat, end_beat } in self.bpm_regions.iter() {
            if (*start_sample..=*end_sample).contains(&time_samples) {
                let seconds = (*end_sample - *start_sample) as f32 / self.sample_rate as f32;
                let beats = *end_beat - *start_beat;
                return beats / seconds * 60.;
            }
        }
        let seconds = self.sample_count as f32 / self.sample_rate as f32;

        seconds / (60. / self.bpm)
    }

    fn calc_jumps(njs: f32, bpm: f32, spawn_offset: f32) -> (f32, f32) {
        let mut hjd = 4.;
        let spb = 60. / bpm;

        let n2 = njs * spb;
        let mut n3 = n2 * hjd;
        while n3 > 17.999 {
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
    fn lane_rotation_degrees(&self) -> f32 { 0. }
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

            m *= Mat4::from_rotation_y(-self.lane_rotation_degrees().to_radians());

            m *= Mat4::from_translation(gp.extend((b - beat) * renderer.beat_spacing));
            m *= Mat4::from_quat(self.get_orientation());
            Some(m)
        } else {
            None
        }

    }

    fn animate_complex(&self, mut m: Mat4, beat: f32, data: &RuntimeData) -> Option<Mat4> {
        fn spawn_parabola(target_height: f32, base_height: f32, half_jump_distance: f32, t: f32) -> f32 {
            let d_sq = (half_jump_distance * half_jump_distance).max(1e-6);
            let movement_range = target_height - base_height;
            (-(movement_range / d_sq) * t * t + target_height).clamp(-9999., 9999.)
        }

        fn look_rotation(forward: Vec3, up: Vec3) -> Quat {
            let forward = forward.normalize();
            let right = up.cross(forward).normalize();
            let up = forward.cross(right);
            Quat::from_mat3(&glam::Mat3::from_cols(right, up, forward))
        }

        const ROTATION_ANIM_TIME: f32 = 0.4;

        const JUMP_FAR_Z: f32 = 500.;
        const PRE_ROLL_SECONDS: f32 = 1.0;
        const POST_ROLL_SECONDS: f32 = 1.0;
        const LOOK_FREEZE_DISTANCE: f32 = 1.0;

        let b = self.beat();
        let dur = self.duration();

        let (_, jd) = data.base_jumps();
        let njs = data.njs;
        let half_jump_distance = jd / 2.0;
        let reaction_time = half_jump_distance / njs;

        let object_time = data.beat_to_seconds(b);
        let dur_seconds = data.beat_to_seconds(b + dur) - object_time;

        let s_time = object_time - reaction_time;
        let d_time = object_time + dur_seconds + reaction_time;

        let pre_roll_start_time = s_time - PRE_ROLL_SECONDS;
        let post_roll_end_time = d_time + POST_ROLL_SECONDS;

        let s_ext = data.seconds_to_beat(pre_roll_start_time);
        let d_ext = data.seconds_to_beat(post_roll_end_time);

        if !(s_ext..d_ext).contains(&beat) {
            return None;
        }

        let current_time = data.beat_to_seconds(beat);

        let mut gp = self.grid_pos();
        gp = Vec2::new(1.5 - gp.x, gp.y + 0.5) * 0.6;

        let z_at_spawn = half_jump_distance;
        let z_at_despawn = (object_time - d_time) * njs;

        let in_pre_roll = current_time < s_time;
        let in_post_roll = current_time > d_time;

        let z = if in_pre_roll {
            let p = f32::inv_lerp(pre_roll_start_time, s_time, current_time).clamp(0., 1.);
            f32::lerp(JUMP_FAR_Z, z_at_spawn, p)
        } else if in_post_roll {
            let p = f32::inv_lerp(d_time, post_roll_end_time, current_time).clamp(0., 1.);
            f32::lerp(z_at_despawn, -JUMP_FAR_Z, p)
        } else {
            (object_time - current_time) * njs
        };

        let start_y = -0.3;

        // Parabola only applies while approaching (z >= 0, i.e. before the hit).
        // Once z has crossed 0 — whether still in the normal phase or already
        // into post-roll — height is just static at gp.y, since that's exactly
        // where the parabola ends up at z == 0 anyway. Clamping the input to
        // spawn_parabola at zero (rather than branching separately) makes this
        // fall out for free instead of needing a third case.
        let y = if self.do_gravity() {
            if in_pre_roll {
                start_y
            } else {
                spawn_parabola(gp.y, start_y, half_jump_distance, z.max(0.0))
            }
        } else {
            gp.y
        };

        let jump_mat = Mat4::from_translation(Vec3::new(gp.x, y, z));

        m *= Mat4::from_rotation_y(-self.lane_rotation_degrees().to_radians());
        m *= Mat4::from_translation(Vec3::new(0., 0.8, 1.));
        m *= jump_mat;

        let jump_progress = (object_time - current_time) / -reaction_time + 1.0;

        let base_rot = if jump_progress <= 0. {
            self.spawn_orientation()
        } else if jump_progress < ROTATION_ANIM_TIME {
            let t = Easing::easeOutSine.apply(jump_progress / ROTATION_ANIM_TIME);
            self.spawn_orientation().slerp(self.get_orientation(), t)
        } else {
            self.get_orientation()
        };

        let final_rot = if self.do_look() {
            let look_z = z.max(LOOK_FREEZE_DISTANCE);
            let look_y = if self.do_gravity() {
                if in_pre_roll { start_y } else { spawn_parabola(gp.y, start_y, half_jump_distance, look_z.max(0.0)) }
            } else {
                gp.y
            };
            let look_pos = Vec3::new(gp.x, look_y, look_z);

            let mut head = HEAD_POS;
            head.y = f32::lerp(head.y, look_pos.y, 0.8);
            let forward = (look_pos - head).normalize();
            let look = look_rotation(forward, base_rot * Vec3::Y);

            base_rot.slerp(look, jump_progress.clamp(0., 1.))
        } else {
            base_rot
        };

        m *= Mat4::from_quat(final_rot);

        Some(m)
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
    pub lane_rotation_deg: f32,

    pub dissolve: f32,
    pub index: u32,
}

pub struct BombNote {
    pub beat: f32,
    pub color: ObjectColor<Self>,
    pub grid_pos: Vec2,
    pub lane_rotation_deg: f32,

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
    pub lane_rotation_deg: f32,

    pub dissolve: f32,
    pub index: u32,
}
impl ColorableObject for Obstacle {
    fn color(col: &ObjectColor<Self>, cs: &ColorScheme) -> Vec4 {
        match col {
            ObjectColor::Default(_) => cs.obstacle,
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
    pub lane_rotation_deg: f32,
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
    fn lane_rotation_degrees(&self) -> f32 { self.lane_rotation_deg }
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
    fn lane_rotation_degrees(&self) -> f32 { self.lane_rotation_deg }
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
    fn lane_rotation_degrees(&self) -> f32 { self.lane_rotation_deg }
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
    lane_rotation_deg: f32,
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
    fn lane_rotation_degrees(&self) -> f32 { self.lane_rotation_deg }
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

            let grid_pos = (spline.position(gap * i as f32) + head_pos).xy();
            let angle = spline.derivative(gap * i as f32).xy().to_angle() - 90f32.to_radians();

            let orientation = Quat::from_rotation_z(-angle);

            links.push(ChainNoteLink {
                grid_pos,
                beat: self.head_beat + (beat_span * (gap * i as f32)),
                orientation,
                spawn_orientation: data.spawn_orientation,
                lane_rotation_deg: self.lane_rotation_deg,
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
    fn lane_rotation_degrees(&self) -> f32 { self.lane_rotation_deg }
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
    pub fn new(info: &InfoFile, diff_data: &BeatmapProjectDiff, diff: &BeatmapFile, bpm_regions: Vec<BpmRegion>, sample_count: usize, sample_rate: u32) -> Result<Self, BeatmapDataError> {

        diff.to_controller(info, diff_data, bpm_regions, sample_count, sample_rate)
    }
}

impl BeatmapFile {
    fn to_controller(&self, info: &InfoFile, diff_data: &BeatmapProjectDiff, bpm_regions: Vec<BpmRegion>, sample_count: usize, sample_rate: u32) -> Result<BeatmapController, BeatmapDataError> {

        let mut rng = rand::rng();
        let mut color_notes = Vec::new();
        let mut bomb_notes = Vec::new();
        let mut obstacles = Vec::new();
        let mut chain_notes = Vec::new();

        match self {
            Self::V2(v2) => {
                let mut rotations = Vec::new();
                for event in v2.events.iter() {
                    if let v2::V2Event::SpawnRotation(rot) = event {
                        rotations.push(*rot);
                    }
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
                for note in v2.notes.iter() {
                    match note {
                        v2::V2Note::Note(color_note) => {
                            let index = color_notes.len() as u32;
                            let color: NoteColor = color_note.typ.into();
                            let beat = color_note.time;
                            let mut lane_rotation_deg = 0i32;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early()) {
                                    lane_rotation_deg += rot.rotation_angle.get_degrees();
                                } else { break 'rot }
                            }

                            color_notes.push(ColorNote {
                                spawn_orientation: get_random_spawn_quat(&mut rng),
                                beat,
                                color,
                                cut_direction: color_note.cut_direction,
                                angle_offset: 0.,
                                grid_pos: Vec2::new(color_note.line_index, color_note.line_layer),
                                lane_rotation_deg: lane_rotation_deg as f32,
                                dissolve: 0.,
                                index
                            })
                        },
                        v2::V2Note::Bomb(bomb_note) => {
                            let index = bomb_notes.len() as u32;
                            let beat = bomb_note.beat;
                            let mut lane_rotation_deg = 0;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early()) {
                                    lane_rotation_deg += rot.rotation_angle.get_degrees();
                                } else { break 'rot }
                            }
                            bomb_notes.push(BombNote {
                                beat,
                                color: ObjectColor::default(),
                                grid_pos: Vec2::new(bomb_note.line_index, bomb_note.line_layer),
                                lane_rotation_deg: lane_rotation_deg as f32,
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
                    let beat = obst.beat;
                    let mut lane_rotation_deg = 0;
                    'rot: for rot in rotations.iter() {
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early()) {
                            lane_rotation_deg += rot.rotation_angle.get_degrees();
                        } else { break 'rot }
                    }
                    obstacles.push(Obstacle {
                        beat,
                        color: ObjectColor::default(),
                        grid_pos,
                        lane_rotation_deg: lane_rotation_deg as f32,
                        size,
                        dissolve: 0.,
                        index,
                    })
                }
            },
            Self::V3(v3) => {
                let mut rotations = Vec::new();
                for event in v3.rotation_events.iter() {
                    rotations.push(*event);
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());

                for chain in v3.chains.iter() {
                    let index = chain_notes.len() as u32;
                    let mut links = Vec::with_capacity(chain.slice_count as usize);
                    let beat = chain.head_beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat { break }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early()) {
                            lane_rotation_deg += rot.rotation;
                        }
                    }

                    for i in 0..chain.slice_count {
                        links.push(ChainNoteLinkData {
                            spawn_orientation: get_random_spawn_quat(&mut rng),
                            index: i as u32,
                        });
                    }
                    chain_notes.push(ChainNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        head_beat: beat,
                        tail_beat: chain.tail_beat,
                        lane_rotation_deg,
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

                    let beat = note.beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat { break }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early()) {
                            lane_rotation_deg += rot.rotation;
                        }
                    }

                    color_notes.push(ColorNote {
                        spawn_orientation: get_random_spawn_quat(&mut rng),
                        beat,
                        color,
                        cut_direction: note.cut_direction,
                        angle_offset: 0.,
                        grid_pos,
                        lane_rotation_deg,
                        dissolve: 0.,
                        index,
                    });
                }
                for bomb in v3.bomb_notes.iter() {
                    let index = bomb_notes.len() as u32;
                    let mut lane_rotation_deg = 0.;
                    let beat = bomb.beat;
                    for rot in rotations.iter() {
                        if rot.beat > beat { break }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early()) {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    bomb_notes.push(BombNote {
                        beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(bomb.line_index, bomb.line_layer),
                        lane_rotation_deg,
                        dissolve: 0.,
                        index,
                    });
                }
                for obst in v3.obstacles.iter() {
                    let index = obstacles.len() as u32;
                    let mut lane_rotation_deg = 0.;
                    let beat = obst.beat;
                    for rot in rotations.iter() {
                        if rot.beat > beat { break }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early()) {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    obstacles.push(Obstacle {
                        beat,
                        color: ObjectColor::default(),
                        grid_pos: Vec2::new(obst.line_index + ((obst.width / 2.) - 0.5), obst.line_layer - 0.8),
                        size: Vec3::new(obst.width, obst.height, obst.duration),
                        lane_rotation_deg,
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
                        lane_rotation_deg: chain.head_rotation_lane as f32,
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
                        lane_rotation_deg: note.rotation_lane as f32,
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
                        lane_rotation_deg: bomb.rotation_lane as f32,
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
                        lane_rotation_deg: obst.rotation_lane as f32,
                        dissolve: 0.,
                        index,
                    });
                }
            }
        }

        Ok(BeatmapController {
            runtime_data: RuntimeData::new(diff_data.njs, info.bpm(), diff_data.njs_offset, bpm_regions, sample_count, sample_rate),
            color_notes,
            bomb_notes,
            obstacles,
            chain_notes,
        })
    }
}

