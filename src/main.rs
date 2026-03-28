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
use std::hash::Hash;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

use eframe::glow::{self, HasContext};
use egui::{Frame, Sense, Ui};
use glam::{Mat4, Quat, Vec3};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;

use self::easing::Easing;
use self::editor::{App, RotationDisplayMode, ViewPlacement, WorkingRenameKey};
use self::light_mesh::BloomfogStyle;
use self::render::{InstanceData, MeshDrawCall, PointDrawCall};
use self::widgets::{MathDragValue, MathDragValueOpt};

pub mod data;
pub mod easing;
pub mod editor;
pub mod light_mesh;
pub mod math_interp;
pub mod renaming;
pub mod render;
pub mod widgets;

#[derive(Copy, Clone)]
struct UnsafeMutRef<T: 'static> {
    t: *mut T,
}

impl<T: 'static> UnsafeMutRef<T> {
    pub unsafe fn new(t: &mut T) -> Self {
        let ptr = t as *mut T;
        Self { t: ptr }
    }
}

impl<T> Deref for UnsafeMutRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.t.cast::<T>() }
    }
}

impl<T> UnsafeMutRef<T> {
    pub fn ref_mut(&self) -> &'static mut T {
        unsafe { &mut *self.t.cast::<T>() }
    }
}

unsafe impl<T> Send for UnsafeMutRef<T> {}
unsafe impl<T> Sync for UnsafeMutRef<T> {}

pub struct RefDuper;

impl RefDuper {
    /// Detaches a reference from it's owner, allowing
    /// mutable references to exist simultaneously
    /// SAFETY: attaches the lifetime to self for
    /// the illusion of safety
    pub(crate) unsafe fn detach_ref<'a, T>(&'a self, t: &T) -> &'a T {
        unsafe { &*(t as *const T) }
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

fn vec3_row<T: 'static + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    v: &mut Vec3,
    w3: f32,
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), v.x);
    vars.insert("y".to_string(), v.y);
    vars.insert("z".to_string(), v.z);
    ui.horizontal(|ui| {
        let mut current = *v;
        for val in current.as_mut() {
            ui.allocate_ui_with_layout(
                (w3, 20.).into(),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let id = ui.next_auto_id();
                    ui.set_clip_rect(ui.max_rect());
                    let resp = ui.add_sized(
                        [w3, 20.],
                        MathDragValue::new(val, &mut vars)
                            .speed(0.1)
                            .max_decimals(3),
                    );

                    if resp.changed() {
                        on_change();
                    }
                    if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                        ui.memory_mut(|m| {
                            m.data.insert_temp(id, *v);
                            m.data.insert_temp(id, snapshot_provider());
                        });
                    }
                    if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                    && let Some((old, t)) = ui.memory_mut(|m| {
                        let o = m.data.get_temp::<Vec3>(id)?;
                        let t = m.data.get_temp::<T>(id)?;
                        Some((o, t))
                    })
                    && old != *v {
                        history_pusher(t);
                    }
                },
            );
        }
        *v = current;
    });
}

fn vec3_opt_row<T: 'static + Clone + Send + Sync>(
    ui: &mut Ui,
    mut v: [&mut Option<f32>; 3],
    w3: f32,
    vars: &mut HashMap<String, f32>,
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    if let Some(x) = v[0] {
        vars.insert("x".into(), *x);
    }
    if let Some(y) = v[1] {
        vars.insert("y".into(), *y);
    }
    if let Some(z) = v[2] {
        vars.insert("z".into(), *z);
    }
    ui.horizontal(|ui| {
        let mut current = [*v[0], *v[1], *v[2]];
        for val in current.iter_mut() {
            ui.allocate_ui_with_layout(
                (w3, 20.).into(),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let id = ui.next_auto_id();
                    ui.set_clip_rect(ui.max_rect());
                    let resp = ui.add_sized(
                        [w3, 20.],
                        MathDragValueOpt::new(val, vars)
                            .speed(0.1)
                            .max_decimals(3),
                    );

                    if resp.changed() {
                        on_change();
                    }

                    if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                        ui.memory_mut(|m| {
                            let c2 = [*v[0], *v[1], *v[2]];
                            m.data.insert_temp(id, c2);
                            m.data.insert_temp(id, snapshot_provider());
                        });
                    }
                    if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                    && let Some((old, t)) = ui.memory_mut(|m| {
                        let o = m.data.get_temp::<[Option<f32>; 3]>(id)?;
                        let t = m.data.get_temp::<T>(id)?;
                        Some((o, t))
                    })
                    && (old[0] != *v[0] || old[1] != *v[1] || old[2] != *v[2]) {
                        history_pusher(t);
                    }
                }
            );
        }
        *v[0] = current[0];
        *v[1] = current[1];
        *v[2] = current[2];
    });
}

