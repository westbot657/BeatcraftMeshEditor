use std::collections::HashMap;
use std::f32::consts::PI;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::glow::Context;
use egui::{Key, Response};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

use crate::data::VertexId;
use crate::Lifeline;
use crate::light_mesh::{LightMesh, LightMeshMetaSnapshot, LightMeshPartSnapshot, LightMeshPlacementSnapshot, Part};
use crate::render::{GpuMesh, InstanceData, MeshDrawCall, Renderer};


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
    pub fn mvp(&self, w: f32, h: f32) -> Mat4 { self.proj_mat(w, h) * self.view_mat() }
    pub fn left(&self) -> Vec3 {
        let m = self.view_mat();
        Vec3::new(m.col(0).x, m.col(1).x, m.col(2).x)
    }
    pub fn up_vec(&self) -> Vec3 {
        let m = self.view_mat();
        Vec3::new(m.col(0).y, m.col(1).y, m.col(2).y)
    }
    pub fn forward(&self) -> Vec3 { (self.target - self.eye()).normalize() }
    pub fn pick_radius(&self, screen_px: f32, h: f32) -> f32 {
        let view_h = 2.0 * self.dist * (self.fov.to_radians() / 2.0).tan();
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

pub struct ViewPlacement {
    pub position: Vec3,
    pub rotation: Quat,
    pub count: u32,
    pub offset_pos: Vec3,
    pub offset_rot: Quat,
    pub visible: bool,
}

pub struct ViewPlacementsSnapshot {
    pub idx: usize,
    pub placements: Vec<ViewPlacement>,
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
            placements: Vec::new()
        }
    }

    pub fn rebuild(&mut self, gl: &Context) {
        let v = mem::take(&mut self.gpu_bufs.0);
        self.gpu_bufs.0 = GpuMesh::set_from_hashmap(gl, &self.data, v);
        let full = self.gpu_bufs.1.get_or_insert_with(|| GpuMesh::new(gl, &[], &[], &[]));
        full.set_from_full_light_mesh(gl, &self.data);
    }

    pub fn render(&self, calls: &mut Vec<InstanceData>) -> Option<&GpuMesh> {
        calls.push(InstanceData::new(Mat4::IDENTITY, 1., None));
        self.gpu_bufs.1.as_ref()

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
    pub mesh: Option<PathBuf>,
    pub camera: Camera,
    pub part: Option<String>,
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
    pub drag_plane: (Vec3, Vec3),
    pub pre_drag_snapshot: Option<LightMesh>,
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
    pub active: usize,
    pub session: Option<PathBuf>,
    pub camera: Camera
}

pub struct Assembly {
    pub handles: Vec<(Vec3, InstanceHandleType, usize)>,
    pub hovered: Option<usize>,
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
}

pub enum HistoryEntry {
    MeshPart(LightMeshPartSnapshot),
    MeshMeta(LightMeshMetaSnapshot),
    MeshPlacement(LightMeshPlacementSnapshot),
    ViewPlacement(ViewPlacementsSnapshot),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryType {
    MeshPart(String),
    MeshMeta,
    MeshPlacement,
    ViewPlacement,
}

pub struct History {
    pub history: Vec<HistoryEntry>,
    pub future: Vec<HistoryEntry>,
    pub limit: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HistoryCycleDir {
    Past,
    Future,
}

impl History {
    pub fn add_history(&mut self, entry: HistoryEntry) {
        self.history.push(entry);
        self.future.clear();
    }

    pub fn cycle_history(
        &mut self,
        dir: HistoryCycleDir,
        view_meshes: &mut [ViewMesh],
    ) {
        let (back, front) = match dir {
            HistoryCycleDir::Past => (&mut self.history, &mut self.future),
            HistoryCycleDir::Future => (&mut self.future, &mut self.history),
        };

        if let Some(restore) = front.pop() {
            let save = match restore {
                HistoryEntry::MeshPart(LightMeshPartSnapshot { idx, name, part }) => {
                    let m = view_meshes.get_mut(idx).unwrap();
                    let current = m.data.parts.insert(name.clone(), *part).unwrap();
                    HistoryEntry::MeshPart(LightMeshPartSnapshot {
                        idx,
                        name,
                        part: Box::new(current)
                    })
                },
                HistoryEntry::MeshMeta(LightMeshMetaSnapshot {
                    idx, mut credits, mut textures, mut data, mut cull
                }) => {
                    let m = view_meshes.get_mut(idx).unwrap();
                    mem::swap(&mut credits, &mut m.data.credits);
                    mem::swap(&mut textures, &mut m.data.textures);
                    mem::swap(&mut data, &mut m.data.data);
                    mem::swap(&mut cull, &mut m.data.cull);

                    HistoryEntry::MeshMeta(LightMeshMetaSnapshot {
                        idx, credits, textures, data, cull
                    })
                },
                HistoryEntry::MeshPlacement(LightMeshPlacementSnapshot {
                    view_idx, mut placements
                }) => {
                    let m = view_meshes.get_mut(view_idx).unwrap();
                    mem::swap(&mut placements, &mut m.data.placements);

                    HistoryEntry::MeshPlacement(LightMeshPlacementSnapshot {
                        view_idx, placements
                    })
                },
                HistoryEntry::ViewPlacement(ViewPlacementsSnapshot {
                    idx, mut placements
                }) => {
                    let m = view_meshes.get_mut(idx).unwrap();
                    mem::swap(&mut placements, &mut m.placements);

                    HistoryEntry::ViewPlacement(ViewPlacementsSnapshot {
                        idx, placements
                    })
                },
            };
            back.push(save);
        }

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
            Arc::new(egui::FontData::from_static(include_bytes!("./assets/fonts/SourceCodePro-Regular.ttf")))
        );
        fonts.families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, String::from("source-code-pro"));

