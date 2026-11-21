use crate::math::HitBox;
use crate::spline::BezierCurve;
use crate::{ColorScheme, HEAD_POS, RuntimeData};
use beatcraft_editor_proc::hex;
use bs_mapping_data::easing::Easing;
use bs_mapping_data::{Color, CutDirection};
use glam::{Mat4, Quat, Vec2, Vec3, Vec3Swizzles, Vec4, FloatExt};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use tracks_rs::animation::track::PathPropertiesMap;
use bs_mapping_data::custom_data::{OptVec2, OptVec3};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ArrowType {
    None,
    Arrow,
    Dot,
    ChainDot,
}

pub const NOTE_HITBOX: HitBox = HitBox::new(Vec3::splat(-0.25), Vec3::splat(0.25));
pub const CHAIN_HEAD_HITBOX: HitBox = HitBox::new(Vec3::new(-0.25, 0., -0.25), Vec3::splat(0.25));
pub const CHAIN_LINK_HITBOX: HitBox = HitBox::new(
    Vec3::new(-0.25, -0.25 * 2. / 16., -0.25),
    Vec3::new(0.25, 0.25 * 2. / 16., 0.25),
);

pub trait GameObject<ObjectSource> {
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
    fn physical_length(&self) -> f32 {
        self.duration()
    }
    fn arrow_type(&self) -> ArrowType {
        ArrowType::None
    }
    fn lane_rotation(&self) -> Quat {
        Quat::IDENTITY
    }
    fn custom_njs(&self) -> Option<f32> {
        None
    }
    fn custom_njs_offset(&self) -> Option<f32> {
        None
    }

    fn animation(&self) -> Option<&PathPropertiesMap> {
        None
    }

    fn upcast_chain_head(&self) -> Option<&ChainNote<ObjectSource>> {
        None
    }

    fn editor_hitbox(&self, _beat_spacing: Option<f32>) -> HitBox {
        NOTE_HITBOX
    }

    /// Animates objects linearly, with no jumps or note-look effects
    fn animate_simple(
        &self,
        mut m: Mat4,
        beat: f32,
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

            m *= Mat4::from_quat(self.lane_rotation());

            m *= Mat4::from_translation(gp.extend((b - beat) * beat_spacing));
            m *= Mat4::from_quat(self.get_orientation());
            Some(m)
        } else {
            None
        }
    }

    /// Fully animates objects, including jumps, note-look, etc
    fn animate_complex(
        &self,
        mut m: Mat4,
        beat: f32,
        second: f32,
        data: &RuntimeData,
    ) -> Option<Mat4> {
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

        let (_, jd) = data.jumps(self.custom_njs_offset());
        let njs = self.custom_njs().unwrap_or(data.njs);
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
        let animation = self.animation();
        let normal_time = f32::inverse_lerp(data.seconds_to_beat(s_time), data.seconds_to_beat(d_time), beat).clamp(0., 1.);

        let mut gp = self.grid_pos();
        gp = Vec2::new(1.5 - gp.x, gp.y + 0.5) * 0.6;

        let z_at_spawn = half_jump_distance;
        let z_at_despawn = (object_time - d_time) * njs;

        let in_pre_roll = second < s_time;
        let in_post_roll = second > d_time;

        let z = if in_pre_roll {
            let p = f32::inverse_lerp(pre_roll_start_time, s_time, second).clamp(0., 1.);
            f32::lerp(JUMP_FAR_Z, z_at_spawn, p)
        } else if in_post_roll {
            let p = f32::inverse_lerp(d_time, post_roll_end_time, second).clamp(0., 1.);
            f32::lerp(z_at_despawn, -JUMP_FAR_Z - self.physical_length(), p)
        } else {
            (object_time - second) * njs
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

        let mut jump_vec = Vec3::new(gp.x, y, z);

        if let Some(a) = animation
            && let Some(val) = a.definite_position.interpolate(normal_time, &data.base_provider_context)
        {
            jump_vec *= Vec3::new(1., 1., 0.);
            jump_vec += val.as_vec3().unwrap();
        }

        let jump_mat = Mat4::from_translation(jump_vec);

        m *= Mat4::from_quat(self.lane_rotation());
        m *= Mat4::from_translation(Vec3::new(0., 0.8, 1.));
        m *= jump_mat;

        let jump_progress = (object_time - second) / -reaction_time + 1.0;

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

        if let Some(a) = animation {
            if let Some(val) = a.scale.interpolate(normal_time, &data.base_provider_context) {
                m *= Mat4::from_scale(val.as_vec3().unwrap())
            }
            if let Some(val) = a.local_rotation.interpolate(normal_time, &data.base_provider_context) {
                let q = val.as_quat().unwrap();
                m *= Mat4::from_quat(Quat::from_xyzw(-q.x, q.y, q.z, -q.w));
            }

        }

        m *= Mat4::from_quat(final_rot);

        Some(m)
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



#[derive(Clone)]
pub struct CustomData {
    pub custom_njs_offset: Option<f32>,
    pub custom_njs: Option<f32>,
    pub local_rotation: Quat,
    pub world_rotation: Quat,
    pub coordinates: OptVec2,
    pub animation: Option<PathPropertiesMap>,
    // pub track: Option<?>,
    // pub animation: Option<?>,
}

pub struct NoteCustomData {
    pub base: CustomData,

    pub disable_note_gravity: bool,
    pub disable_note_look: bool,
    
    pub spawn_effect: Option<bool>,
    pub disable_debris: bool,

}
impl Deref for NoteCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

pub struct BombCustomData {
    pub base: CustomData,

    pub disable_note_gravity: bool,
}
impl Deref for BombCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

pub struct ObstacleCustomData {
    pub base: CustomData,
}
impl Deref for ObstacleCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Clone)]
pub struct ChainCustomData {
    pub base: CustomData,
    