fn delta_function_row<T: 'static + Clone + Send + Sync>(
    ui: &mut Ui,
    func_delta_vars: (&mut Easing, &mut Option<f32>, &mut HashMap<String, f32>),
    salt: impl Hash,
    w: (f32, f32),
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let (func, delta, vars) = func_delta_vars;
    let (w2, w3) = w;
    if let Some(d) = delta {
        vars.insert("d".into(), *d);
    }

    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            (w2, 20.).into(),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let id = ui.next_auto_id();
                ui.set_clip_rect(ui.max_rect());
                let resp = ui.add_sized(
                    [w2, 20.],
                    MathDragValueOpt::new(delta, vars)
                        .speed(0.1)
                        .max_decimals(3),
                );

                if resp.changed() {
                    on_change();
                }

                if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                    ui.memory_mut(|m| {
                        m.data.insert_temp(id, *delta);
                        m.data.insert_temp(id, snapshot_provider());
                    });
                }
                if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                && let Some((old, t)) = ui.memory_mut(|m| {
                    let o = m.data.get_temp::<Option<f32>>(id)?;
                    let t = m.data.get_temp::<T>(id)?;
                    Some((o, t))
                })
                && old == *delta {
                    history_pusher(t);
                }

            }
        );
        ui.allocate_ui_with_layout(
            (w3, 20.).into(),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let old = *func;
                ui.set_clip_rect(ui.max_rect());
                egui::ComboBox::from_id_salt(egui::Id::new("delta_function").with(salt))
                    .selected_text(func.display_name())
                    .width(w3)
                    .show_ui(ui, |ui| {
                        for (name, easing) in Easing::iter_all() {
                            ui.selectable_value(
                                func, easing, name
                            );
                        }
                    });
                if old != *func {
                    on_change();
                    history_pusher(snapshot_provider());
                }
            }
        );
    });
}