        fonts.families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, String::from("source-code-pro"));

        cc.egui_ctx.set_fonts(fonts);

        let gl = Arc::clone(cc.gl.as_ref().expect("GL context not found"));

        let gl2 = Arc::clone(&gl);

        let mut meshes = Vec::new();
        if let Some(p) = path {
            meshes.push(LightMesh::load(&p).unwrap().into_view_mesh(p, &gl));
        }

        Self {
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
                drag_plane: (Vec3::ZERO, Vec3::ZERO),
                pre_drag_snapshot: None,
                rot_axis: Vec3::ZERO,
            },
            click_cycle: ClickCycle {
                vertices: InnerCycle { last_pos: Vec2::ZERO, candidates: Vec::new(), current: 0 },
                instances: InnerCycle { last_pos: Vec2::ZERO, candidates: Vec::new(), current: 0 },
            },
            view: View {
                meshes,
                active: 0,
                session: None,
                camera: Camera::default(),
            },
            state: State {
                vp_rect: egui::Rect { min: egui::Pos2 { x: 0., y: 0. }, max: egui::Pos2 { x: 0., y: 0. } },
                wireframe: true,
                show_grid: true,
                show_verts: true,
                euler_swizzle: EulerSwizzle::YXZ,
                status: "".to_string(),
                status_timer: 0.,
                clipboard: Clipboard::None,
                dirty: false,
                gl,
            },
            assembly: Assembly {
                handles: Vec::new(),
                hovered: None,
            },
            history: History {
                history: Vec::new(),
                future: Vec::new(),
                limit: 200
            }
        }
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

    pub fn handle_keys(&mut self, ctx: &egui::Context, gl: &Context) {
        let input = ctx.input(|i| i.clone());
        let ctrl = input.modifiers.ctrl;
        let shift = input.modifiers.shift;

        if ctrl && input.key_pressed(Key::Z) {
            if shift { self.redo(); } else { self.undo(); }
        }
        if ctrl && input.key_pressed(Key::S) {
            match self.mode {
                EditorMode::View => {

                }
                EditorMode::Assembly | EditorMode::Edit => {

                }
            }
        }
        if ctrl && input.key_pressed(Key::C) {
            // TODO
        }
        if ctrl && input.key_pressed(Key::V) {
            // TODO
        }

        if self.mode != EditorMode::View {
            if input.key_pressed(Key::W) {
                self.state.wireframe = !self.state.wireframe;
            }
            if input.key_pressed(Key::G) {
                self.state.show_grid = !self.state.show_grid;
            }
            if input.key_pressed(Key::V) {
                self.state.show_verts = !self.state.show_verts;
            }
        }

    }

    pub fn handle_3d_input(&mut self, resp: &Response, ctx: &egui::Context, gl: &Context) {
        let rect = self.state.vp_rect;
        let w = rect.width();
        let h = rect.height();

        let pointer = ctx.input(|i| i.pointer.clone());
        let shift = ctx.input(|i| i.modifiers.shift);
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let primary_pressed = resp.drag_started_by(egui::PointerButton::Primary);
        let secondary_pressed = resp.drag_started_by(egui::PointerButton::Secondary);
        let primary_released = resp.drag_stopped_by(egui::PointerButton::Primary);

        let mouse_pos = pointer.latest_pos()
            .map(|p| Vec2::new(p.x - rect.min.x, p.y - rect.min.y))
            .unwrap_or(Vec2::new(0., h));

        let mx = mouse_pos.x;
        let my = h - mouse_pos.y;

        if resp.hovered() {
            let scroll = ctx.input(|i| if shift { i.raw_scroll_delta.x } else { i.raw_scroll_delta.y } );
            if scroll != 0. {
                let factor = if scroll > 0. { if shift { 0.44 } else { 0.88 } } else if shift { 2.24 } else { 1.12 };
                self.cam().dist = (self.cam().dist * factor).clamp(0.05, 5000.);
            }
        }

        if primary_pressed {
            self.on_3d_press((mx, my), (w, h), ctrl, shift, gl);
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
                DragState::None => {},
                DragState::Orbit => {
                    let cam = self.cam();
                    cam.yaw -= ldx * 0.008;
                    cam.pitch = (cam.pitch + ldy * 0.008).clamp(-PI/2.+0.001, PI/2.-0.001);
                },
                DragState::Pan => {
                    let cam = self.cam();
                    let sc = cam.dist * 0.0012;
                    let r = cam.left() * ldx * sc;
                    let u = cam.up_vec() * ldy * sc;
                    cam.target -= r - u;
                },
                DragState::Vertex => {},
                DragState::Instance => {},
                DragState::InstanceRotation => {},
                DragState::Marquee(v4) => {
                    self.drag.state = DragState::Marquee(Vec4::new(v4.x, v4.y, mx, my))
                },
            }
        }

        if let DragState::Marquee(v4) = self.drag.state {
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("marquee")));
            let sx0 = rect.min.x + v4.x;
            let sy0 = rect.min.y + (h - v4.y);
            let sx1 = rect.min.x + v4.z;
            let sy1 = rect.min.y + (h - v4.w);
            painter.rect_stroke(
                egui::Rect::from_two_pos(egui::pos2(sx0, sy0), egui::pos2(sx1, sy1)),
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(120, 180, 255, 200)),
                egui::StrokeKind::Middle
            );
        }

    }

    pub fn undo(&mut self) {
        self.history.cycle_history(
            HistoryCycleDir::Past,
            &mut self.view.meshes,
        );
    }

    pub fn redo(&mut self) {
        self.history.cycle_history(
            HistoryCycleDir::Future,
            &mut self.view.meshes,
        );
    }

    pub fn frame_to_geometry(&mut self) {

    }

    fn finish_marquee(&mut self, rect: Vec4, gl: &Context) {

    }

    pub fn get_current_mesh_idx(&self) -> Option<usize> {
        let path = self.editor.mesh.as_ref()?;

        for (idx, mesh) in self.view.meshes.iter().enumerate() {
            if mesh.path == *path {
                return Some(idx)
            }
        }
        None
    }

    pub fn get_current_part_name(&self) -> Option<&str> {
        self.editor.part.as_deref()
    }

    pub fn get_current_part(&self) -> Option<(usize, &str, &Part)> {
        let idx = self.get_current_mesh_idx()?;
        let name = self.get_current_part_name()?;

        Some((idx, name, self.view.meshes.get(idx)?.data.parts.get(name)?))
    }

    pub fn push_history(&mut self, typ: HistoryType) {
        match typ {
            HistoryType::MeshPart(name) => {
                if let Some((idx, name, part)) = self.get_current_part() {
                    self.history.add_history(HistoryEntry::MeshPart(
                        LightMeshPartSnapshot {
                            idx,
                            name: name.to_string(),
                            part: Box::new(part.clone()),
                        }
                    ));
                }
            },
            HistoryType::MeshMeta => {},
            HistoryType::MeshPlacement => {},
            HistoryType::ViewPlacement => {},
        }
    }


    fn on_3d_press(&mut self, mouse: (f32, f32), size: (f32, f32), ctrl: bool, shift: bool, gl: &Context) {
        let (mx, my) = mouse;
        let (w, h) = size;

        let mvp = self.cam().mvp(w, h);

        match self.mode {
            EditorMode::View => {
                self.drag.state = DragState::Orbit;
            },
            EditorMode::Assembly => {
                self.drag.state = DragState::Orbit;
            },
            EditorMode::Edit => {
                self.drag.state = DragState::Orbit;
                let r = self.cam().pick_radius(8., h);
                let pick_cycle = &mut self.click_cycle.vertices;
                let same_spot = (mx - pick_cycle.last_pos.x).abs() <= 2.
                    && (my - pick_cycle.last_pos.y).abs() <= 2.;

                let ll = Lifeline;

                let hits: Vec<&VertexId> = self.raycast_vertices(mx, my, w, h, &mvp, r)
                    .iter()
                    .map(|r| unsafe { ll.detach_ref(*r) })
                    .collect();

                let pick_cycle = &mut self.click_cycle.vertices;
                let hit = if same_spot && !pick_cycle.candidates.is_empty() {
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
                };

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
                } else if shift {
                    self.drag.state = DragState::Marquee(Vec4::new(mx, my, mx, my));
                } else {
                    self.selection = Selection::None;
                    self.upload_selection_points(gl);
                }

            },
        }

    }

    fn on_3d_release(&mut self, mx: f32, my: f32, gl: &Context) {

        if let DragState::Marquee(vec4) = self.drag.state {
            self.finish_marquee(vec4, gl);
        }

        if matches!(self.drag.state, DragState::Vertex | DragState::Instance | DragState::InstanceRotation)
            && let Some(snap) = self.drag.pre_drag_snapshot.take() {
            // Push history and clear future.
        }

        self.drag.state = DragState::None;
    }

    fn raycast_vertices(&self, mx: f32, my: f32, w: f32, h: f32, mvp: &Mat4, r: f32) -> Vec<&VertexId> {



        Vec::new()
    }

    fn upload_selection_points(&mut self, gl: &Context) {

    }

}