    pub disable_note_gravity: bool,
    pub disable_note_look: bool,
    
    pub spawn_effect: Option<bool>,
    pub disable_debris: bool,
    
    pub tail_coordinates: OptVec2,
}
impl Deref for ChainCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

pub struct ArcCustomData {
    pub base: CustomData,
}
impl Deref for ArcCustomData {
    type Target = CustomData;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

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

    pub custom_data: NoteCustomData,

    pub source: ObjectSource,
}

pub struct BombNote<ObjectSource> {
    pub beat: f32,
    pub color: ObjectColor<Self>,
    pub grid_pos: Vec2,
    pub lane_rotation_deg: f32,
    pub dissolve: f32,
    pub index: u32,

    pub custom_data: BombCustomData,

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

pub struct Obstacle<ObjectSource> {
    pub beat: f32,
    pub color: ObjectColor<Self>,
    pub grid_pos: Vec2,
    pub duration: f32,
    pub size: Vec3,
    pub lane_rotation_deg: f32,
    pub dissolve: f32,
    pub index: u32,

    pub custom_data: ObstacleCustomData,
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

    pub custom_data: ChainCustomData,

    pub source: ObjectSource,
}

pub struct ChainNoteLink<ObjectSource> {
    pub grid_pos: Vec2,
    pub beat: f32,
    pub orientation: Quat,
    pub lane_rotation_deg: f32,
    pub spawn_orientation: Quat,
    pub color: NoteColor,
    pub index: u32,
    pub dissolve: f32,

    pub custom_data: ChainCustomData,

    pub _source: PhantomData<ObjectSource>,
}

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

    pub noodle_data: ArcCustomData,

    pub source: ObjectSource,
}