fn quat_row<T: 'static + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    q: &mut Quat,
    mode: &mut RotationDisplayMode,
    w: (f32, f32),
    snapshot_provider: impl Fn() -> T,
    mut history_pusher: impl FnMut(T),
    mut on_change: impl FnMut(),
) {
    let (w2, w3) = w;
    ui.horizontal(|ui| {
        let mode_label = match mode {
            RotationDisplayMode::Quaternion => "QUAT",
            RotationDisplayMode::Euler(s) => s.label(),
        };
        if ui.small_button(mode_label).clicked() {
            *mode = mode.cycle();
        }
    });
    let current = *q;
    match mode {
        RotationDisplayMode::Quaternion => {
            let mut v = current.to_array();
            let mut vars = HashMap::new();
            vars.insert("x".to_string(), v[0]);
            vars.insert("y".to_string(), v[1]);
            vars.insert("z".to_string(), v[2]);
            vars.insert("w".to_string(), v[3]);
            ui.horizontal(|ui| {
                for val in &mut v[0..2] {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(w2, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let id = ui.next_auto_id();
                            ui.set_clip_rect(ui.max_rect());
                            let resp = ui.add_sized(
                                [w2, 20.],
                                MathDragValue::new(val, &mut vars)
                                    .speed(0.001)
                                    .max_decimals(3),
                            );

                            if resp.changed() {
                                on_change();
                            }

                            if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                                ui.memory_mut(|m| {
                                    m.data.insert_temp(id, *q);
                                    m.data.insert_temp(id, snapshot_provider());
                                });
                            }
                            if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                            && let Some((old, t)) = ui.memory_mut(|m| {
                                let o = m.data.get_temp::<Quat>(id)?;
                                let t = m.data.get_temp::<T>(id)?;
                                Some((o, t))
                            })
                            && old != *q {
                                history_pusher(t);
                            }

                        },
                    );

                }
            });
            ui.horizontal(|ui| {
                for val in &mut v[2..4] {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(w2, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let id = ui.next_auto_id();
                            ui.set_clip_rect(ui.max_rect());
                            let resp = ui.add_sized(
                                [w2, 20.],
                                MathDragValue::new(val, &mut vars)
                                    .speed(0.001)
                                    .max_decimals(3),
                            );

                            if resp.changed() {
                                on_change();
                            }

                            if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                                ui.memory_mut(|m| {
                                    m.data.insert_temp(id, *q);
                                    m.data.insert_temp(id, snapshot_provider());
                                });
                            }
                            if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                            && let Some((old, t)) = ui.memory_mut(|m| {
                                let o = m.data.get_temp::<Quat>(id)?;
                                let t = m.data.get_temp::<T>(id)?;
                                Some((o, t))
                            })
                            && old != *q {
                                history_pusher(t);
                            }
                        },
                    );
                }
            });
            *q = Quat::from_array(v);
        }
        RotationDisplayMode::Euler(swizzle) => {
            let (ax, ay, az) = q.to_euler(swizzle.to_glam());
            let [n1, n2, n3] = swizzle.names();
            let normalize_angle = |d: f32| {
                let d = if d == -0.0 { 0.0 } else { d };
                if (d - 180.0).abs() < 0.001 || (d + 180.0).abs() < 0.001 {
                    180.0
                } else {
                    d
                }
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
                    ui.allocate_ui_with_layout(
                        (w3, 20.).into(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let id = ui.next_auto_id();
                            ui.set_clip_rect(ui.max_rect());
                            let resp = ui.add_sized(
                                [w3, 20.],
                                MathDragValue::new(val, &mut vars)
                                    .speed(0.5)
                                    .max_decimals(1)
                                    .suffix("\u{b0}")
                                    .degrees(),
                            );

                            if resp.changed() {
                                on_change();
                            }

                            if resp.drag_started() || (resp.gained_focus() && !resp.dragged()) {
                                ui.memory_mut(|m| {
                                    m.data.insert_temp(id, *q);
                                    m.data.insert_temp(id, snapshot_provider());
                                });
                            }
                            if (resp.drag_stopped() || (resp.lost_focus() && !resp.dragged()))
                            && let Some((old, t)) = ui.memory_mut(|m| {
                                let o = m.data.get_temp::<Quat>(id)?;
                                let t = m.data.get_temp::<T>(id)?;
                                Some((o, t))
                            })
                            && old != *q {
                                history_pusher(t);
                            }
                        },
                    );
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let gl = frame.gl().unwrap();

        if self.state.dirty {
            self.rebuild_meshes(gl);
        }

        self.handle_keys(ctx, gl);

        let dt = ctx.input(|i| i.unstable_dt);
        if self.state.status_timer > 0. {
            if let Some(t) = self.state.title_content.as_mut() && !t.is_empty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{} {}", self.title, t)));
                t.clear();
            }
            self.state.status_timer -= dt;
        } else if self.state.title_content.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title.to_string()));
            self.state.title_content = None;
        }

        self.handle_file_open(gl);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // Open Session...
                    if ui.button("Open Session\u{2026}  \u{2502}").clicked() && !self.block_input()
                    {
                        let (sx, rx) = mpsc::channel();
                        self.state.ui.open_session_channel = Some(rx);
                        std::thread::spawn(move || {
                            if let Some(session) = rfd::FileDialog::new()
                                .set_title("Open Session...")
                                .add_filter("json", &["json"])
                                .pick_file()
                            {
                                let _ = sx.send(session);
                            }
                        });
                    }
                    // Open...
                    if ui.button("Open\u{2026}          \u{2502}").clicked() && !self.block_input()
                    {
                        let (sx, rx) = mpsc::channel();
                        self.state.ui.open_mesh_channel = Some(rx);
                        std::thread::spawn(move || {
                            if let Some(meshes) = rfd::FileDialog::new()
                                .set_title("Open Meshes...")
                                .add_filter("json", &["json"])
                                .pick_files()
                            {
                                let _ = sx.send(meshes);
                            }
                        });
                    }
                    if ui.button("Save           \u{2502} [Ctrl+S]").clicked() {
                        match self.mode {
                            editor::EditorMode::View => {}
                            editor::EditorMode::Assembly | editor::EditorMode::Edit => {}
                        }
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo  \u{2502}       [Ctrl+Z]").clicked() {
                        self.undo(gl);
                        ui.close();
                    }
                    if ui.button("Redo  \u{2502} [Ctrl+Shift+Z]").clicked() {
                        self.redo(gl);
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.state.wireframe, "Wireframe     \u{2502} [W]");
                    ui.checkbox(&mut self.state.show_grid, "Show Grid     \u{2502} [G]");
                    ui.checkbox(&mut self.state.show_verts, "Vertices      \u{2502} [C]");
                    if ui.button("Reframe Camera  \u{2502} [F]").clicked() {
                        self.frame_to_geometry();
                        ui.close();
                    }
                });
                if !self.state.status.is_empty() && self.state.status_timer > 0. {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(&self.state.status)
                                .color(egui::Color32::from_rgb(140, 200, 140)),
                        );
                    });
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mode = match self.mode {
                    editor::EditorMode::Assembly => "Assembly ",
                    editor::EditorMode::Edit => "Part Edit",
                    editor::EditorMode::View => "View     ",
                };
                ui.label(format!(
                    "Mode: {mode} [E]  |  Reframe [F]  |  [W]ireframe / [G]rid / [V]ertices"
                ));
            });
        });

        egui::SidePanel::left("left_panel")
            .exact_width(250.)
            .resizable(false)
            .show(ctx, |ui| {
                ui.allocate_ui(ui.available_size(), |ui| {
                    ui.allocate_exact_size((230., 1.).into(), Sense::empty());
                    ui.allocate_ui((ui.available_width(), ui.available_height()-45.).into(), |ui| {
                        egui::ScrollArea::vertical().id_salt("left_p_scroll").show(ui, |ui| match self.mode {
                            editor::EditorMode::View => {
                                draw_view_left(self, ui);
                            }
                            editor::EditorMode::Assembly => {
                                draw_assembly_left(self, ui, gl);
                            }
                            editor::EditorMode::Edit => {
                                draw_edit_left(self, ui, gl);
                            }
                        });
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
                                ui.add_sized(
                                    [width, 20.],
                                    MathDragValue::new(&mut target.x, &mut vars).speed(0.1),
                                );
                                ui.add_sized(
                                    [width, 20.],
                                    MathDragValue::new(&mut target.y, &mut vars).speed(0.1),
                                );
                                ui.add_sized(
                                    [width, 20.],
                                    MathDragValue::new(&mut target.z, &mut vars).speed(0.1),
                                );
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
                egui::ScrollArea::vertical().show(ui, |ui| match self.mode {
                    editor::EditorMode::View => {
                        draw_view_right(self, ui, gl);
                    }
                    editor::EditorMode::Assembly => {
                        draw_assembly_right();
                    }
                    editor::EditorMode::Edit => {
                        draw_edit_right();
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
                    callback: Arc::new(eframe::egui_glow::CallbackFn::new(
                        move |_info, painter| {
                            let gl = painter.gl();
                            unsafe {
                                let w = rect.width();
                                let h = rect.height();
                                let vp = s.ref_mut().cam().vp(w, h);
                                let eye = s.ref_mut().cam().eye();

                                gl.clear_color(0.07, 0.08, 0.11, 1.);
                                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                                gl.enable(glow::DEPTH_TEST);

                                if s.state.show_grid {
                                    let flat = s.render.renderer.flat;
                                    gl.line_width(1.);
                                    gl.use_program(Some(flat));
                                    if let Some(l) = gl.get_uniform_location(flat, "uMVP") {
                                        gl.uniform_matrix_4_f32_slice(
                                            Some(&l),
                                            false,
                                            &vp.to_cols_array(),
                                        );
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
                                    if let Some(l) =
                                        gl.get_uniform_location(s.render.renderer.flat, "uColor")
                                    {
                                        gl.uniform_4_f32(Some(&l), 0.2, 0.45, 0.9, 0.9);
                                    }
                                    gl.draw_arrays(glow::LINES, 2, 2);
                                    gl.line_width(1.);
                                    gl.bind_vertex_array(None);
                                }

                                match s.mode {
                                    editor::EditorMode::View => {
                                        draw_view_gl(&s, gl, &vp, eye);
                                    }
                                    editor::EditorMode::Assembly => {
                                        draw_assembly_gl(&s, gl, &vp, eye);
                                    }
                                    editor::EditorMode::Edit => {
                                        draw_edit_gl(&s, gl, &vp, eye);
                                    }
                                }
                            }
                        },
                    )),
                });
            });
    }
}

fn draw_view_left(s: &mut App, ui: &mut Ui) {
    let mut to_remove = None;
    for (i, mesh) in s.view.meshes.iter_mut().enumerate() {
        let name = mesh.path.with_extension("");
        let name = name
            .file_name()
            .map(|x| x.to_string_lossy())
            .unwrap_or_else(|| std::borrow::Cow::Borrowed("?"));

        ui.add_space(2.0);
        let selected = s.state.ui.view_mesh == Some(i);
        let available = ui.available_size_before_wrap();
        if ui
            .add_sized(
                egui::Vec2::new(available.x, 20.0),
                egui::Button::selectable(selected, name.as_ref()),
            )
            .clicked()
        {
            s.state.ui.view_mesh = if selected { None } else { Some(i) };
        };
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut mesh.visible, "");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    to_remove = Some(i);
                }
                if ui.button("Edit").clicked() {
                    s.editor.mesh = Some(i);
                    s.last_mode = s.mode;
                    s.mode = editor::EditorMode::Assembly;
                }
            });
        });

        ui.separator();
    }
    if let Some(i) = to_remove {
        s.view.meshes.remove(i);
        if let Some(sel) = s.state.ui.view_mesh {
            if sel == i {
                s.state.ui.view_mesh = None;
            } else if sel > i {
                s.state.ui.view_mesh = Some(sel - 1);
            }
        }
        if let Some(sel) = s.editor.mesh {
            if sel == i {
                s.editor.mesh = None;
            } else if sel > i {
                s.editor.mesh = Some(sel - 1);
            }
        }
    }
}

