
pub mod spline;

use std::fmt::Debug;
use std::marker::PhantomData;

use beatcraft_editor_proc::hex;
use bs_mapping_data::easing::Easing;
use bs_mapping_data::{BpmRegion, Color, CutDirection};
use glam::{Mat4, Quat, Vec2, Vec3, Vec3Swizzles, Vec4};

use self::spline::BezierCurve;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl HitBox {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

pub trait CutDirectionExt {
    fn angle_degrees(&self) -> f32;
    fn world_angle_radians(&self) -> f32;
    fn to_quat(&self) -> Quat;
}
impl CutDirectionExt for CutDirection {
    fn angle_degrees(&self) -> f32 {
        match self {
            CutDirection::Up => 180f32,
            CutDirection::Down => 0.,
            CutDirection::Left => 90.,
            CutDirection::Right => -90.,
            CutDirection::UpLeft => 135.,
            CutDirection::UpRight => -135.,
            CutDirection::DownLeft => 45.,
            CutDirection::DownRight => -45.,
            CutDirection::Dot => 0.,
        }
    }
    fn world_angle_radians(&self) -> f32 {
        -self.angle_degrees().to_radians()
    }
    fn to_quat(&self) -> Quat {
        Quat::from_rotation_z(self.angle_degrees().to_radians())
    }
}

trait Lerp<V>
where
    V: Copy
        + std::ops::Mul<Output = V>
        + std::ops::Div<Output = V>
        + std::ops::Add<Output = V>
        + std::ops::Sub<Output = V>
{
    fn lerp(a: V, b: V, t: V) -> V {
        a + (b - a) * t
    }
    fn inv_lerp(a: V, b: V, x: V) -> V {
        (x - a) / (b - a)
    }
}

impl Lerp<f32> for f32 {}
impl Lerp<f64> for f64 {}

pub struct BeatmapController<ObjectSource> {
    pub runtime_data: RuntimeData,
    pub color_notes: Vec<ColorNote<ObjectSource>>,
    pub bomb_notes: Vec<BombNote<ObjectSource>>,
    pub obstacles: Vec<Obstacle<ObjectSource>>,
    pub chain_notes: Vec<ChainNote<ObjectSource>>,
    pub arcs: Vec<Arc<ObjectSource>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeData {
    pub njs: f32,
    pub bpm: f32,
    pub spawn_offset: f32,
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
    pub fn new(
        njs: f32,
        bpm: f32,
        spawn_offset: f32,
        bpm_regions: Vec<BpmRegion>,
        sample_count: usize,
        sample_rate: u32,
    ) -> Self {
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
        for BpmRegion {
            start_sample,
            end_sample,
            start_beat,
            end_beat,
        } in self.bpm_regions.iter()
        {
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
        for BpmRegion {
            start_sample,
            end_sample,
            start_beat,
            end_beat,
        } in self.bpm_regions.iter()
        {
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

        for BpmRegion {
            start_sample,
            end_sample,
            start_beat,
            end_beat,
        } in self.bpm_regions.iter()
        {
            if (*start_sample..=*end_sample).contains(&time_samples) {
                let seconds = (*end_sample - *start_sample) as f32 / self.sample_rate as f32;
                let beats = *end_beat - *start_beat;
                return beats / seconds * 60.;
            }
        }
        self.bpm
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

pub const NOTE_HITBOX: HitBox = HitBox::new(Vec3::splat(-0.25), Vec3::splat(0.25));
pub const CHAIN_HEAD_HITBOX: HitBox = HitBox::new(Vec3::new(-0.25, 0., -0.25), Vec3::splat(0.25));
pub const CHAIN_LINK_HITBOX: HitBox = HitBox::new(
    Vec3::new(-0.25, -0.25 * 2. / 16., -0.25),
    Vec3::new(0.25, 0.25 * 2. / 16., 0.25),
);

pub trait GameObject<ObjectSource>: Debug {
    fn beat(&self) -> f32;
    fn grid_pos(&self) -> Vec2;
    fn get_orientation(&self) -> Quat;
    fn spawn_orientation(&self) -> Quat {
        Quat::IDENTITY
    }
    fn do_gravity(&self) -> bool {
        false
    }
    fn do_look(&self) -> bool {
        false
    }
    fn do_spawn_rotation(&self) -> bool {
        false
    }
    fn duration(&self) -> f32 {
        0.
    }
    fn arrow_type(&self) -> ArrowType {
        ArrowType::None
    }
    fn lane_rotation_degrees(&self) -> f32 {
        0.
    }

    fn upcast_chain_head(&self) -> Option<&ChainNote<ObjectSource>> {
        None
    }

    fn editor_hitbox(&self, _beat_spacing: Option<f32>) -> HitBox {
        NOTE_HITBOX
    }

    fn animate_simple(
        &self,
        mut m: Mat4,
        beat: f32,
        _data: &RuntimeData,
        beats_before: u16,
        visible_beat_count: u8,
        beat_spacing: f32,
    ) -> Option<Mat4> {
        let b = self.beat();
        let pre = beats_before as f32;
        let post = visible_beat_count as f32 - pre;
        let s = b - post;
        let d = b + self.duration() + pre;

        if (s..d).contains(&beat) {
            let gp = self.grid_pos();

            let gp = Vec2::new(1.5 - gp.x, gp.y + 0.5) * 0.6;

            m *= Mat4::from_rotation_y(-self.lane_rotation_degrees().to_radians());

            m *= Mat4::from_translation(gp.extend((b - beat) * beat_spacing));
            m *= Mat4::from_quat(self.get_orientation());
            Some(m)
        } else {
            None
        }
    }

    fn animate_complex(&self, mut m: Mat4, beat: f32, data: &RuntimeData) -> Option<Mat4> {
        fn spawn_parabola(
            target_height: f32,
            base_height: f32,
            half_jump_distance: f32,
            t: f32,
        ) -> f32 {
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
        const LOOK_FREEZE_DISTANCE: f32 = 1.25;

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
                if in_pre_roll {
                    start_y
                } else {
                    spawn_parabola(gp.y, start_y, half_jump_distance, look_z.max(0.0))
                }
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
            left_note: hex!(#BF2F2F),
            right_note: hex!(#1F63A7),
            obstacle: hex!(#FF2F2F),
            lights: LightColors {
                primary: hex!(#BF2F2F),
                secondary: hex!(#1F63A7),
                white: hex!(#FFFFFF),
            },
            boost: LightColors {
                primary: hex!(#BF2F2F),
                secondary: hex!(#1F63A7),
                white: hex!(#FFFFFF),
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RawNoteColor {
    Red,
    Blue,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NoteColor {
    Red,
    Blue,
    CustomRed(Vec4),
    CustomBlue(Vec4),
}
impl NoteColor {
    pub fn color(&self, cs: &ColorScheme) -> Vec4 {
        match self {
            Self::Red => cs.left_note,
            Self::Blue => cs.right_note,
            Self::CustomRed(c) => *c,
            Self::CustomBlue(c) => *c,
        }
    }
    pub fn raw(&self) -> RawNoteColor {
        match self {
            Self::Red | Self::CustomRed(_) => RawNoteColor::Red,
            Self::Blue | Self::CustomBlue(_) => RawNoteColor::Blue,
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

pub trait ColorableObject
where
    Self: Sized,
{
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

#[derive(Debug)]
pub struct ColorNote<ObjectSource> {
    pub spawn_orientation: Quat,
    pub beat: f32,
    pub color: NoteColor,
    pub cut_direction: CutDirection,
    pub angle_offset_deg: f32,
    pub grid_pos: Vec2,
    pub lane_rotation_deg: f32,

    pub dissolve: f32,
    pub index: u32,

    pub source: ObjectSource,
}

#[derive(Debug)]
pub struct BombNote<ObjectSource> {
    pub beat: f32,
    pub color: ObjectColor<Self>,
    pub grid_pos: Vec2,
    pub lane_rotation_deg: f32,

    pub dissolve: f32,
    pub index: u32,

    pub source: ObjectSource,
}
impl<ObjectSource> ColorableObject for BombNote<ObjectSource> {
    fn color(col: &ObjectColor<Self>, _cs: &ColorScheme) -> Vec4 {
        match col {
            ObjectColor::Default(_) => hex!(#333333),
            ObjectColor::Custom(vec4) => *vec4,
        }
    }
}

#[derive(Debug)]
pub struct Obstacle<ObjectSource> {
    pub beat: f32,
    pub color: ObjectColor<Self>,
    pub grid_pos: Vec2,
    pub duration: f32,
    pub size: Vec3,
    pub lane_rotation_deg: f32,

    pub dissolve: f32,
    pub index: u32,

    pub noodle_logic: bool,

    pub source: ObjectSource,
}
impl<ObjectSource> ColorableObject for Obstacle<ObjectSource> {
    fn color(col: &ObjectColor<Self>, cs: &ColorScheme) -> Vec4 {
        match col {
            ObjectColor::Default(_) => cs.obstacle,
            ObjectColor::Custom(vec4) => *vec4,
        }
    }
}

#[derive(Debug)]
pub struct ChainNoteLinkData {
    pub spawn_orientation: Quat,
    pub index: u32,
}

#[derive(Debug)]
pub struct ChainNote<ObjectSource> {
    pub spawn_orientation: Quat,
    pub head_beat: f32,
    pub tail_beat: f32,
    pub head_lane_rotation_deg: f32,
    pub tail_lane_rotation_deg: f32,
    pub cut_direction: CutDirection,
    pub color: NoteColor,
    pub head_grid_pos: Vec2,
    pub tail_grid_pos: Vec2,
    pub squish_factor: f32,
    pub links: Vec<ChainNoteLinkData>,

    pub dissolve: f32,
    pub index: u32,

    pub source: ObjectSource,
}

#[derive(Debug)]
pub struct Arc<ObjectSource> {
    pub head_beat: f32,
    pub tail_beat: f32,
    pub head_cut_direction: CutDirection,
    pub tail_cut_direction: CutDirection,
    pub color: NoteColor,
    pub head_grid_pos: Vec2,
    pub tail_grid_pos: Vec2,
    pub head_magnitude: f32,
    pub tail_magnitude: f32,
    pub has_head_note: bool,
    pub has_tail_note: bool,

    pub source: ObjectSource,
}

impl<ObjectSource: Debug> GameObject<ObjectSource> for ColorNote<ObjectSource> {
    fn beat(&self) -> f32 {
        self.beat
    }
    fn grid_pos(&self) -> Vec2 {
        self.grid_pos
    }
    fn get_orientation(&self) -> Quat {
        self.cut_direction.to_quat() * Quat::from_rotation_z(self.angle_offset_deg.to_radians())
    }
    fn do_gravity(&self) -> bool {
        true
    }
    fn do_look(&self) -> bool {
        true
    }
    fn do_spawn_rotation(&self) -> bool {
        true
    }
    fn spawn_orientation(&self) -> Quat {
        self.spawn_orientation
    }
    fn lane_rotation_degrees(&self) -> f32 {
        self.lane_rotation_deg
    }
    fn arrow_type(&self) -> ArrowType {
        if self.cut_direction == CutDirection::Dot {
            ArrowType::Dot
        } else {
            ArrowType::Arrow
        }
    }
}

impl<ObjectSource: Debug> GameObject<ObjectSource> for BombNote<ObjectSource> {
    fn beat(&self) -> f32 {
        self.beat
    }
    fn grid_pos(&self) -> Vec2 {
        self.grid_pos
    }
    fn get_orientation(&self) -> Quat {
        Quat::IDENTITY
    }
    fn do_gravity(&self) -> bool {
        true
    }
    fn do_spawn_rotation(&self) -> bool {
        true
    }
    fn lane_rotation_degrees(&self) -> f32 {
        self.lane_rotation_deg
    }
}

impl<ObjectSource: Debug> GameObject<ObjectSource> for Obstacle<ObjectSource> {
    fn beat(&self) -> f32 {
        self.beat
    }
    fn grid_pos(&self) -> Vec2 {
        self.grid_pos + Vec2::new((self.size.x / 2.) - 0.5, -0.8)
    }
    fn get_orientation(&self) -> Quat {
        Quat::IDENTITY
    }
    fn duration(&self) -> f32 {
        self.duration
    }
    fn lane_rotation_degrees(&self) -> f32 {
        self.lane_rotation_deg
    }
    fn editor_hitbox(&self, beat_spacing: Option<f32>) -> HitBox {
        let z = if let Some(bs) = beat_spacing
            && !self.noodle_logic
        {
            self.duration * bs
        } else {
            self.size.z
        };
        HitBox::new(
            Vec3::new(-self.size.x / 2. * 0.6, 0., 0.),
            Vec3::new(self.size.x / 2. * 0.6, self.size.y * 0.6, z),
        )
    }
}

#[derive(Debug)]
pub struct ChainNoteLink<ObjectSource> {
    pub grid_pos: Vec2,
    pub beat: f32,
    pub orientation: Quat,
    pub lane_rotation_deg: f32,
    pub spawn_orientation: Quat,
    pub color: NoteColor,
    pub index: u32,
    pub dissolve: f32,
    pub _source: PhantomData<ObjectSource>,
}
impl<ObjectSource: Debug> GameObject<ObjectSource> for ChainNote<ObjectSource> {
    fn beat(&self) -> f32 {
        self.head_beat
    }
    fn grid_pos(&self) -> Vec2 {
        self.head_grid_pos
    }
    fn get_orientation(&self) -> Quat {
        self.cut_direction.to_quat()
    }
    fn do_gravity(&self) -> bool {
        true
    }
    fn do_look(&self) -> bool {
        true
    }
    fn do_spawn_rotation(&self) -> bool {
        true
    }
    fn spawn_orientation(&self) -> Quat {
        self.spawn_orientation
    }
    fn upcast_chain_head(&self) -> Option<&ChainNote<ObjectSource>> {
        Some(self)
    }
    fn lane_rotation_degrees(&self) -> f32 {
        self.head_lane_rotation_deg
    }
    fn editor_hitbox(&self, _bs: Option<f32>) -> HitBox {
        CHAIN_HEAD_HITBOX
    }
}
impl<ObjectSource> ChainNote<ObjectSource> {
    pub fn get_links(&self) -> Vec<ChainNoteLink<ObjectSource>> {
        let slice_count = self.links.len();
        if slice_count == 0 {
            return Vec::new();
        }

        let head_pos = self.head_grid_pos.extend(0.);
        let tail_offset = self.tail_grid_pos.extend(0.) - head_pos;
        let mag = tail_offset.length();
        let f = self.cut_direction.world_angle_radians() - 90f32.to_radians();
        let ctrl = Vec3::new(f.cos() * 0.5 * mag, f.sin() * 0.5 * mag, 0.);

        let spline = BezierCurve::new(Vec3::ZERO, ctrl, tail_offset);

        let gap = self.squish_factor / slice_count as f32;
        let beat_span = self.tail_beat - self.head_beat;

        let mut links = Vec::with_capacity(self.links.len());
        for (i, data) in self.links.iter().enumerate() {
            let i = i + 1;

            let grid_pos = (spline.position(gap * i as f32) + head_pos).xy();
            let angle = spline.derivative(gap * i as f32).xy().to_angle() - 90f32.to_radians();

            let orientation = Quat::from_rotation_z(-angle);

            links.push(ChainNoteLink {
                grid_pos,
                beat: self.head_beat + (beat_span * (gap * i as f32)),
                orientation,
                spawn_orientation: data.spawn_orientation,
                lane_rotation_deg: self.head_lane_rotation_deg,
                color: self.color,
                index: self.index * 5 + i as u32 * 2,
                dissolve: self.dissolve,
                _source: PhantomData
            });
        }
        links
    }
}
impl<ObjectSource: Debug> GameObject<ObjectSource> for ChainNoteLink<ObjectSource> {
    fn beat(&self) -> f32 {
        self.beat
    }
    fn grid_pos(&self) -> Vec2 {
        self.grid_pos
    }
    fn get_orientation(&self) -> Quat {
        self.orientation
    }
    fn do_gravity(&self) -> bool {
        true
    }
    fn do_look(&self) -> bool {
        true
    }
    fn do_spawn_rotation(&self) -> bool {
        true
    }
    fn spawn_orientation(&self) -> Quat {
        self.spawn_orientation
    }
    fn lane_rotation_degrees(&self) -> f32 {
        self.lane_rotation_deg
    }
    fn editor_hitbox(&self, _bs: Option<f32>) -> HitBox {
        CHAIN_LINK_HITBOX
    }
}

impl<ObjectSource: Debug> GameObject<ObjectSource> for Arc<ObjectSource> {
    fn beat(&self) -> f32 {
        self.head_beat
    }

    fn grid_pos(&self) -> Vec2 {
        self.head_grid_pos
    }

    fn get_orientation(&self) -> Quat {
        Quat::IDENTITY
    }

}



