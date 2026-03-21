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

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::glow::{self, HasContext};
use egui::Frame;

use self::editor::App;

pub mod data;
pub mod easing;
pub mod light_mesh;
pub mod math_interp;
pub mod render;
pub mod editor;

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

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // Open Session...
                    if ui.button("Open Session\u{2026}  \u{2502}").clicked() {
                        // TODO: load with rfd + thread + channel
                    }
                    // Open...
                    if ui.button("Open\u{2026}          \u{2502}").clicked() {
                        // TODO: load with rfd + thread + channel
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // TODO: metadata panels
                });
            });

        egui::SidePanel::right("right_panel")
            .exact_width(300.)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // TODO: main panel
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
                            let mvp = s.ref_mut().cam().mvp(w, h);

                            gl.clear_color(0.07, 0.08, 0.11, 1.);
                            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                            gl.enable(glow::DEPTH_TEST);

                            if s.state.show_grid {
                                let flat = s.render.renderer.flat;
                                gl.use_program(Some(flat));
                                if let Some(l) = gl.get_uniform_location(flat, "uMVP") {
                                    gl.uniform_matrix_4_f32_slice(Some(&l), false, &mvp.to_cols_array());
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

                                }
                                editor::EditorMode::Assembly => {

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
