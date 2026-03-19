use std::collections::HashMap;
use std::path::PathBuf;

use glam::{Quat, Vec2, Vec3, Vec4};

use crate::data::VertexId;
use crate::light_mesh::LightMesh;
use crate::render::{GpuMesh, Renderer};


#[derive(Copy, Clone)]
pub struct Camera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub fov: f32,
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EditorMode {
    View,
    Assembly,
    Edit,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ToolMode {
    Auto,
    Move,
    Rotate,
    Select,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DragState {
    None,
    Orbit,
    Pan,
    Vertex,
    Instance,
    InstanceRotation,
    Marquee(Vec4),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InstanceHandleType {
    Position,
    XRot,
    YRot,
    ZRot,
}

pub struct ViewMesh {
    pub path: PathBuf,
    pub data: LightMesh,
    pub gpu_bufs: HashMap<usize, GpuMesh>,
    pub visible: bool,
    pub placements: Vec<ViewPlacement>,
}

pub struct ViewPlacement {
    pub position: Vec3,
    pub rotation: Quat,
    pub count: u32,
    pub offset_pos: Vec3,
    pub offset_rot: Quat,
    pub visible: bool,
}

impl Default for ViewPlacement {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO, rotation: Quat::IDENTITY,
            count: 1,
            offset_pos: Vec3::ZERO, offset_rot: Quat::IDENTITY,
            visible: true,
        }
    }
}

pub struct Render {
    pub renderer: Renderer,
    pub assembly: Option<GpuMesh>,
    pub parts: HashMap<String, GpuMesh>,
    pub orphans: HashMap<String, GpuMesh>,
    pub sel_points: Option<GpuMesh>,
}

pub struct Editor {
    pub mesh: WorkingLightMesh,
    pub camera: Camera,
    pub part: Option<String>,
    pub part_names: Vec<String>,
}

pub struct Selection {
    pub verts: Vec<VertexId>,
    pub instances: Vec<usize>,
    pub hovered: usize,
}

pub struct Drag {
    pub state: DragState,
    pub drag_last: Vec2,
    pub drag_plane: (Vec3, Vec3),
    pub pre_drag_snapshot: Option<LightMesh>,
    pub rot_axis: Vec3,
    pub pending_desel: usize,
}

pub struct InnerCycle {
    pub last_pos: Vec2,
    pub candidates: Vec<usize>,
    pub current: usize,
}

pub struct ClickCycle {
    pub verticex: InnerCycle,
    pub instances: InnerCycle,
}

pub struct View {
    pub meshes: Vec<ViewMesh>,
    pub active: usize,
    pub session: Option<PathBuf>,
}

pub struct Assembly {
    pub handles: Vec<(Vec3, InstanceHandleType, usize)>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EulerSwizzle {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZYX,
    ZXY,
}

pub enum Clipboard {
    Mesh(LightMesh),
    Vec3 { v3: Vec3, mask: u8 },
    MultiVec3 { v3s: Vec<Vec3>, mask: u8 },
    Quat(Quat),
    MutliQuat(Vec<Quat>),
    Euler { v3: Vec3, mask: u8 },
    MultiEuler(Vec<(Vec3, u8)>),
    Instance(Vec<(PathBuf, ViewPlacement)>),
}

pub struct State {
    pub vp_rect: egui::Rect,
    pub wireframe: bool,
    pub show_grid: bool,
    pub show_verts: bool,
    pub euler_swizzle: EulerSwizzle,
    pub status: String,
    pub status_timer: f32,
    pub clipboard: Clipboard,
    pub dirty: bool,
}


pub struct App {
    pub mode: EditorMode,
    pub last_mode: EditorMode,
    pub tool: ToolMode,
    pub last_tool: ToolMode,
    pub render: Render,
    pub editor: Editor,
    pub selection: Selection,
    pub drag: Drag,
    pub click_cycle: ClickCycle,
    pub view: View,
    pub state: State,
    pub assembly: Assembly,
}

pub struct TriMeta {
    pub tag: String,
    pub channel: u8,
    pub material: u8,
    pub vname: Option<String>,
}

#[derive(Default, Debug)]
pub struct WorkingLightMesh {
    pub mesh: LightMesh,
    pub path: Option<PathBuf>,
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub colors: Vec<i32>,
    pub tri_indices: Vec<[usize; 3]>,
}

impl WorkingLightMesh {

    pub fn new(mesh: LightMesh, path: Option<PathBuf>) -> Self {
        let mut s = Self {
            mesh,
            path,
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            tri_indices: Vec::new(),
        };
        s.resolve();
        s
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        Ok(Self::new(LightMesh::load(&path)?, Some(path)))
    }

    pub fn resolve(&mut self) {

    }

}


impl App {
    pub fn new(cc: &eframe::CreationContext, path: Option<PathBuf>) -> Self {
        let mut md = if let Some(p) = path {
            WorkingLightMesh::load(p).unwrap_or_default()
        } else {
            WorkingLightMesh::default()
        };
    }
}