fn draw_view_right(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let s2 = unsafe { rd.detach_mut_ref(s) };
    let s3 = unsafe { rd.detach_mut_ref(s) };
    if let Some(sel) = s.state.ui.view_mesh
        && let Some(mesh) = s.view.meshes.get_mut(sel)
    {
        let rd2 = RefDuper;
        let mesh2 = unsafe { rd2.detach_mut_ref(mesh) };
        if ui.button("+ Add Placement").clicked() {
            mesh.placements.push(ViewPlacement::default());
            s.state.ui.collapsed.entry(sel).or_default().push(false);
            s.state
                .ui
                .view_rotation_modes
                .entry(sel)
                .or_default()
                .push(Default::default());
        }

        let mut to_remove = None;
        for (i, placement) in mesh.placements.iter_mut().enumerate() {
            let collapsed = s.state.ui.collapsed.entry(sel).or_default();
            if collapsed.len() <= i {
                collapsed.push(false);
            }
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

                let modes = s.state.ui.view_rotation_modes.entry(sel).or_default();
                if modes.len() <= i {
                    modes.push(Default::default());
                }
                let [rot_mode, off_mode] =
                    &mut s.state.ui.view_rotation_modes.get_mut(&sel).unwrap()[i];

                ui.label("Position");
                vec3_row(
                    ui, &mut placement.position, w3,
                    || mesh2.placements.clone(),
                    |t| s2.add_history(editor::HistoryEntry::ViewPlacement(
                        editor::ViewPlacementsSnapshot {
                            idx: sel,
                            placements: t
                        }
                    )),
                    || s3.rebuild_meshes(gl)
                );

                ui.label("Rotation");
                quat_row(
                    ui, &mut placement.rotation, rot_mode, (w2, w3),
                    || mesh2.placements.clone(),
                    |t| s2.add_history(editor::HistoryEntry::ViewPlacement(
                        editor::ViewPlacementsSnapshot {
                            idx: sel,
                            placements: t
                        }
                    )),
                    || s3.rebuild_meshes(gl)
                );

                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(w3, 20.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_clip_rect(ui.max_rect());
                            ui.add(
                                egui::DragValue::new(&mut placement.count)
                                    .speed(1)
                                    .range(1..=u32::MAX),
                            );
                        },
                    );
                    ui.label("Count");
                });

                ui.label("Offset Position");
                vec3_row(
                    ui, &mut placement.offset_pos, w3,
                    || mesh2.placements.clone(),
                    |t| s2.add_history(editor::HistoryEntry::ViewPlacement(
                        editor::ViewPlacementsSnapshot {
                            idx: sel,
                            placements: t
                        }
                    )),
                    || s3.rebuild_meshes(gl)
                );

                ui.label("Offset Rotation");
                quat_row(
                    ui, &mut placement.offset_rot, off_mode, (w2, w3),
                    || mesh2.placements.clone(),
                    |t| s2.add_history(editor::HistoryEntry::ViewPlacement(
                        editor::ViewPlacementsSnapshot {
                            idx: sel,
                            placements: t
                        }
                    )),
                    || s3.rebuild_meshes(gl)
                );

                if ui
                    .add_sized([ui.available_width(), 20.0], egui::Button::new("Delete"))
                    .clicked()
                {
                    to_remove = Some(i);
                }
            }

            ui.separator();
        }

        if let Some(rem) = to_remove {
            mesh.placements.remove(rem);
            if let Some(collapsed) = s.state.ui.collapsed.get_mut(&sel) {
                collapsed.remove(rem);
            }
            if let Some(modes) = s.state.ui.view_rotation_modes.get_mut(&sel) {
                modes.remove(rem);
            }
        }
    }
}

