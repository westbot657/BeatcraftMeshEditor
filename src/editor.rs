use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::{fs, mem};

use eframe::glow::Context;
use egui::{Key, Response};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

use crate::RefDuper;
use crate::data::{NormalId, SessionData, SessionMeshData, SessionPlacementData, UvId, VertexId};
use crate::light_mesh::{
    LightMesh, LightMeshMetaSnapshot, LightMeshPartSnapshot, LightMeshPlacementSnapshot, Part,
};
use crate::render::{GpuMesh, InstanceData, Renderer};

#[derive(Copy, Clone)]
pub struct Camera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub fov: f32,
}

impl Camera {
    pub fn eye(&self) -> Vec3 {
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();
        let cy = self.yaw.cos();
        let sy = self.yaw.sin();
        self.target + self.dist * Vec3::new(cp * sy, sp, cp * cy)
    }
    pub fn view_mat(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }
    pub fn proj_mat(&self, w: f32, h: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, (w / h).max(0.001), 0.1, 5000.0)
    }
    pub fn vp(&self, w: f32, h: f32) -> Mat4 {
        self.proj_mat(w, h) * self.view_mat()
    }
    pub fn left(&self) -> Vec3 {
        let m = self.view_mat();
        Vec3::new(m.col(0).x, m.col(1).x, m.col(2).x)
    }
    pub fn up_vec(&self) -> Vec3 {
        let m = self.view_mat();
        Vec3::new(m.col(0).y, m.col(1).y, m.col(2).y)
    }
    pub fn forward(&self) -> Vec3 {
        (self.target - self.eye()).normalize()
    }
    pub fn pick_radius(&self, screen_px: f32, h: f32) -> f32 {
        let view_h = 2.0 * self.dist * (self.fov / 2.0).tan();
        (screen_px / h) * view_h
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw: 135f32.to_radians(),
            pitch: 45f32.to_radians(),
            dist: 50.,
            fov: 100f32.to_radians(),
        }
    }
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
    pub gpu_bufs: (HashMap<String, GpuMesh>, Option<GpuMesh>),
    pub visible: bool,
    pub placements: Vec<ViewPlacement>,
}

#[derive(Debug)]
pub struct ViewPlacement {
    pub position: Vec3,
    pub rotation: Quat,
    pub count: u32,
    pub offset_pos: Vec3,
    pub offset_rot: Quat,
    pub visible: bool,
}

#[derive(Debug)]
pub struct ViewPlacementsSnapshot {
    pub idx: usize,
    pub placements: Vec<ViewPlacement>,
}

impl Default for ViewPlacement {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            count: 1,
            offset_pos: Vec3::ZERO,
            offset_rot: Quat::IDENTITY,
            visible: true,
        }
    }
}

impl LightMesh {
    pub fn into_view_mesh(self, path: PathBuf, gl: &Context) -> ViewMesh {
        let mut v = ViewMesh::new(path, self);
        v.rebuild(gl);
        v
    }
}

impl ViewMesh {
    pub fn new(path: PathBuf, light_mesh: LightMesh) -> Self {
        Self {
            path,
            data: light_mesh,
            gpu_bufs: (HashMap::new(), None),
            visible: true,
            placements: Vec::new(),
        }
    }

    pub fn rebuild(&mut self, gl: &Context) {
        let v = mem::take(&mut self.gpu_bufs.0);
        self.gpu_bufs.0 = GpuMesh::set_from_hashmap(gl, &self.data, v);
        let full = self
            .gpu_bufs
            .1
            .get_or_insert_with(|| GpuMesh::new(gl, &[], &[], &[]));
        full.set_from_full_light_mesh(gl, &self.data);
    }

    pub fn render_view_placements(&self, calls: &mut Vec<InstanceData>) -> Option<&GpuMesh> {
        if self.visible {
            if self.placements.is_empty() {
                calls.push(InstanceData::new(Mat4::IDENTITY, 1., Some([0.2, 0.2, 0.2])));
            } else {
                for placement in self.placements.iter() {
                    let mut pos = placement.position;
                    let mut rot = placement.rotation;
                    for _ in 0..placement.count {
                        calls.push(InstanceData::new(
                            Mat4::from_translation(pos) * Mat4::from_quat(rot),
                            1.,
                            Some([0.2, 0.2, 0.2]),
                        ));
                        pos += placement.offset_pos;
                        rot *= placement.offset_rot;
                    }
                }
            }
            self.gpu_bufs.1.as_ref()
        } else {
            None
        }
    }

    pub fn render_assembly(&self, calls: &mut Vec<InstanceData>) -> Option<&GpuMesh> {
        calls.push(InstanceData::new(Mat4::IDENTITY, 1., Some([0.2, 0.2, 0.2])));
        self.gpu_bufs.1.as_ref()
    }