impl<ObjectSource> GameObject<ObjectSource> for ColorNote<ObjectSource> {
    fn beat(&self) -> f32 {
        self.beat
    }
    fn grid_pos(&self) -> Vec2 {
        self.grid_pos
    }
    fn get_orientation(&self) -> Quat {
        self.custom_data.world_rotation *
            self.custom_data.local_rotation *
            self.cut_direction.to_quat() *
            Quat::from_rotation_z(self.angle_offset_deg.to_radians())
    }
    fn spawn_orientation(&self) -> Quat {
        self.spawn_orientation
    }
    fn do_gravity(&self) -> bool {
        !self.custom_data.disable_note_gravity
    }
    fn do_look(&self) -> bool {
        !self.custom_data.disable_note_look
    }
    fn do_spawn_rotation(&self) -> bool {
        true
    }
    fn arrow_type(&self) -> ArrowType {
        if self.cut_direction == CutDirection::Dot {
            ArrowType::Dot
        } else {
            ArrowType::Arrow
        }
    }
    fn lane_rotation(&self) -> Quat {
        self.custom_data.world_rotation *
        Quat::from_rotation_y(-self.lane_rotation_deg.to_radians())
    }
    fn custom_njs(&self) -> Option<f32> {
        self.custom_data.custom_njs
    }
    fn custom_njs_offset(&self) -> Option<f32> {
        self.custom_data.custom_njs_offset
    }
    fn animation(&self) -> Option<&PathPropertiesMap> {
        self.custom_data.animation.as_ref()
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
        self.custom_data.world_rotation * self.custom_data.local_rotation
    }
    fn do_gravity(&self) -> bool {
        !self.custom_data.disable_note_gravity
    }
    fn do_spawn_rotation(&self) -> bool {
        true
    }
    fn lane_rotation(&self) -> Quat {
        self.custom_data.world_rotation *
        Quat::from_rotation_y(-self.lane_rotation_deg.to_radians())
    }
    fn custom_njs(&self) -> Option<f32> {
        self.custom_data.custom_njs
    }
    fn custom_njs_offset(&self) -> Option<f32> {
        self.custom_data.custom_njs_offset
    }
    fn animation(&self) -> Option<&PathPropertiesMap> {
        self.custom_data.animation.as_ref()
    }
}

impl<ObjectSource: Debug> GameObject<ObjectSource> for Obstacle<ObjectSource> {
    fn beat(&self) -> f32 {
        self.beat
    }
    fn grid_pos(&self) -> Vec2 {
        let mut base = self.grid_pos + Vec2::new((self.size.x / 2.) - 0.5, -0.8);
        if let Some(x) = self.custom_data.coordinates.x() {
            base.x = x + 1.5 + (self.size.x/2.);
        }
        if let Some(y) = self.custom_data.coordinates.y() {
            base.y = y - 0.8;
        }
        base
    }
    fn get_orientation(&self) -> Quat {
        self.custom_data.world_rotation * self.custom_data.local_rotation
    }
    fn spawn_orientation(&self) -> Quat {
        self.custom_data.world_rotation * self.custom_data.local_rotation
    }
    fn duration(&self) -> f32 {
        self.duration
    }
    fn physical_length(&self) -> f32 {
        self.size.z
    }
    fn lane_rotation(&self) -> Quat {
        self.custom_data.world_rotation *
        Quat::from_rotation_y(-self.lane_rotation_deg.to_radians())
    }
    fn custom_njs(&self) -> Option<f32> {
        self.custom_data.custom_njs
    }
    fn custom_njs_offset(&self) -> Option<f32> {
        self.custom_data.custom_njs_offset
    }
    fn animation(&self) -> Option<&PathPropertiesMap> {
        self.custom_data.animation.as_ref()
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
    fn spawn_orientation(&self) -> Quat {
        self.spawn_orientation
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
    fn lane_rotation(&self) -> Quat {
        self.custom_data.world_rotation *
        Quat::from_rotation_y(-self.head_lane_rotation_deg.to_radians())
    }
    fn custom_njs(&self) -> Option<f32> {
        self.custom_data.custom_njs
    }
    fn custom_njs_offset(&self) -> Option<f32> {
        self.custom_data.custom_njs_offset
    }
    fn animation(&self) -> Option<&PathPropertiesMap> {
        self.custom_data.animation.as_ref()
    }
    fn upcast_chain_head(&self) -> Option<&ChainNote<ObjectSource>> {
        Some(self)
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
                custom_data: self.custom_data.clone(),
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
    fn spawn_orientation(&self) -> Quat {
        self.spawn_orientation
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
    fn lane_rotation(&self) -> Quat {
        self.custom_data.world_rotation *
        Quat::from_rotation_y(-self.lane_rotation_deg.to_radians())
    }
    fn animation(&self) -> Option<&PathPropertiesMap> {
        self.custom_data.animation.as_ref()
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

    fn duration(&self) -> f32 {
        self.tail_beat - self.head_beat
    }

}