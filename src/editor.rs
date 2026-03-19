use std::collections::HashMap;
use std::f32::consts::PI;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::glow::Context;
use egui::{Key, Response};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

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

impl Camera {
    pub fn eye(&self) -> Vec3 {
        let cp = self.pitch.cos(); let sp = self.pitch.sin();
        let cy = self.yaw.cos();   let sy = self.yaw.sin();
        self.target + self.dist * Vec3::new(cp * sy, sp, cp * cy)
    }
    pub fn view_mat(&self) -> Mat4 { Mat4::look_at_lh(self.eye(), self.target, Vec3::Y) }
    pub fn proj_mat(&self, w: f32, h: f32) -> Mat4 {
        Mat4::perspective_lh(self.fov, (w / h).max(0.001), 0.1, 5000.0)
    }
    pub fn mvp(&self, w: f32, h: f32) -> Mat4 { self.proj_mat(w, h) * self.view_mat() }
    pub fn right(&self) -> Vec3 { let v = self.view_mat(); Vec3::new(v.col(0).x, v.col(0).y, v.col(0).z) }
    pub fn up_vec(&self) -> Vec3 { let v = self.view_mat(); Vec3::new(v.col(1).x, v.col(1).y, v.col(1).z) }
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
    pub hovered: isize,
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
    pub camera: Camera
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
        let md = if let Some(p) = path {
            WorkingLightMesh::load(p).unwrap_or_default()
        } else {
            WorkingLightMesh::default()
        };

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
                mesh: md,
                camera: Camera::default(),
                part: None,
                part_names: Vec::new(),
            },
            selection: Selection {
                verts: Vec::new(),
                instances: Vec::new(),
                hovered: -1,
            },
            drag: Drag {
                state: DragState::None,
                drag_last: Vec2::ZERO,
                drag_plane: (Vec3::ZERO, Vec3::ZERO),
                pre_drag_snapshot: None,
                rot_axis: Vec3::ZERO,
                pending_desel: 0,
            },
            click_cycle: ClickCycle {
                verticex: InnerCycle { last_pos: Vec2::ZERO, candidates: Vec::new(), current: 0 },
                instances: InnerCycle { last_pos: Vec2::ZERO, candidates: Vec::new(), current: 0 },
            },
            view: View {
                meshes: Vec::new(),
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
                status: "Beatcraft LightMesh Editor".to_string(),
                status_timer: 0.,
                clipboard: Clipboard::None,
                dirty: false,
                gl,
            },
            assembly: Assembly {
                handles: Vec::new(),
            },
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
                _ => {

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
            self.on_3d_press(mx, my, w, h, shift, gl);
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
            let ldy = -drag_delta.y;
            match self.drag.state {
                DragState::None => {},
                DragState::Orbit => {
                    let cam = self.cam();
                    cam.yaw += ldx * 0.008;
                    cam.pitch = (cam.pitch - ldy * 0.008).clamp(-PI/2.+0.001, PI/2.-0.001);
                },
                DragState::Pan => {},
                DragState::Vertex => {},
                DragState::Instance => {},
                DragState::InstanceRotation => {},
                DragState::Marquee(v4) => {
                    self.drag.state = DragState::Marquee(Vec4::new(v4.x, v4.y, mx, my))
                },
            }
        }

    }


    pub fn undo(&mut self) {

    }

    pub fn redo(&mut self) {

    }

    pub fn frame_to_geometry(&mut self) {

    }


    fn on_3d_press(&mut self, mx: f32, my: f32, w: f32, h: f32, shift: bool, gl: &Context) {
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

            },
        }

    }

    fn on_3d_release(&mut self, mx: f32, my: f32, gl: &Context) {

    }

}


