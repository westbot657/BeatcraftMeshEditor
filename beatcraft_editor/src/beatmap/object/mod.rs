use glam::Mat4;

pub mod notes;
pub mod obstacles;
pub mod chains;

pub struct CoreGameObjectData {
    pub x: f32,
    pub y: f32,
    pub beat: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CutDirection {
    Up,
    Left,
    Down,
    Right,
    UpLeft,
    DownLeft,
    DownRight,
    UpRight,
    Dot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Red,
    Blue,
}

pub struct CoreColorNoteData {
    pub core: CoreGameObjectData,
    pub cut_direction: CutDirection,
    pub typ: NoteType,
    pub angle_offset: f32,
}

pub struct CoreBombNoteData {
    pub core: CoreGameObjectData,
}

pub struct CoreChainNoteData {
    pub core: CoreGameObjectData,
    pub tail_x: f32,
    pub tail_y: f32,
    pub tail_beat: f32,
    pub slice_count: u32,
    pub squish_factor: f32,
    pub cut_direction: CutDirection,
    pub typ: NoteType,
}

pub struct CoreObstacleData {
    pub core: CoreGameObjectData,
    pub width: f32,
    pub height: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcMidAnchorMode {
    Straight,
    CW,
    CCW,
}

pub struct CoreArcData {
    pub core: CoreGameObjectData,
    pub tail_x: f32,
    pub tail_y: f32,
    pub head_cut_direction: CutDirection,
    pub tail_cut_direction: CutDirection,
    pub head_magnitude: f32,
    pub tail_magnitude: f32,
    pub mid_anchor_mode: ArcMidAnchorMode,
}


pub trait PhysicalGameplayObject {
    fn render(&self, transform: Mat4);
}


