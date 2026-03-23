// Control Scheme
//   Left-click             select vertex (shift = multi-select)
//   Left-drag (empty)      orbit
//   Shift+Left-click       add/remove select
//   Shift+Left-drag        marquee select
//   Right-drag             pan
//   Scroll                 zoom
//   Left-drag (vertex)     move along viewport-parallel plane
//   W                      wireframe toggle
//   G                      grid toggle
//   V                      vertex dots toggle
//   Space (part-edit)      spawn vertex at cursor
//   E                      assembly <-> part-edit mode
//   [ / ]                  cycle active part
//   N                      create/remove triangle from selection
//   X                      flip winding of selected triangles
//   Ctrl+Z / Ctrl+Shift+Z  undo / redo
//   Ctrl+S                 save JSON
//   Ctrl+Shift+S           optimized save
//   Escape                 deselect all

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

use eframe::glow::{self, HasContext};
use egui::Frame;
use egui::ahash::HashMapExt;
use glam::{Quat, Vec3};

use self::editor::{App, EulerSwizzle, RotationDisplayMode, ViewPlacement};
use self::render::MeshDrawCall;
use self::widgets::MathDragValue;

pub mod data;
pub mod easing;
pub mod light_mesh;
pub mod math_interp;
pub mod render;
pub mod editor;
pub mod widgets;

#[derive(Copy, Clone)]
struct UnsafeMutRef<T: 'static> {
    t: *mut T,
}

impl<T: 'static> UnsafeMutRef<T> {
    pub unsafe fn new(t: &mut T) -> Self {
        let ptr = t as *mut T;
        Self {
            t: ptr
        }
    }
}

impl<T> Deref for UnsafeMutRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { & *self.t.cast::<T>() }
    }
}

impl<T> UnsafeMutRef<T> {
    pub fn ref_mut(&self) -> &'static mut T {
        unsafe { &mut *self.t.cast::<T>() }
    }
}

unsafe impl<T> Send for UnsafeMutRef<T> {}
unsafe impl<T> Sync for UnsafeMutRef<T> {}

pub(crate) struct Lifeline;

impl Lifeline {

    /// Detaches a reference from it's owner, allowing
    /// mutable references to exist simultaneously
    /// SAFETY: attaches the lifetime to self for
    /// the illusion of safety
    pub(crate) unsafe fn detach_ref<'a, T>(&'a self, t: &T) -> &'a T {
        unsafe { & *(t as *const T) }
    }

    /// Detaches a mutable reference from it's owner, allowing
    /// more references to be created
    /// SAFETY: attaches the lifetime to self for
    /// the illusion of safety
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn detach_mut_ref<'a, T>(&'a self, t: &mut T) -> &'a mut T {
        unsafe { &mut *(t as *mut T) }
    }

}


impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let gl = frame.gl().unwrap();

        if self.state.dirty { self.rebuild_meshes(gl); }

        self.handle_keys(ctx, gl);

        let dt = ctx.input(|i| i.unstable_dt);
        if self.state.status_timer > 0. { self.state.status_timer -= dt; }

        self.handle_file_open(gl);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // Open Session...
                    if ui.button("Open Session\u{2026}  \u{2502}").clicked() && !self.block_input() {
                        let (sx, rx) = mpsc::channel();
                        self.state.ui.open_session_channel = Some(rx);
                        std::thread::spawn(move || {
                            if let Some(session) = rfd::FileDialog::new()
                                .set_title("Open Session...")
                                .add_filter("json", &["json"])
                                .pick_file() {
                                let _ = sx.send(session);
                            }
                        });
                    }
                    // Open...
                    if ui.button("Open\u{2026}          \u{2502}").clicked() && !self.block_input() {
                        let (sx, rx) = mpsc::channel();
                        self.state.ui.open_mesh_channel = Some(rx);
                        std::thread::spawn(move || {
                            if let Some(meshes) = rfd::FileDialog::new()
                                .set_title("Open Meshes...")
                                .add_filter("json", &["json"])
                                .pick_files() {
                                let _ = sx.send(meshes);
                            }
                        });
                    }
                    if ui.button("Save           \u{2502} [Ctrl+S]").clicked() {
                        match self.mode {
                            editor::EditorMode::View => {

                            }
                            editor::EditorMode::Assembly | editor::EditorMode::Edit => {

                            }
                        }
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo  \u{2502}       [Ctrl+Z]").clicked() {
                        self.undo();
                        ui.close();
                    }
                    if ui.button("Redo  \u{2502} [Ctrl+Shift+Z]").clicked() {
                        self.redo();
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.state.wireframe,  "Wireframe     \u{2502} [W]");
                    ui.checkbox(&mut self.state.show_grid,  "Show Grid     \u{2502} [G]");
                    ui.checkbox(&mut self.state.show_verts, "Vertices      \u{2502} [C]");
                    if ui.button("Reframe Camera  \u{2502} [F]").clicked() {
                        self.frame_to_geometry();
                        ui.close();
                    }
                });
                if !self.state.status.is_empty() && self.state.status_timer > 0. {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&self.state.status).color(egui::Color32::from_rgb(140, 200, 140)));
                    });
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mode = match self.mode {
                    editor::EditorMode::Assembly => "Assembly ",
                    editor::EditorMode::Edit     => "Part Edit",
                    editor::EditorMode::View     => "View     ",
                };
                ui.label(format!("Mode: {mode} [E]  |  Reframe [F]  |  [W]ireframe / [G]rid / [V]ertices"));
            });
        });

        egui::SidePanel::left("left_panel")
            .exact_width(220.)
            .resizable(false)
            .show(ctx, |ui| {
                ui.allocate_ui(ui.available_size(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        match self.mode {
                            editor::EditorMode::View => {
                                let mut to_remove = None;
                                for (i, mesh) in self.view.meshes.iter_mut().enumerate() {
                                    let name = mesh.path.with_extension("");
                                    let name = name.file_name()
                                        .map(|x| x.to_string_lossy())
                                        .unwrap_or_else(|| std::borrow::Cow::Borrowed("?"));

                                    ui.add_space(2.0);
                                    let selected = self.state.ui.view_mesh == Some(i);
                                    let available = ui.available_size_before_wrap();
                                    if ui.add_sized(
                                        egui::Vec2::new(available.x, 20.0),
                                        egui::Button::selectable(selected, name.as_ref()),
                                    ).clicked() {
                                        self.state.ui.view_mesh = if selected { None } else { Some(i) };
                                    };
                                    ui.add_space(2.0);

                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut mesh.visible, "");
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("Close").clicked() {
                                                to_remove = Some(i);
                                            }
                                            if ui.button("Edit").clicked() {
                                                self.editor.mesh = Some(i);
                                                self.last_mode = self.mode;
                                                self.mode = editor::EditorMode::Assembly;
                                            }
                                        });
                                    });

                                    ui.separator();
                                }
                                if let Some(i) = to_remove {
                                    self.view.meshes.remove(i);
                                    if let Some(sel) = self.state.ui.view_mesh {
                                        if sel == i {
                                            self.state.ui.view_mesh = None;
                                        } else if sel > i {
                                            self.state.ui.view_mesh = Some(sel - 1);
                                        }
                                    }
                                    if let Some(sel) = self.editor.mesh {
                                        if sel == i {
                                            self.editor.mesh = None;
                                        } else if sel > i {
                                            self.editor.mesh = Some(sel - 1);
                                        }
                                    }
                                }
                            },
                            editor::EditorMode::Assembly => {

                            },
                            editor::EditorMode::Edit => {

                            },
                        }
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(5.);
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.set_width(ui.available_width());
                                let target = &mut self.cam().target;
                                let spacing = ui.spacing().item_spacing.x;
                                let width = (ui.available_width() - spacing * 2.0) / 3.0;
                                let mut vars = HashMap::new();
                                vars.insert("x".to_string(), target.x);
                                vars.insert("y".to_string(), target.y);
                                vars.insert("z".to_string(), target.z);
                                ui.add_sized([width, 20.], MathDragValue::new(&mut target.x, &mut vars).speed(0.1));
                                ui.add_sized([width, 20.], MathDragValue::new(&mut target.y, &mut vars).speed(0.1));
                                ui.add_sized([width, 20.], MathDragValue::new(&mut target.z, &mut vars).speed(0.1));
                            });
                            ui.label("Camera Pivot");
                        });
                    });
                });

            });

        egui::SidePanel::right("right_panel")
            .exact_width(300.)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.mode {
                        editor::EditorMode::View => {
                            if let Some(sel) = self.state.ui.view_mesh && let Some(mesh) = self.view.meshes.get_mut(sel) {
                                if ui.button("+ Add Placement").clicked() {
                                    mesh.placements.push(ViewPlacement::default());
                                    self.state.ui.collapsed.entry(sel).or_default().push(false);
                                    self.state.ui.view_rotation_modes.entry(sel).or_default()
                                        .push([RotationDisplayMode::Euler(EulerSwizzle::YXZ), RotationDisplayMode::Euler(EulerSwizzle::YXZ)]);
                                }

                                let mut to_remove = None;
                                for (i, placement) in mesh.placements.iter_mut().enumerate() {
                                    let collapsed = self.state.ui.collapsed.entry(sel).or_default();
                                    if collapsed.len() <= i { collapsed.push(false); }
                                    let is_collapsed = &mut collapsed[i];

                                    ui.horizontal(|ui| {
                                        let icon = if *is_collapsed { "▶" } else { "▼" };
                                        if ui.button(icon).clicked() {
                                            *is_collapsed = !*is_collapsed;
                                        }
                                        ui.label(format!("Placement {}", i + 1));
                                    });

                                    if !*is_collapsed {
                                        let w2 = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                                        let w3 = (ui.available_width() - ui.spacing().item_spacing.x * 2.0) / 3.0;

                                        let vec3_row = |ui: &mut egui::Ui, v: &mut Vec3| {
                                            let mut vars = HashMap::new();
                                            vars.insert("x".to_string(), v.x);
                                            vars.insert("y".to_string(), v.y);
                                            vars.insert("z".to_string(), v.z);
                                            ui.horizontal(|ui| {
                                                for val in v.as_mut() {
                                                    ui.allocate_ui_with_layout(egui::Vec2::new(w3, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                        ui.set_clip_rect(ui.max_rect());
                                                        ui.add_sized([w3, 20.], MathDragValue::new(val, &mut vars).speed(0.1).max_decimals(3));
                                                    });
                                                }
                                            });
                                        };

                                        let quat_row = |ui: &mut egui::Ui, q: &mut Quat, mode: &mut RotationDisplayMode| {
                                            ui.horizontal(|ui| {
                                                let mode_label = match mode {
                                                    RotationDisplayMode::Quaternion => "QUAT",
                                                    RotationDisplayMode::Euler(s) => s.label()
                                                };
                                                if ui.small_button(mode_label).clicked() {
                                                    *mode = mode.cycle();
                                                }
                                            });

                                            match mode {
                                                RotationDisplayMode::Quaternion => {
                                                    let mut v = q.to_array();
                                                    let mut vars = HashMap::new();
                                                    vars.insert("x".to_string(), v[0]);
                                                    vars.insert("y".to_string(), v[1]);
                                                    vars.insert("z".to_string(), v[2]);
                                                    vars.insert("w".to_string(), v[3]);
                                                    ui.horizontal(|ui| {
                                                        for val in &mut v[0..2] {
                                                            ui.allocate_ui_with_layout(egui::Vec2::new(w2, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                                ui.set_clip_rect(ui.max_rect());
                                                                ui.add_sized([w2, 20.], MathDragValue::new(val, &mut vars).speed(0.001).max_decimals(3));
                                                            });
                                                        }
                                                    });
                                                    ui.horizontal(|ui| {
                                                        for val in &mut v[2..4] {
                                                            ui.allocate_ui_with_layout(egui::Vec2::new(w2, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                                ui.set_clip_rect(ui.max_rect());
                                                                ui.add_sized([w2, 20.], MathDragValue::new(val, &mut vars).speed(0.001).max_decimals(3));
                                                            });
                                                        }
                                                    });
                                                    *q = Quat::from_array(v);
                                                }
                                                RotationDisplayMode::Euler(swizzle) => {
                                                    let (ax, ay, az) = q.to_euler(swizzle.to_glam());
                                                    let [n1, n2, n3] = swizzle.names();
                                                    let normalize_angle = |d: f32| {
                                                        let d = if d == -0.0 { 0.0 } else { d };
                                                        if (d - 180.0).abs() < 0.001 || (d + 180.0).abs() < 0.001 { 180.0 } else { d }
                                                    };

                                                    let mut degrees = [
                                                        normalize_angle(ax.to_degrees()),
                                                        normalize_angle(ay.to_degrees()),
                                                        normalize_angle(az.to_degrees()),
                                                    ];

                                                    let mut vars = HashMap::new();
                                                    vars.insert(n1.to_string(), degrees[0]);
                                                    vars.insert(n2.to_string(), degrees[1]);
                                                    vars.insert(n3.to_string(), degrees[2]);

                                                    ui.horizontal(|ui| {
                                                        for val in &mut degrees {
                                                            ui.allocate_ui_with_layout(egui::Vec2::new(w3, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                                ui.set_clip_rect(ui.max_rect());
                                                                ui.add_sized([w3, 20.], MathDragValue::new(val, &mut vars).speed(0.5).max_decimals(1).suffix("\u{b0}").degrees());
                                                            });
                                                        }
                                                    });
                                                    let rot = glam::EulerRot::from(*swizzle);
                                                    *q = Quat::from_euler(
                                                        rot,
                                                        degrees[0].to_radians(),
                                                        degrees[1].to_radians(),
                                                        degrees[2].to_radians(),
                                                    );
                                                }
                                            }
                                        };

                                        let modes = self.state.ui.view_rotation_modes.entry(sel).or_default();
                                        if modes.len() <= i {
                                            modes.push([RotationDisplayMode::Euler(EulerSwizzle::YXZ), RotationDisplayMode::Euler(EulerSwizzle::YXZ)]);
                                        }
                                        let [rot_mode, off_mode] = &mut self.state.ui.view_rotation_modes
                                            .get_mut(&sel).unwrap()[i];

                                        ui.label("Position");
                                        vec3_row(ui, &mut placement.position);

                                        ui.label("Rotation");
                                        quat_row(ui, &mut placement.rotation, rot_mode);

                                        ui.horizontal(|ui| {
                                            ui.allocate_ui_with_layout(egui::Vec2::new(w3, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.set_clip_rect(ui.max_rect());
                                                ui.add(egui::DragValue::new(&mut placement.count).speed(1).range(1..=u32::MAX));
                                            });
                                            ui.label("Count");
                                        });

                                        ui.label("Offset Position");
                                        vec3_row(ui, &mut placement.offset_pos);

                                        ui.label("Offset Rotation");
                                        quat_row(ui, &mut placement.offset_rot, off_mode);

                                        if ui.add_sized([ui.available_width(), 20.0], egui::Button::new("Delete")).clicked() {
                                            to_remove = Some(i);
                                        }
                                    }

                                    ui.separator();
                                }

                                if let Some(rem) = to_remove {
                                    mesh.placements.remove(rem);
                                    if let Some(collapsed) = self.state.ui.collapsed.get_mut(&sel) {
                                        collapsed.remove(rem);
                                    }
                                    if let Some(modes) = self.state.ui.view_rotation_modes.get_mut(&sel) {
                                        modes.remove(rem);
                                    }
                                }
                            }
                        },
                        editor::EditorMode::Assembly => {

                        },
                        editor::EditorMode::Edit => {

                        },
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.state.vp_rect = rect;

                let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                self.handle_3d_input(&resp, ctx, gl);


                let s = unsafe { UnsafeMutRef::new(self) };

                ui.painter().add(egui::PaintCallback {
                    rect,
                    callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |_info, painter| {
                        let gl = painter.gl();
                        unsafe {

                            let w = rect.width();
                            let h = rect.height();
                            let vp = s.ref_mut().cam().vp(w, h);

                            gl.clear_color(0.07, 0.08, 0.11, 1.);
                            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                            gl.enable(glow::DEPTH_TEST);

                            if s.state.show_grid {
                                let flat = s.render.renderer.flat;
                                gl.use_program(Some(flat));
                                if let Some(l) = gl.get_uniform_location(flat, "uMVP") {
                                    gl.uniform_matrix_4_f32_slice(Some(&l), false, &vp.to_cols_array());
                                }
                                if let Some(l) = gl.get_uniform_location(flat, "uColor") {
                                    gl.uniform_4_f32(Some(&l), 0.27, 0.27, 0.34, 0.5);
                                }
                                gl.bind_vertex_array(Some(s.render.renderer.grid_vao));
                                gl.draw_arrays(glow::LINES, 0, s.render.renderer.grid_n);
                                gl.line_width(2.);
                                if let Some(l) = gl.get_uniform_location(flat, "uColor") {
                                    gl.uniform_4_f32(Some(&l), 0.85, 0.2, 0.2, 0.9);
                                }
                                gl.bind_vertex_array(Some(s.render.renderer.axis_vao));
                                gl.draw_arrays(glow::LINES, 0, 2);
                                if let Some(l) = gl.get_uniform_location(s.render.renderer.flat, "uColor") {
                                    gl.uniform_4_f32(Some(&l), 0.2, 0.45, 0.9, 0.9);
                                }
                                gl.draw_arrays(glow::LINES, 2, 2);
                                gl.line_width(1.);
                                gl.bind_vertex_array(None);
                            }

                            match s.mode {
                                editor::EditorMode::View => {
                                    let mut calls = Vec::new();
                                    for vm in s.view.meshes.iter() {
                                        let mut draws = Vec::new();
                                        if let Some(mesh) = vm.render_view_placements(&mut draws) {
                                            calls.push(MeshDrawCall { mesh, instances: draws, wireframe: s.state.wireframe })
                                        }
                                    }

                                    s.render.renderer.draw_meshes(gl, &vp, &calls);
                                }
                                editor::EditorMode::Assembly => {
                                    if let Some(sel) = s.editor.mesh && let Some(mesh) = s.view.meshes.get(sel) {
                                        let mut instances = Vec::new();
                                        if let Some(mesh) = mesh.render_assembly(&mut instances) {
                                            s.render.renderer.draw_meshes(gl, &vp, &[
                                                MeshDrawCall { mesh, instances, wireframe: s.state.wireframe }
                                            ]);
                                        }
                                    }
                                }
                                editor::EditorMode::Edit => {

                                }
                            }

                        }
                    }))
                });

            });

    }
}

pub fn main() -> Result<(), eframe::Error> {
    let path = std::env::args().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Beatcraft LightMesh Editor")
            .with_inner_size([1440., 860.]),
        multisampling: 4,
        depth_buffer: 24,
        ..Default::default()
    };

    eframe::run_native(
        "Beatcraft LightMesh Editor",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, path)))),
    )

}