fn draw_view_gl(s: &UnsafeMutRef<App>, gl: &glow::Context, vp: &Mat4, eye: Vec3) {
    let mut calls = Vec::new();
    for vm in s.view.meshes.iter() {
        let mut draws = Vec::new();
        if let Some(mesh) = vm.render_view_placements(&mut draws) {
            calls.push(MeshDrawCall {
                mesh,
                instances: draws,
                wireframe: s.state.wireframe,
            })
        }
    }
    s.render.renderer.draw_meshes(gl, vp, eye, &calls);
}

fn draw_assembly_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    let rd = RefDuper;
    let self2 = unsafe { rd.detach_mut_ref(s) };
    let self3 = unsafe { rd.detach_mut_ref(s) };
    let self4 = unsafe { rd.detach_mut_ref(s) };
    if let Some(mesh) = s.get_current_view_mesh_mut() {
        let rd2 = RefDuper;
        let mesh2 = unsafe { rd2.detach_mut_ref(mesh) };
        let path = mesh.path.clone();
        let part_names = mesh.data.part_names.clone();
        let toggles = self2
            .state
            .ui
            .assembly_collapsed
            .entry(path.clone())
            .or_default();
        let w = ui.available_width() - ui.spacing().item_spacing.x;
        let w2 = (w - ui.spacing().item_spacing.x) / 2.0;
        let w3 = (w - ui.spacing().item_spacing.x * 2.0) / 3.0;

        ui.horizontal(|ui| {
            let icon = if toggles.placements { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                toggles.placements = !toggles.placements;
            }
            ui.label("Placements");
        });

        if !toggles.placements {
            if ui
                .add_sized([w, 20.], egui::Button::new("+ Add Placement"))
                .clicked()
                && let Some(first) = part_names.first()
            {
                mesh.data.placements.push(light_mesh::Placement {
                    part: first.clone(),
                    position: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                    remap_data: IndexMap::new(),
                });
            }

            let mut to_remove = None;
            for (pi, placement) in mesh.data.placements.iter_mut().enumerate() {
                let pt_collapsed = toggles
                    .placement_parts
                    .entry(pi)
                    .or_insert(([true, true, true], Default::default()));

                ui.horizontal(|ui| {
                    let icon = if pt_collapsed.0[0] { "▶" } else { "▼" };
                    if ui.small_button(icon).clicked() {
                        pt_collapsed.0[0] = !pt_collapsed.0[0];
                    }

                    egui::ComboBox::from_id_salt(egui::Id::new("placement_part").with(pi))
                        .selected_text(placement.part.as_str())
                        .width(w - ui.spacing().item_spacing.x * 2. - 24.)
                        .show_ui(ui, |ui| {
                            for name in &part_names {
                                ui.selectable_value(
                                    &mut placement.part,
                                    name.clone(),
                                    name.as_str(),
                                );
                            }
                        });

                    if ui.small_button("×").clicked() {
                        to_remove = Some(pi);
                    }
                });

                if !pt_collapsed.0[0] {
                    let rot_mode = &mut pt_collapsed.1;

                    // Position
                    ui.horizontal(|ui| {
                        ui.label("Position");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Paste").clicked() { /* TODO */ }
                            if ui.small_button("Copy").clicked() { /* TODO */ }
                        });
                    });
                    vec3_row(
                        ui, &mut placement.position, w3,
                        || mesh2.data.placements.clone(),
                        |t| self3.add_history(editor::HistoryEntry::MeshPlacement(
                            light_mesh::LightMeshPlacementSnapshot {
                                view_idx: self3.get_current_mesh_idx().unwrap(),
                                placements: t
                            }
                        )),
                        || self4.rebuild_meshes(gl)
                    );

                    // Rotation
                    ui.horizontal(|ui| {
                        ui.label("Rotation");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Paste").clicked() { /* TODO */ }
                            if ui.small_button("Copy").clicked() { /* TODO */ }
                        });
                    });
                    quat_row(
                        ui, &mut placement.rotation, rot_mode, (w2, w3),
                        || mesh2.data.placements.clone(),
                        |t| self3.add_history(editor::HistoryEntry::MeshPlacement(
                            light_mesh::LightMeshPlacementSnapshot {
                                view_idx: self3.get_current_mesh_idx().unwrap(),
                                placements: t
                            }
                        )),
                        || self4.rebuild_meshes(gl)
                    );

                    // Scale
                    ui.horizontal(|ui| {
                        ui.label("Scale");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Paste").clicked() { /* TODO */ }
                            if ui.small_button("Copy").clicked() { /* TODO */ }
                            // TODO: re-add lock system when it works
                            // correctly
                            // let lock_icon = if pt_collapsed.0[2] { "[#]" } else { "[ ]" };
                            // if ui.small_button(lock_icon).clicked() {
                            //     pt_collapsed.0[2] = !pt_collapsed.0[2];
                            // }
                        });
                    });
                    vec3_row(
                        ui, &mut placement.scale, w3,
                        || mesh2.data.placements.clone(),
                        |t| self3.add_history(editor::HistoryEntry::MeshPlacement(
                            light_mesh::LightMeshPlacementSnapshot {
                                view_idx: self3.get_current_mesh_idx().unwrap(),
                                placements: t
                            }
                        )),
                        || self4.rebuild_meshes(gl)
                    );

                    // Remap Data
                    ui.horizontal(|ui| {
                        let icon = if pt_collapsed.0[1] { "▶" } else { "▼" };
                        if ui.small_button(icon).clicked() {
                            pt_collapsed.0[1] = !pt_collapsed.0[1];
                        }
                        ui.label("Remap Data");
                    });

                    if !pt_collapsed.0[1] {
                        let mut remap_to_remove = None;
                        for (ri, (from, to)) in placement.remap_data.iter_mut2().enumerate() {
                            ui.horizontal(|ui| {
                                let fw = (w - ui.spacing().item_spacing.x * 3. - 24.) / 2.;
                                ui.add_sized([fw, 20.], egui::TextEdit::singleline(from));
                                ui.label("→");
                                ui.add_sized([fw, 20.], egui::TextEdit::singleline(to));
                                if ui.small_button("×").clicked() {
                                    remap_to_remove = Some(ri);
                                }
                            });
                        }
                        if let Some(ri) = remap_to_remove {
                            placement.remap_data.shift_remove_index(ri);
                        }
                        if ui.add_sized([w, 20.], egui::Button::new("+ Add")).clicked() {
                            placement.remap_data.insert(String::new(), String::new());
                        }
                    }
                }

                ui.separator();
            }

            if let Some(i) = to_remove {
                mesh.data.placements.remove(i);
                toggles.placement_parts.remove(&i);
            }
        }

        ui.horizontal(|ui| {
            let icon = if toggles.data { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                toggles.data = !toggles.data;
            }
            ui.label("Data");
        });

        if !toggles.data {
            if ui
                .add_sized([w, 20.], egui::Button::new("+ Add Data"))
                .clicked()
            {
                mesh.data.data.insert(
                    format!("new_data_{}", mesh.data.data.len()),
                    Default::default(),
                );
            }

            let mut data_to_remove = None;
            let data_keys: Vec<String> = mesh.data.data.keys().cloned().collect();
            for (di, key) in data_keys.iter().enumerate() {
                let entry = mesh.data.data.get_mut(key).unwrap();
                let di_collapsed = toggles.datas.entry(di).or_insert(true);

                if ui
                    .horizontal(|ui| {
                        let icon = if *di_collapsed { "▶" } else { "▼" };
                        if ui.small_button(icon).clicked() {
                            *di_collapsed = !*di_collapsed;
                        }
                        let mut name = key.clone();
                        if let WorkingRenameKey::DataTag(ref name2) = self2.state.ui.working_key
                            && *name2 == name
                        {
                            name = self2.state.ui.working_name.take().unwrap_or(name);
                        }
                        if ui
                            .add_sized(
                                [w - ui.spacing().item_spacing.x * 2. - 24., 20.],
                                egui::TextEdit::singleline(&mut name),
                            )
                            .changed()
                        {
                            let _ = self3.rename(editor::Rename::DataTag {
                                view_idx: self3.get_current_mesh_idx().unwrap(),
                                swap: editor::DataSwap {
                                    from: key.clone(),
                                    to: name,
                                },
                            });
                            self3.state.ui.working_key = WorkingRenameKey::None;
                            return true;
                        }
                        if name != *key {
                            self2.state.ui.working_key = WorkingRenameKey::DataTag(key.clone());
                            self2.state.ui.working_name = Some(name);
                        }
                        if ui.small_button("×").clicked() {
                            data_to_remove = Some(key.clone());
                        }
                        false
                    })
                    .inner
                {
                    break;
                };

                if !*di_collapsed {
                    ui.horizontal(|ui| {
                        // Mat: click-cycle 0->1->2->0
                        let mat_label = format!("Material {}", entry.material);
                        if ui.button(mat_label).clicked() {
                            entry.material = (entry.material + 1) % 3;
                        }

                        // Ch: dropdown 0..7
                        egui::ComboBox::from_id_salt(egui::Id::new("data_ch").with(di))
                            .selected_text(format!("Channel {}", entry.color))
                            .show_ui(ui, |ui| {
                                for ch in 0u8..8 {
                                    ui.selectable_value(&mut entry.color, ch, ch.to_string());
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Texture");
                        ui.add(
                            egui::DragValue::new(&mut entry.texture)
                                .range(0..=255u8)
                                .speed(1),
                        );
                    });
                }

                ui.separator();
            }

            if let Some(key) = data_to_remove {
                mesh.data.data.shift_remove(&key);
            }
        }

        ui.horizontal(|ui| {
            let icon = if toggles.textures { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                toggles.textures = !toggles.textures;
            }
            ui.label("Textures");
        });

        if !toggles.textures {
            if ui
                .add_sized([w, 20.], egui::Button::new("+ Add Texture"))
                .clicked()
            {
                mesh.data
                    .textures
                    .insert(format!("{}", mesh.data.textures.len()), String::new());
            }

            let mut tex_to_remove = None;
            let tex_keys: Vec<String> = mesh.data.textures.keys().cloned().collect();
            for (ti, key) in tex_keys.iter().enumerate() {
                let val = mesh.data.textures.get_mut(key).unwrap();
                ui.horizontal(|ui| {
                    ui.label(format!("{ti}"));
                    ui.add_sized(
                        [w - ui.spacing().item_spacing.x * 2. - 24. - 16., 20.],
                        egui::TextEdit::singleline(val),
                    );
                    if ui.small_button("×").clicked() {
                        tex_to_remove = Some(key.clone());
                    }
                });
            }

            if let Some(key) = tex_to_remove {
                mesh.data.textures.shift_remove(&key);
            }
        }

        ui.horizontal(|ui| {
            let icon = if toggles.settings { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                toggles.settings = !toggles.settings;
            }
            ui.label("Render Settings");
        });

        if !toggles.settings {
            ui.horizontal(|ui| {
                ui.checkbox(&mut mesh.data.cull, "Cull");
                ui.checkbox(&mut mesh.data.do_bloom, "Bloom");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut mesh.data.do_mirroring, "Mirror");
                ui.checkbox(&mut mesh.data.do_solid, "Solid");
            });
            egui::ComboBox::from_id_salt("bloomfog_style")
                .selected_text(mesh.data.bloomfog_style.label())
                .width(w)
                .show_ui(ui, |ui| {
                    for style in [
                        BloomfogStyle::BloomOnly,
                        BloomfogStyle::Everything,
                        BloomfogStyle::Nothing,
                    ] {
                        ui.selectable_value(&mut mesh.data.bloomfog_style, style, style.label());
                    }
                });
        }

        ui.horizontal(|ui| {
            let icon = if toggles.credits { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                toggles.credits = !toggles.credits;
            }
            ui.label("Credits");
        });

        if !toggles.credits {
            let mut to_remove = None;
            for (ci, credit) in mesh.data.credits.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [w - ui.spacing().item_spacing.x - 24., 20.],
                        egui::TextEdit::singleline(credit),
                    );
                    if ui.small_button("×").clicked() {
                        to_remove = Some(ci);
                    }
                });
            }
            if let Some(i) = to_remove {
                mesh.data.credits.remove(i);
            }
            if ui
                .add_sized([w, 20.], egui::Button::new("+ Add Credit"))
                .clicked()
            {
                mesh.data.credits.push(String::new());
            }
        }
    }
}