    pub fn destroy(self, gl: &Context) {
        if let Some(m) = self.gpu_bufs.1 {
            m.destroy(gl);
        }
        for m in self.gpu_bufs.0.into_values() {
            m.destroy(gl);
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
    pub mesh: Option<usize>,
    pub camera: Camera,
    pub part: Option<usize>,
    pub hovered: Option<VertexId>,
}

pub enum Selection {
    None,
    Vertices(Vec<VertexId>),
    Instances(Vec<usize>),
}

pub struct Drag {
    pub state: DragState,
    pub drag_last: Vec2,
    pub drag_ref: Vec3,
    pub pre_drag_snapshot: Option<Part>,
    pub rot_axis: Vec3,
}

pub struct InnerCycle {
    pub last_pos: Vec2,
    pub candidates: Vec<VertexId>,
    pub current: usize,
}

pub struct ClickCycle {
    pub vertices: InnerCycle,
    pub instances: InnerCycle,
}

pub struct View {
    pub meshes: Vec<ViewMesh>,
    pub session: Option<PathBuf>,
    pub camera: Camera,
}

pub struct Assembly {
    pub handles: Vec<(Vec3, InstanceHandleType, usize)>,
    pub hovered: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationDisplayMode {
    Quaternion,
    Euler(EulerSwizzle),
}

impl Default for RotationDisplayMode {
    fn default() -> Self {
        Self::Euler(EulerSwizzle::YXZ)
    }
}

impl RotationDisplayMode {
    pub fn cycle(&self) -> Self {
        match self {
            Self::Quaternion => Self::Euler(EulerSwizzle::XYZ),
            Self::Euler(EulerSwizzle::XYZ) => Self::Euler(EulerSwizzle::XZY),
            Self::Euler(EulerSwizzle::XZY) => Self::Euler(EulerSwizzle::YXZ),
            Self::Euler(EulerSwizzle::YXZ) => Self::Euler(EulerSwizzle::YZX),
            Self::Euler(EulerSwizzle::YZX) => Self::Euler(EulerSwizzle::ZYX),
            Self::Euler(EulerSwizzle::ZYX) => Self::Euler(EulerSwizzle::ZXY),
            Self::Euler(EulerSwizzle::ZXY) => Self::Quaternion,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EulerSwizzle {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZYX,
    ZXY,
}

impl EulerSwizzle {
    pub fn to_glam(self) -> glam::EulerRot {
        match self {
            EulerSwizzle::XYZ => glam::EulerRot::XYZ,
            EulerSwizzle::XZY => glam::EulerRot::XZY,
            EulerSwizzle::YXZ => glam::EulerRot::YXZ,
            EulerSwizzle::YZX => glam::EulerRot::YZX,
            EulerSwizzle::ZYX => glam::EulerRot::ZYX,
            EulerSwizzle::ZXY => glam::EulerRot::ZXY,
        }
    }
}

impl From<EulerSwizzle> for glam::EulerRot {
    fn from(value: EulerSwizzle) -> Self {
        value.to_glam()
    }
}

impl EulerSwizzle {
    pub fn label(&self) -> &'static str {
        match self {
            Self::XYZ => "XYZ",
            Self::XZY => "XZY",
            Self::YXZ => "YXZ",
            Self::YZX => "YZX",
            Self::ZYX => "ZYX",
            Self::ZXY => "ZXY",
        }
    }

    pub fn names(&self) -> [&'static str; 3] {
        match self {
            EulerSwizzle::XYZ => ["x", "y", "z"],
            EulerSwizzle::XZY => ["x", "z", "y"],
            EulerSwizzle::YXZ => ["y", "x", "z"],
            EulerSwizzle::YZX => ["y", "z", "x"],
            EulerSwizzle::ZYX => ["z", "y", "x"],
            EulerSwizzle::ZXY => ["z", "x", "y"],
        }
    }
}

pub enum Clipboard {
    None,
    Mesh(Box<LightMesh>),
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
    pub gl: Arc<Context>,
    pub ui: UiState,
}

pub struct PartCollapseToggles {
    pub placements: bool,
    pub data: bool,
    pub textures: bool,
    pub settings: bool,
    pub credits: bool,
    pub placement_parts: HashMap<usize, ([bool; 3], RotationDisplayMode)>, // part toggle, remap toggle, scale_lock, rotation mode
    pub datas: HashMap<usize, bool>,
}

impl Default for PartCollapseToggles {
    fn default() -> Self {
        Self {
            placements: false,
            data: false,
            textures: true,
            settings: true,
            credits: true,
            placement_parts: HashMap::default(),
            datas: HashMap::default(),
        }
    }
}

#[derive(Default)]
pub struct UiState {
    pub view_mesh: Option<usize>,
    pub open_mesh_channel: Option<mpsc::Receiver<Vec<PathBuf>>>,
    pub open_session_channel: Option<mpsc::Receiver<PathBuf>>,
    pub collapsed: HashMap<usize, Vec<bool>>,
    pub view_rotation_modes: HashMap<usize, Vec<[RotationDisplayMode; 2]>>,
    pub assembly_collapsed: HashMap<PathBuf, PartCollapseToggles>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PartId {
    pub view_idx: usize,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataSwap<T: std::fmt::Debug + PartialEq + Eq + Clone> {
    pub from: T,
    pub to: T,
}

impl<T: std::fmt::Debug + PartialEq + Eq + Clone> DataSwap<T> {
    fn invert(self) -> Self {
        Self {
            from: self.to,
            to: self.from
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Rename {
    DataTag { view_idx: usize, swap: DataSwap<String> },
    Part { view_idx: usize, swap: DataSwap<String> },
    /// Can rename any vertex between indexed and named.
    Vertex { part: PartId, swap: DataSwap<VertexId> },
    /// Can rename any uv including jumping between indexed and named.
    Uv { part: PartId, swap: DataSwap<UvId> },
    /// Can rename any normal between indexed and named.
    Normal { part: PartId, swap: DataSwap<NormalId> },
    /// Can rename compute vertices
    ComputeVertex { part: PartId, swap: DataSwap<String> },
    /// Can rename compute normals
    ComputeNormal { part: PartId, swap: DataSwap<String> },
}

impl Rename {
    pub fn invert(self) -> Self {
        match self {
            Self::DataTag { view_idx, swap } => Self::DataTag { view_idx, swap: swap.invert() },
            Self::Part { view_idx, swap } => Self::Part { view_idx, swap: swap.invert() },
            Self::Uv { part, swap } => Self::Uv { part, swap: swap.invert() },
            Self::Normal { part, swap } => Self::Normal { part, swap: swap.invert() },
            Self::Vertex { part, swap } => Self::Vertex { part, swap: swap.invert() },
            Self::ComputeNormal { part, swap } => Self::ComputeNormal { part, swap: swap.invert() },
            Self::ComputeVertex { part, swap } => Self::ComputeVertex { part, swap: swap.invert() },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexMutationRequest {
    InsertVertex { part: PartId, index: usize, vertex: Vec3 },
    InsertUv { part: PartId, index: usize, uv: Vec2 },
    InsertNormal { part: PartId, index: usize, normal: Vec3 },
    RemoveVertex { part: PartId, index: usize },
    RemoveUv { part: PartId, index: usize },
    RemoveNormal { part: PartId, index: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub enum IndexMutation {
    InsertedVertex { part: PartId, index: usize },
    InsertedUv { part: PartId, index: usize },
    InsertedNormal { part: PartId, index: usize },
    RemovedVertex { part: PartId, index: usize, vertex: Vec3 },
    RemovedUv { part: PartId, index: usize, uv: Vec2 },
    RemovedNormal { part: PartId, index: usize, normal: Vec3 },
}

impl IndexMutation {
    fn inverse_request(self) -> IndexMutationRequest {
        match self {
            Self::InsertedVertex { part, index } =>
                IndexMutationRequest::RemoveVertex { part, index },
            Self::InsertedUv { part, index } =>
                IndexMutationRequest::RemoveUv { part, index },
            Self::InsertedNormal { part, index } =>
                IndexMutationRequest::RemoveNormal { part, index },
            Self::RemovedVertex { part, index, vertex } =>
                IndexMutationRequest::InsertVertex { part, index, vertex},
            Self::RemovedUv { part, index, uv } =>
                IndexMutationRequest::InsertUv { part, index, uv },
            Self::RemovedNormal { part, index, normal } =>
                IndexMutationRequest::InsertNormal { part, index, normal },
        }
    }
}

#[derive(Debug)]
pub enum HistoryEntry {
    MeshPart(LightMeshPartSnapshot),
    MeshMeta(LightMeshMetaSnapshot),
    MeshPlacement(LightMeshPlacementSnapshot),
    ViewPlacement(ViewPlacementsSnapshot),
    IndexMutation(IndexMutation),
    Rename(Rename),
    MutliStep(Vec<HistoryEntry>),
}

pub struct History {
    pub history: VecDeque<HistoryEntry>,
    pub future: VecDeque<HistoryEntry>,
    pub limit: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HistoryCycleDir {
    Past,
    Future,
}

impl History {
    pub fn add_history(&mut self, entry: HistoryEntry) {
        println!("Saving history: {:?}", entry);
        self.history.push_back(entry);
        self.future.clear();
        if self.history.len() > self.limit {
            let _ = self.history.pop_front();
        }
    }

    fn process_history(
        &mut self,
        editor: &mut App,
        gl: &Context,
        entry: HistoryEntry
    ) -> HistoryEntry {
        match entry {
            HistoryEntry::MeshPart(LightMeshPartSnapshot { idx, name, part }) => {
                let m = editor.view.meshes.get_mut(idx).unwrap();
                let current = m.data.parts.insert(name.clone(), *part).unwrap();
                m.rebuild(gl);
                HistoryEntry::MeshPart(LightMeshPartSnapshot {
                    idx,
                    name,
                    part: Box::new(current),
                })
            }
            HistoryEntry::MeshMeta(LightMeshMetaSnapshot {
                idx,
                mut credits,
                mut textures,
                mut data,
                mut cull,
            }) => {
                let m = editor.view.meshes.get_mut(idx).unwrap();
                mem::swap(&mut credits, &mut m.data.credits);
                mem::swap(&mut textures, &mut m.data.textures);
                mem::swap(&mut data, &mut m.data.data);
                mem::swap(&mut cull, &mut m.data.cull);
                m.rebuild(gl);
                HistoryEntry::MeshMeta(LightMeshMetaSnapshot {
                    idx,
                    credits,
                    textures,
                    data,
                    cull,
                })
            }
            HistoryEntry::MeshPlacement(LightMeshPlacementSnapshot {
                view_idx,
                mut placements,
            }) => {
                let m = editor.view.meshes.get_mut(view_idx).unwrap();
                mem::swap(&mut placements, &mut m.data.placements);
                m.rebuild(gl);
                HistoryEntry::MeshPlacement(LightMeshPlacementSnapshot {
                    view_idx,
                    placements,
                })
            }
            HistoryEntry::ViewPlacement(ViewPlacementsSnapshot {
                idx,
                mut placements,
            }) => {
                let m = editor.view.meshes.get_mut(idx).unwrap();
                mem::swap(&mut placements, &mut m.placements);
                m.rebuild(gl);
                HistoryEntry::ViewPlacement(ViewPlacementsSnapshot { idx, placements })
            }
            HistoryEntry::IndexMutation(idx_mut) => {
                HistoryEntry::IndexMutation(editor.mutate_index_inner(idx_mut.inverse_request()))
            }
            HistoryEntry::Rename(rename) => {

                editor.rename(rename.clone());

                HistoryEntry::Rename(rename.invert())
            }
            HistoryEntry::MutliStep(mut steps) => {
                let mut out = Vec::new();
                while let Some(step) = steps.pop() {
                    out.push(self.process_history(editor, gl, step));
                }
                HistoryEntry::MutliStep(out)
            }
        }
    }

    pub fn cycle_history(
        &mut self,
        dir: HistoryCycleDir,
        editor: &mut App,
        gl: &Context,
    ) {
        let front = match dir {
            HistoryCycleDir::Future => &mut self.future,
            HistoryCycleDir::Past => &mut self.history,
        };

        let save = if let Some(restore) = front.pop_back() {
            println!("Restoring {:?}", restore);
            self.process_history(editor, gl, restore)
        } else { return };

        let back = match dir {
            HistoryCycleDir::Future => &mut self.history,
            HistoryCycleDir::Past => &mut self.future,
        };
        back.push_back(save);
    }
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
    pub history: History,
}

pub struct TriMeta {
    pub tag: String,
    pub channel: u8,
    pub material: u8,
    pub vname: Option<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext, path: Option<PathBuf>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "source-code-pro".to_string(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "./assets/fonts/SourceCodePro-Regular.ttf"
            ))),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, String::from("source-code-pro"));

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, String::from("source-code-pro"));

        cc.egui_ctx.set_fonts(fonts);

        let gl = Arc::clone(cc.gl.as_ref().expect("GL context not found"));

        let gl2 = Arc::clone(&gl);

        let mut s = Self {
            mode: EditorMode::View,
            last_mode: EditorMode::View,
            tool: ToolMode::Auto,
            last_tool: ToolMode::Auto,
            render: Render {
                renderer: Renderer::new(&gl2).unwrap(),
                assembly: None,
                parts: HashMap::new(),
                orphans: HashMap::new(),
                sel_points: None,
            },
            editor: Editor {
                mesh: None,
                camera: Camera::default(),
                part: None,
                hovered: None,
            },
            selection: Selection::None,
            drag: Drag {
                state: DragState::None,
                drag_last: Vec2::ZERO,
                drag_ref: Vec3::ZERO,
                pre_drag_snapshot: None,
                rot_axis: Vec3::ZERO,
            },
            click_cycle: ClickCycle {
                vertices: InnerCycle {
                    last_pos: Vec2::ZERO,
                    candidates: Vec::new(),
                    current: 0,
                },
                instances: InnerCycle {
                    last_pos: Vec2::ZERO,
                    candidates: Vec::new(),
                    current: 0,
                },
            },
            view: View {
                meshes: Vec::new(),
                session: None,
                camera: Camera::default(),
            },
            state: State {
                vp_rect: egui::Rect {
                    min: egui::Pos2 { x: 0., y: 0. },
                    max: egui::Pos2 { x: 0., y: 0. },
                },
                wireframe: true,
                show_grid: true,
                show_verts: true,
                euler_swizzle: EulerSwizzle::YXZ,
                status: "".to_string(),
                status_timer: 0.,
                clipboard: Clipboard::None,
                dirty: false,
                gl,
                ui: UiState::default(),
            },
            assembly: Assembly {
                handles: Vec::new(),
                hovered: None,
            },
            history: History {
                history: VecDeque::new(),
                future: VecDeque::new(),
                limit: 200,
            },
        };

        if let Some(p) = path
            && s.load_session(&p, &gl2).is_err()
        {
            let _ = s.load_meshes(vec![p], &gl2);
        }

        s
    }

    pub fn load_meshes_to_vec(paths: Vec<PathBuf>, gl: &Context) -> anyhow::Result<Vec<ViewMesh>> {
        let mut out = Vec::new();
        for path in paths {
            out.push(LightMesh::load(&path)?.into_view_mesh(path, gl))
        }
        Ok(out)
    }

    pub fn load_meshes(&mut self, paths: Vec<PathBuf>, gl: &Context) -> anyhow::Result<()> {
        let mut meshes = Self::load_meshes_to_vec(paths, gl)?;
        // Clear out old meshes with same paths as new meshes
        self.view.meshes.retain(|view_mesh| {
            !meshes
                .iter()
                .any(|new_mesh| new_mesh.path == view_mesh.path)
        });
        self.view.meshes.append(&mut meshes);
        Ok(())
    }

    pub fn cam(&mut self) -> &mut Camera {
        match self.mode {
            EditorMode::View => &mut self.view.camera,
            EditorMode::Assembly => &mut self.editor.camera,
            EditorMode::Edit => &mut self.editor.camera,
        }
    }

    pub fn rebuild_meshes(&mut self, gl: &Context) {
        for view_mesh in self.view.meshes.iter_mut() {
            view_mesh.rebuild(gl);
        }
    }

    pub fn block_input(&self) -> bool {
        self.state.ui.open_mesh_channel.is_some() || self.state.ui.open_session_channel.is_some()
    }

    pub fn handle_keys(&mut self, ctx: &egui::Context, gl: &Context) {
        if self.block_input() {
            return;
        }

        let input = ctx.input(|i| i.clone());
        let ctrl = input.modifiers.ctrl;
        let shift = input.modifiers.shift;

        if ctrl && input.key_pressed(Key::Z) {
            if shift {
                self.redo(gl);
            } else {
                self.undo(gl);
            }
        }
        if ctrl && input.key_pressed(Key::S) {
            match self.mode {
                EditorMode::View => {}
                EditorMode::Assembly | EditorMode::Edit => {}
            }
        }
        if ctrl && input.key_pressed(Key::C) {
            // TODO
        }
        if ctrl && input.key_pressed(Key::V) {
            // TODO
        }

        if input.key_pressed(Key::W) {
            self.state.wireframe = !self.state.wireframe;
        }
        if input.key_pressed(Key::G) {
            self.state.show_grid = !self.state.show_grid;
        }
        if input.key_pressed(Key::V) {
            self.state.show_verts = !self.state.show_verts;
        }
        if input.key_pressed(Key::I) {
            self.last_mode = self.mode;
            self.mode = EditorMode::View;
        }

        if input.key_pressed(Key::Escape) {
            self.selection = Selection::None;
            self.upload_selection_points(gl);
        }

        match self.mode {
            EditorMode::View => {}
            EditorMode::Assembly => {
                if input.key_pressed(Key::E) {
                    self.last_mode = self.mode;
                    self.mode = EditorMode::Edit;
                    if self.editor.part.is_none()
                        && let Some(sel) = self.editor.mesh
                        && let Some(mesh) = self.view.meshes.get(sel)
                        && !mesh.data.parts.is_empty()
                    {
                        self.editor.part = Some(0);
                    }
                }
            }
            EditorMode::Edit => {
                if input.key_pressed(Key::E) {
                    self.last_mode = self.mode;
                    self.mode = EditorMode::Assembly;
                }
                if input.key_pressed(Key::OpenBracket)
                    && let Some(sel) = self.editor.mesh
                    && let Some(mesh) = self.view.meshes.get(sel)
                {
                    if mesh.data.part_names.is_empty() {
                        self.editor.part = None;
                    } else {
                        let l = mesh.data.part_names.len();
                        self.editor.part =
                            Some(self.editor.part.map(|x| (x + l - 1) % l).unwrap_or(0));
                    }
                }
                if input.key_pressed(Key::CloseBracket)
                    && let Some(sel) = self.editor.mesh
                    && let Some(mesh) = self.view.meshes.get(sel)
                {
                    if mesh.data.part_names.is_empty() {
                        self.editor.part = None;
                    } else {
                        self.editor.part = Some(
                            self.editor
                                .part
                                .map(|x| (x + 1) % mesh.data.part_names.len())
                                .unwrap_or(0),
                        );
                    }
                }
            }
        }
    }

    pub fn handle_3d_input(&mut self, resp: &Response, ctx: &egui::Context, gl: &Context) {
        if self.block_input() {
            return;
        }

        let rect = self.state.vp_rect;
        let w = rect.width();
        let h = rect.height();

        let pointer = ctx.input(|i| i.pointer.clone());
        let shift = ctx.input(|i| i.modifiers.shift);
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let primary_pressed = resp.drag_started_by(egui::PointerButton::Primary);
        let primary_clicked = resp.clicked_by(egui::PointerButton::Primary);
        let secondary_pressed = resp.drag_started_by(egui::PointerButton::Secondary);
        let primary_released = resp.drag_stopped_by(egui::PointerButton::Primary);

        let mouse_pos = pointer
            .latest_pos()
            .map(|p| Vec2::new(p.x - rect.min.x, p.y - rect.min.y))
            .unwrap_or(Vec2::new(0., h));

        let mx = mouse_pos.x;
        let my = h - mouse_pos.y;

        if resp.hovered() {
            let scroll = ctx.input(|i| {
                if shift {
                    i.raw_scroll_delta.x
                } else {
                    i.raw_scroll_delta.y
                }
            });
            if scroll != 0. {
                let factor = if scroll > 0. {
                    if shift { 0.44 } else { 0.88 }
                } else if shift {
                    2.24
                } else {
                    1.12
                };
                self.cam().dist = (self.cam().dist * factor).clamp(0.05, 5000.);
            }
        }

        if primary_pressed {
            self.on_3d_press((mx, my), (w, h), ctrl, shift, gl);
        }
        if primary_clicked {
            self.on_3d_click((mx, my), (w, h), ctrl, shift, gl);
        }
        if primary_released {
            self.on_3d_release(mx, my, gl);
        }
        if secondary_pressed {
            self.drag.state = DragState::Pan;
            self.drag.drag_last = mouse_pos;
        }

        let drag_delta = resp.drag_delta();
        if drag_delta != egui::Vec2::ZERO {
            let ldx = drag_delta.x;
            let ldy = drag_delta.y;
            match self.drag.state {
                DragState::None => {}
                DragState::Orbit => {
                    let cam = self.cam();
                    cam.yaw -= ldx * 0.008;
                    cam.pitch = (cam.pitch + ldy * 0.008).clamp(-PI / 2. + 0.001, PI / 2. - 0.001);
                }
                DragState::Pan => {
                    let cam = self.cam();
                    let sc = cam.dist * 0.0012;
                    let r = cam.left() * ldx * sc;
                    let u = cam.up_vec() * ldy * sc;
                    cam.target -= r - u;
                }
                DragState::Vertex => {
                    // Project mouse delta onto a plane at drag_ref facing the camera.
                    // We unproject both last and current mouse positions to rays, then
                    // intersect each with the drag plane to get world-space positions,
                    // and move all selected verts by the delta between them.
                    let cam = *self.cam();
                    let drag_ref = self.drag.drag_ref;
                    let plane_normal = cam.forward();

                    let last = self.drag.drag_last;

                    let ray_to_plane = |ray_pos: Vec3, ray_dir: Vec3| -> Option<Vec3> {
                        let denom = plane_normal.dot(ray_dir);
                        if denom.abs() < 1e-6 {
                            return None;
                        }
                        let t = plane_normal.dot(drag_ref - ray_pos) / denom;
                        Some(ray_pos + ray_dir * t)
                    };

                    let (rp0, rd0) = Self::unproject(last, Vec2::new(w, h), &cam.vp(w, h));
                    let (rp1, rd1) =
                        Self::unproject(Vec2::new(mx, my), Vec2::new(w, h), &cam.vp(w, h));

                    if let (Some(p0), Some(p1)) = (ray_to_plane(rp0, rd0), ray_to_plane(rp1, rd1)) {
                        let delta = p1 - p0;

                        let rd = RefDuper;
                        let self2 = unsafe { rd.detach_mut_ref(self) };
                        if let Selection::Vertices(ref verts) = self.selection
                            && let Some(part) = self2.get_current_part_mut()
                        {
                            for (id, pos) in part.vertices.get_mut_vec() {
                                if verts.contains(&id) {
                                    *pos += delta;
                                }
                            }
                        }
                    }

                    if let Some(sel) = self.editor.mesh
                        && let Some(mesh) = self.view.meshes.get_mut(sel)
                    {
                        mesh.rebuild(gl);
                        self.upload_selection_points(gl);
                    }

                    self.drag.drag_last = Vec2::new(mx, my);
                }
                DragState::Instance => {}
                DragState::InstanceRotation => {}
                DragState::Marquee(v4) => {
                    self.drag.state = DragState::Marquee(Vec4::new(v4.x, v4.y, mx, my))
                }
            }
        }

        if let DragState::Marquee(v4) = self.drag.state {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("marquee"),
            ));
            let sx0 = rect.min.x + v4.x;
            let sy0 = rect.min.y + (h - v4.y);
            let sx1 = rect.min.x + v4.z;
            let sy1 = rect.min.y + (h - v4.w);
            painter.rect_stroke(
                egui::Rect::from_two_pos(egui::pos2(sx0, sy0), egui::pos2(sx1, sy1)),
                0.0,
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_premultiplied(120, 180, 255, 200),
                ),
                egui::StrokeKind::Middle,
            );
        }
    }

    pub fn undo(&mut self, gl: &Context) {
        let rd = RefDuper;
        let self2 = unsafe { rd.detach_mut_ref(self) };
        self.history
            .cycle_history(HistoryCycleDir::Past, self2, gl);
    }

    pub fn redo(&mut self, gl: &Context) {
        let rd = RefDuper;
        let self2 = unsafe { rd.detach_mut_ref(self) };
        self.history
            .cycle_history(HistoryCycleDir::Future, self2, gl);
    }

    pub fn frame_to_geometry(&mut self) {}

    fn finish_marquee(&mut self, rect: Vec4, gl: &Context) {
        let sx0 = rect.x.min(rect.z);
        let sx1 = rect.x.max(rect.z);
        let sy0 = rect.y.min(rect.w);
        let sy1 = rect.y.max(rect.w);
        if sx1 - sx0 < 4.0 && sy1 - sy0 < 4.0 {
            return;
        }
        let ww = self.state.vp_rect.width();
        let wh = self.state.vp_rect.height();
        let vp = self.cam().vp(ww, wh);
        let in_box = |p: Vec3| -> bool {
            Self::project_to_screen(p, &vp, ww, wh)
                .map(|sp| sx0 <= sp.x && sp.x <= sx1 && sy0 <= sp.y && sp.y <= sy1)
                .unwrap_or(false)
        };
        match self.mode {
            EditorMode::Edit => {
                if let Some((_, _, part)) = self.get_current_part() {
                    let pos = part.vertices.get_vec(part, true);

                    if !matches!(self.selection, Selection::Vertices(_)) {
                        self.selection = Selection::Vertices(Vec::new())
                    }
                    let Selection::Vertices(ref mut selection) = self.selection else {
                        unreachable!()
                    };

                    for (i, p) in pos.iter() {
                        if in_box(*p) && !selection.contains(i) {
                            selection.push(i.clone());
                        }
                    }
                    self.upload_selection_points(gl);
                }
            }
            EditorMode::Assembly => {}
            _ => {}
        }
    }

    pub fn get_current_view_mesh(&self) -> Option<&ViewMesh> {
        self.view.meshes.get(self.editor.mesh?)
    }

    pub fn get_current_view_mesh_mut(&mut self) -> Option<&mut ViewMesh> {
        self.view.meshes.get_mut(self.editor.mesh?)
    }

    pub fn get_current_mesh_idx(&self) -> Option<usize> {
        self.editor.mesh
    }

    pub fn get_current_part_name(&self) -> Option<&str> {
        let sel = self.editor.mesh?;
        let mesh = self.view.meshes.get(sel)?;
        mesh.data
            .part_names
            .get(self.editor.part?)
            .map(|x| x.as_str())
    }

    pub fn get_current_part(&self) -> Option<(usize, &str, &Part)> {
        let idx = self.get_current_mesh_idx()?;
        let name = self.get_current_part_name()?;

        Some((idx, name, self.view.meshes.get(idx)?.data.parts.get(name)?))
    }

    pub fn get_current_part_mut(&mut self) -> Option<&mut Part> {
        let idx = self.get_current_mesh_idx()?;
        let name = self.get_current_part_name()?;

        // # SAFETY:
        // this borrow only exists to the end of this function
        // so no mutation can happen while it exists
        let name = unsafe { &*(name as *const _) };

        self.view.meshes.get_mut(idx)?.data.parts.get_mut(name)
    }

    pub fn add_history(&mut self, entry: HistoryEntry) {
        self.history.add_history(entry);
    }

    /// Renames the specified data and updates history
    pub fn rename(&mut self, rename: Rename) -> anyhow::Result<()> {
        match &rename {
            Rename::DataTag { view_idx, swap } => {
                if let Some(vm) = self.view.meshes.get_mut(*view_idx) {
                    vm.data.rename_data(swap);
                }
            },
            Rename::Part { view_idx, swap } => {
                if let Some(vm) = self.view.meshes.get_mut(*view_idx) {
                    vm.data.rename_part(swap);
                }
            },
            Rename::Vertex { part, swap } => {
                if let Some(vm) = self.view.meshes.get_mut(part.view_idx) {
                    vm.data.rename_vertex(part.name.as_str(), swap)?
                }
            },
            Rename::Uv { part, swap } => {
                if let Some(vm) = self.view.meshes.get_mut(part.view_idx) {
                    vm.data.rename_uv(part.name.as_str(), swap)?
                }
            },
            Rename::Normal { part, swap } => {
                if let Some(vm) = self.view.meshes.get_mut(part.view_idx) {
                    vm.data.rename_normal(part.name.as_str(), swap)?
                }
            },
            Rename::ComputeVertex { part, swap } => {},
            Rename::ComputeNormal { part, swap } => {},
        }
        self.add_history(HistoryEntry::Rename(rename));
        Ok(())
    }

    fn mutate_index_inner(&mut self, idx_mut: IndexMutationRequest) -> IndexMutation {
        todo!()
    }

    /// modifies indexed data and pushes to history
    pub fn mutate_index(&mut self, idx_mut: IndexMutationRequest) {
        let res = self.mutate_index_inner(idx_mut);
        self.add_history(HistoryEntry::IndexMutation(res));
    }

    fn check_vertex_collision(
        &mut self,
        mx: f32,
        my: f32,
        w: f32,
        h: f32,
        vp: &Mat4,
        include_compute: bool,
    ) -> Option<VertexId> {
        let r = self.cam().pick_radius(8., h);
        let pick_cycle = &mut self.click_cycle.vertices;
        let same_spot =
            (mx - pick_cycle.last_pos.x).abs() <= 2. && (my - pick_cycle.last_pos.y).abs() <= 2.;

        let rd = RefDuper;

        let (_, _, part) = self.get_current_part()?;

        let verts = part.vertices.get_vec(part, include_compute);

        let hits: Vec<&VertexId> = self
            .raycast_vertices(&verts, Vec2::new(mx, my), Vec2::new(w, h), vp, r)
            .iter()
            .map(|r| unsafe { rd.detach_ref(*r) })
            .collect();

        let pick_cycle = &mut self.click_cycle.vertices;
        if same_spot && !pick_cycle.candidates.is_empty() {
            pick_cycle.current = (pick_cycle.current + 1) % pick_cycle.candidates.len();
            pick_cycle.candidates.get(pick_cycle.current).cloned()
        } else if !hits.is_empty() {
            if !same_spot {
                pick_cycle.last_pos = Vec2::new(mx, my);
                pick_cycle.candidates = hits.iter().map(|r| (*r).clone()).collect();
                pick_cycle.current = 0;
            }
            hits.first().map(|r| (*r).clone())
        } else {
            pick_cycle.last_pos = Vec2::new(-9999., -9999.);
            pick_cycle.candidates.clear();
            None
        }
    }

    fn on_3d_press(
        &mut self,
        mouse: (f32, f32),
        size: (f32, f32),
        ctrl: bool,
        shift: bool,
        gl: &Context,
    ) {
        if self.block_input() {
            return;
        }

        let (mx, my) = mouse;
        let (w, h) = size;

        let vp = self.cam().vp(w, h);

        self.drag.state = DragState::Orbit;
        match self.mode {
            EditorMode::View => {}
            EditorMode::Assembly => {}
            EditorMode::Edit => {
                let hit = self.check_vertex_collision(mx, my, w, h, &vp, false);
                let rd = RefDuper;
                let self2 = unsafe { rd.detach_ref(self) };
                if let (Some(hit_id), Selection::Vertices(verts)) = (hit, &mut self.selection) {
                    self.drag.drag_ref = self2
                        .get_current_part()
                        .unwrap()
                        .2
                        .resolve_vertex(&hit_id)
                        .unwrap();
                    if shift {
                        if ctrl && verts.contains(&hit_id) {
                            let idx = verts.iter().position(|id| *id == hit_id).unwrap();
                            verts.remove(idx);
                        } else if !verts.contains(&hit_id) {
                            verts.push(hit_id);
                        }
                    } else if !verts.contains(&hit_id) {
                        verts.clear();
                        verts.push(hit_id);
                    }
                    self.upload_selection_points(gl);
                    self.drag.state = DragState::Vertex;
                    self.drag.drag_last = Vec2::new(mx, my);

                    if let Some((idx, name, part)) = self.get_current_part() {
                        self.add_history(HistoryEntry::MeshPart(LightMeshPartSnapshot {
                            idx,
                            name: name.to_string(),
                            part: Box::new(part.clone())
                        }));
                    }

                } else if shift {
                    self.drag.state = DragState::Marquee(Vec4::new(mx, my, mx, my));
                }
            }
        }
    }

    fn on_3d_click(
        &mut self,
        mouse: (f32, f32),
        size: (f32, f32),
        ctrl: bool,
        shift: bool,
        gl: &Context,
    ) {
        if self.block_input() {
            return;
        }

        let (mx, my) = mouse;
        let (w, h) = size;

        let vp = self.cam().vp(w, h);
        match self.mode {
            EditorMode::View => {}
            EditorMode::Assembly => {}
            EditorMode::Edit => {
                let hit = self.check_vertex_collision(mx, my, w, h, &vp, true);

                if hit.is_some() && matches!(self.selection, Selection::None) {
                    self.selection = Selection::Vertices(Vec::new());
                }

                if let (Some(hit_id), Selection::Vertices(verts)) = (hit, &mut self.selection) {
                    if shift {
                        if ctrl && verts.contains(&hit_id) {
                            let idx = verts.iter().position(|id| *id == hit_id).unwrap();
                            verts.remove(idx);
                        } else if !verts.contains(&hit_id) {
                            verts.push(hit_id);
                        }
                    } else if !verts.contains(&hit_id) {
                        verts.clear();
                        verts.push(hit_id);
                    }
                    self.upload_selection_points(gl);
                }
            }
        }
    }

    fn on_3d_release(&mut self, mx: f32, my: f32, gl: &Context) {
        if let DragState::Marquee(vec4) = self.drag.state {
            self.finish_marquee(vec4, gl);
        }

        self.drag.state = DragState::None;
    }

    fn unproject(point: Vec2, screen_size: Vec2, vp: &Mat4) -> (Vec3, Vec3) {
        let inv = vp.inverse();
        let nx = (point.x / screen_size.x) * 2.0 - 1.0;
        let ny = (point.y / screen_size.y) * 2.0 - 1.0;
        let n4 = inv * Vec4::new(nx, ny, -1.0, 1.0);
        let f4 = inv * Vec4::new(nx, ny, 1.0, 1.0);
        let n3 = Vec3::new(n4.x, n4.y, n4.z) / n4.w;
        let f3 = Vec3::new(f4.x, f4.y, f4.z) / f4.w;
        (n3, (f3 - n3).normalize())
    }

    fn check_point_dist(ray_pos: Vec3, ray_dir: Vec3, point: Vec3, r: f32) -> Option<f32> {
        let delta = ray_pos - point;
        let b = 2. * ray_dir.dot(delta);
        let disc = b * b - 4. * (delta.dot(delta) - r * r);
        if disc < 0. {
            return None;
        }
        let t = (-b - disc.sqrt()) / 2.;
        if t > 0. { Some(t) } else { None }
    }

    fn raycast_vertices<'a, K>(
        &self,
        vertices: &'a [(K, Vec3)],
        mouse: Vec2,
        size: Vec2,
        vp: &Mat4,
        r: f32,
    ) -> Vec<&'a K> {
        let (ray_pos, ray_dir) = Self::unproject(mouse, size, vp);

        let mut hits: Vec<(&K, f32)> = vertices
            .iter()
            .filter_map(|(id, vert)| {
                Self::check_point_dist(ray_pos, ray_dir, *vert, r).map(|dist| (id, dist))
            })
            .collect();

        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        hits.into_iter().map(|(id, _)| id).collect()
    }

    fn project_to_screen(p: Vec3, mvp: &Mat4, ww: f32, wh: f32) -> Option<egui::Pos2> {
        let v = *mvp * glam::Vec4::new(p.x, p.y, p.z, 1.0);
        if v.w <= 0.0 {
            return None;
        }
        let sx = (v.x / v.w + 1.0) / 2.0 * ww;
        let sy = (v.y / v.w + 1.0) / 2.0 * wh;
        Some(egui::pos2(sx, sy))
    }

    fn upload_selection_points(&mut self, gl: &Context) {
        if let Selection::Vertices(ref verts) = self.selection {
            if verts.is_empty() {
                self.selection = Selection::None;
                if let Some(buf) = self.render.sel_points.take() {
                    buf.destroy(gl);
                }
                return;
            }
            if let Some((_, _, part)) = self.get_current_part() {
                let mut selected = Vec::new();
                for id in verts.iter() {
                    if let Ok(vert) = part.resolve_vertex(id) {
                        selected.push(vert);
                    }
                }
                if selected.is_empty() {
                    if let Some(buf) = self.render.sel_points.take() {
                        buf.destroy(gl);
                    };
                    return;
                }
                let nrm = vec![Vec3::Y; selected.len()];
                let ch = vec![0i32; 3 * selected.len()];
                self.render.sel_points = Some(GpuMesh::new(gl, &selected, &nrm, &ch));
            }
        } else if matches!(self.selection, Selection::None)
            && let Some(buf) = self.render.sel_points.take()
        {
            buf.destroy(gl);
        }
    }

    pub fn handle_file_open(&mut self, gl: &Context) {
        if let Some(recv) = self.state.ui.open_session_channel.as_ref() {
            match recv.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.state.ui.open_session_channel = None;
                }
                Ok(session) => {
                    if let Err(e) = self.load_session(&session, gl) {
                        eprintln!("Error loading session: {e}")
                    }
                }
            }
        }
        if let Some(recv) = self.state.ui.open_mesh_channel.as_ref() {
            match recv.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.state.ui.open_mesh_channel = None;
                }
                Ok(meshes) => {
                    if let Err(e) = self.load_meshes(meshes, gl) {
                        eprintln!("Error loading meshes: {e}");
                    }
                }
            }
        }
    }

    pub fn load_session(&mut self, path: &Path, gl: &Context) -> anyhow::Result<()> {
        let raw = fs::read_to_string(path)?;
        let session: SessionData = serde_json::from_str(&raw)?;

        self.view.camera = session.camera.into();
        self.editor.camera = self.view.camera;

        let mut vms = Vec::new();

        for SessionMeshData { path, placements } in session.meshes {
            let mut vm = LightMesh::load(&path)?.into_view_mesh(path, gl);

            for SessionPlacementData {
                position,
                rotation,
                count,
                offset_pos,
                offset_rot,
            } in placements
            {
                vm.placements.push(ViewPlacement {
                    position,
                    rotation,
                    count,
                    offset_pos,
                    offset_rot,
                    visible: true,
                })
            }
            vms.push(vm);
        }

        mem::swap(&mut self.view.meshes, &mut vms);

        for vm in vms {
            vm.destroy(gl);
        }

        Ok(())
    }
}