fn draw_assembly_right() {}

fn draw_assembly_gl(s: &UnsafeMutRef<App>, gl: &glow::Context, vp: &Mat4, eye: Vec3) {
    if let Some(sel) = s.editor.mesh
        && let Some(mesh) = s.view.meshes.get(sel)
    {
        let mut instances = Vec::new();
        if let Some(mesh) = mesh.render_assembly(&mut instances) {
            s.render.renderer.draw_meshes(
                gl,
                vp,
                eye,
                &[MeshDrawCall {
                    mesh,
                    instances,
                    wireframe: s.state.wireframe,
                }],
            );
        }
    }
}

fn draw_edit_left(s: &mut App, ui: &mut Ui, gl: &glow::Context) {
    // TODO: add raw vertices, uvs, and normals
    let rd = RefDuper;
    let self2 = unsafe { rd.detach_mut_ref(s) };
    let self3 = unsafe { rd.detach_mut_ref(s) };
    let self4 = unsafe { rd.detach_mut_ref(s) };
    if let Some(part) = self2.get_current_part_mut() {
        let rd2 = RefDuper;
        let part2 = unsafe { rd2.detach_mut_ref(part) };
        let w = ui.available_width();
        //let w2 = (w - ui.spacing().item_spacing.x) / 2.;
        let w3 = (w - ui.spacing().item_spacing.x * 2.) / 3.;

        // Indexed vertices
        let verts = &mut s.state.ui.edit_collpased.i_vertices;
        ui.horizontal(|ui| {
            let icon = if *verts { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                *verts = !*verts;
            }
            ui.label("Indexed Vertices");
        });

        if !*verts {
            for vert in part.vertices.indexed.iter_mut() {
                vec3_row(
                    ui, vert, w3,
                    || part2.clone(),
                    |t| self3.add_history(editor::HistoryEntry::MeshPart(
                        light_mesh::LightMeshPartSnapshot {
                            idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                            part: Box::new(t)
                        }
                    )),
                    || self4.rebuild_meshes(gl)
                );
            }
        }

        // Named vertices
        let verts = &mut s.state.ui.edit_collpased.n_vertices;
        ui.horizontal(|ui| {
            let icon = if *verts { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                *verts = !*verts;
            }
            ui.label("Named Vertices");
        });

        if !*verts {
            for (key, vert) in part.vertices.named.iter_mut2() {
                let mut name = key.clone();
                if let WorkingRenameKey::NamedVert(ref name2) = self3.state.ui.working_key
                    && *name2 == name
                {
                    name = self3.state.ui.working_name.take().unwrap_or(name);
                }

                if ui
                    .add_sized(
                        [w, 20.],
                        egui::TextEdit::singleline(&mut name),
                    )
                    .changed()
                {
                    let _ = self3.rename(editor::Rename::Vertex {
                        part: editor::PartId {
                            view_idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                        },
                        swap: editor::DataSwap {
                            from: data::VertexId::Named(key.clone()),
                            to: data::VertexId::Named(name)
                        }
                    });
                    self3.state.ui.working_key = WorkingRenameKey::None;
                    return;
                }

                if name != *key {
                    self3.state.ui.working_key = WorkingRenameKey::NamedVert(key.clone());
                    self3.state.ui.working_name = Some(name);
                }

                vec3_row(
                    ui, vert, w3,
                    || part2.clone(),
                    |t| self3.add_history(editor::HistoryEntry::MeshPart(
                        light_mesh::LightMeshPartSnapshot {
                            idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                            part: Box::new(t)
                        }
                    )),
                    || self4.rebuild_meshes(gl)
                );

                // remove button
            }
        }

        let verts = &mut s.state.ui.edit_collpased.c_vertices;
        ui.horizontal(|ui| {
            let icon = if *verts { "▶" } else { "▼" };
            if ui.small_button(icon).clicked() {
                *verts = !*verts;
            }
            ui.label("Compute Vertices");
        });

        if !*verts {
            for (key, comp) in part.vertices.compute.iter_mut2() {
                let mut name = key.clone();
                if let WorkingRenameKey::CompVert(ref name2) = self3.state.ui.working_key
                    && *name2 == name {
                    name = self3.state.ui.working_name.take().unwrap_or(name);
                }

                if ui
                    .add_sized(
                        [w, 20.],
                        egui::TextEdit::singleline(&mut name),
                    )
                    .changed()
                {
                    let _ = self3.rename(editor::Rename::Vertex {
                        part: editor::PartId {
                            view_idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                        },
                        swap: editor::DataSwap {
                            from: data::VertexId::Named(key.clone()),
                            to: data::VertexId::Named(name),
                        }
                    });
                    self3.state.ui.working_key = WorkingRenameKey::None;
                    return;
                }

                if name != *key {
                    self3.state.ui.working_key = WorkingRenameKey::CompVert(key.clone());
                    self3.state.ui.working_name = Some(name);
                }

                let mut vars = HashMap::new();

                ui.label("D          Easing");
                delta_function_row(
                    ui, (&mut comp.function, &mut comp.delta, &mut vars),
                    key.as_str(),
                    (w3, w3*2. + ui.spacing().item_spacing.x),
                    || part2.clone(),
                    |t| self3.add_history(editor::HistoryEntry::MeshPart(
                        light_mesh::LightMeshPartSnapshot {
                            idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                            part: Box::new(t)
                        }
                    )),
                    || self4.rebuild_meshes(gl)
                );

                ui.label("X          Y          Z");
                vec3_opt_row(
                    ui, [&mut comp.x, &mut comp.y, &mut comp.z],
                    w3, &mut vars,
                    || part2.clone(),
                    |t| self3.add_history(editor::HistoryEntry::MeshPart(
                        light_mesh::LightMeshPartSnapshot {
                            idx: self3.get_current_mesh_idx().unwrap(),
                            name: self3.get_current_part_name().unwrap().to_string(),
                            part: Box::new(t)
                        }
                    )),
                    || self4.rebuild_meshes(gl)
                );

            }
        }

    }
}

fn draw_edit_right() {
    // TODO:
    // if vertices == 2:
    //     add compute vertex button
    // elif vertices >= 3:
    //     allow creating/removing triangles
    // if vertices > 1:
    //     multiplexed position input
    // if vertices make up triangles:
    //     multiplexed normal and uv inputs
}

fn draw_edit_gl(s: &UnsafeMutRef<App>, gl: &glow::Context, vp: &Mat4, eye: Vec3) {
    if let Some((_, name, _part)) = s.get_current_part()
        && let Some(sel) = s.editor.mesh
        && let Some(mesh) = s.view.meshes.get(sel)
        && let Some(mesh) = mesh.gpu_bufs.0.get(name)
    {
        let calls = vec![MeshDrawCall {
            mesh,
            instances: vec![InstanceData::new(Mat4::IDENTITY, 1., Some([0.2, 0.2, 0.2]))],
            wireframe: s.state.wireframe,
        }];

        s.render.renderer.draw_meshes(gl, vp, eye, &calls);

        let mut calls = Vec::new();
        if s.state.show_verts {
            calls.push(PointDrawCall {
                mesh,
                instances: vec![InstanceData::new(Mat4::IDENTITY, 1., Some([0., 1., 1.]))],
                size: 2.5,
            });
        }

        if let Some(selected) = s.render.sel_points.as_ref() {
            calls.push(PointDrawCall {
                mesh: selected,
                instances: vec![InstanceData::new(Mat4::IDENTITY, 1., Some([1., 1., 0.]))],
                size: 4.,
            });
        }

        if !calls.is_empty() {
            s.render.renderer.draw_points_batch(gl, vp, &calls);
        }
    }
}

pub fn main() -> Result<(), eframe::Error> {
    let path = std::env::args().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Beatcraft Mesh Editor")
            .with_inner_size([1440., 860.]),
        multisampling: 4,
        depth_buffer: 24,
        ..Default::default()
    };

    eframe::run_native(
        "Beatcraft Mesh Editor",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, path)))),
    )
}
