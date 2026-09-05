use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};

use clap::Parser;
use eframe::glow::{self, HasContext};
use egui::{Align2, Color32, Frame, ImageSource, Layout, Pos2, Sense, Ui};
use fluent_templates::LanguageIdentifier;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use self::config::{AppData, KeyMaps, LocaleCache, RawAppData, RecentProject};
use self::data::mesh::{
    BillboardData, MaterialType, NormalId, ShaderSettingsData, ShaderStyle, UvId, VertexId,
};
use self::editor::{
    App, CreateEnv, MINECRAFT_F, RoutineAction, SOURCE_CODE_F, Selection, SettingsPage,
    SettingsScreen, UiState, ViewStyle, WorkingRenameKey, setup_fonts,
};
use self::light_mesh::{BloomfogStyle, ComputeNormal, ComputeVertex, Part, Triangle};
use self::renaming::light_mesh::rehash;
use self::render::{
    GridType, HandleDrawCall, InstanceData, LIGHT_COLORS, MeshDrawCall, PointDrawCall,
};
use self::ui_elements::*;
use self::widgets::{MathDragValue, TextInput};
use bs_mapping_data::easing::Easing;
use fluent_templates::fluent_bundle::FluentValue;

pub mod audio;
pub mod beatmap;
pub mod config;
pub mod data;
pub mod editor;
pub mod light_mesh;
pub mod math_interp;
pub mod renaming;
pub mod render;
pub mod scenes;
pub mod ui_elements;
pub mod widgets;

// Logging targets
pub const DB_LOGIC: &str = "logic";
pub const DB_RENDER: &str = "render";
pub const DB_AUDIO: &str = "audio";
pub const DB_DATA: &str = "data";
pub const DB_HISTORY: &str = "history";
pub const DB_MATH: &str = "math";
pub const DB_MAIN: &str = "editor";

pub const SMALL_X: &str = "×";
pub const R_ARROW: &str = "▶";
pub const D_ARROW: &str = "▼";
pub const SMALL_R_ARROW: &str = "→";

pub static ENVIRONMENT_EDITOR_ICON: egui::ImageSource =
    egui::include_image!("assets/textures/environment_editor.png");
pub static SABER_EDITOR_ICON: egui::ImageSource =
    egui::include_image!("assets/textures/saber_editor.png");
pub static NOTE_EDITOR_ICON: egui::ImageSource =
    egui::include_image!("assets/textures/note_editor.png");
pub static BEATMAP_EDITOR_ICON: egui::ImageSource =
    egui::include_image!("assets/textures/beatmap_editor.png");

pub static MISSING_EDITOR_ICON: egui::ImageSource =
    egui::include_image!("assets/textures/svg/missing_editor.svg");

pub const APP_NAME: &str = "Beatcraft Mesh Editor";
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

const MODIFIER_NAMES: egui::ModifierNames<'_> = egui::ModifierNames {
    is_short: false,
    alt: "Alt",
    ctrl: "Ctrl",
    shift: "Shift",
    mac_cmd: "Cmd",
    mac_alt: "Alt",
    concat: "+",
};

pub fn get_data_folder() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("beatcraftmesheditor"))
}

pub fn get_data_file() -> Option<PathBuf> {
    get_data_folder().map(|dir| dir.join("data.json"))
}

#[derive(thiserror::Error, Debug)]
pub enum AppDataError {
    #[error("Missing Data Directory")]
    MissingDataDirectory,
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub fn load_app_data() -> Result<AppData, AppDataError> {
    let path = get_data_file().ok_or(AppDataError::MissingDataDirectory)?;
    if path.exists() {
        let json = fs::read_to_string(&path)?;
        let data: RawAppData = serde_json::from_str(&json)?;
        Ok(data.into())
    } else {
        Ok(Default::default())
    }
}

pub fn save_app_data(data: &AppData) -> Result<(), AppDataError> {
    let path = get_data_file().ok_or(AppDataError::MissingDataDirectory)?;
    let raw: RawAppData = data.into();
    let s = serde_json::to_string(&raw)?;
    let _ = fs::create_dir_all(path.parent().unwrap());
    fs::write(path, &s)?;
    Ok(())
}

#[derive(Copy, Clone)]
pub(crate) struct UnsafeMutRef<T: 'static> {
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let gl = frame.gl().unwrap();

        if self.state.dirty {
            self.rebuild_meshes(gl);
        }

        let (shift, ctrl, alt) = self.handle_keys(ctx, gl);

        let dt = ctx.input(|i| i.unstable_dt);
        if self.state.status_timer > 0. {
            if let Some(t) = self.state.title_content.as_mut()
                && !t.is_empty()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "{} {}",
                    self.title, t
                )));
                t.clear();
            }
            self.state.status_timer -= dt;
        } else if self.state.title_content.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title.to_string()));
            self.state.title_content = None;
        }

        self.update_routines(gl);

        self.audio_system.update();

        let rd = RefDuper;
        let self2 = unsafe { rd.detach_mut_ref(self) };
        if let Some((label, popup)) = self.state.ui.custom_popup.last()
            && let Some(resp) = egui::Window::new(label)
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .show(ctx, |ui| popup(self2, ui))
            && let Some(inner) = resp.inner
        {
            match inner {
                editor::PopupResponse::KeepOpen => {}
                editor::PopupResponse::Close => {
                    self.state.ui.custom_popup.pop();
                }
                editor::PopupResponse::OpenNew(x) => self.state.ui.custom_popup.push(x),
            }
        }

        if let Some(settings_page) = self.state.settings_screen.as_mut() {
            if !Self::draw_settings_page(
                &mut self.data.locale,
                ctx,
                &mut self.data.keymaps,
                settings_page,
            ) {
                self.state.settings_screen = None;
            }
        } else {
            match self.context {
                editor::EditorContext::Model(model_editor_context) => match model_editor_context {
                    editor::ModelEditorContext::Environment => {
                        self.draw_environment_editor(ctx, frame, shift, ctrl)
                    }
                    editor::ModelEditorContext::Saber => todo!(),
                    editor::ModelEditorContext::Notes => todo!(),
                },
                editor::EditorContext::Map(map_editor_context) => match map_editor_context {
                    editor::MapEditorContext::Beatmap => {
                        self.draw_beatmap_editor(ctx, frame, shift, ctrl, alt)
                    }
                    editor::MapEditorContext::Lightshow => todo!(),
                    editor::MapEditorContext::Audio => todo!(),
                },
                editor::EditorContext::None => self.draw_welcome_page(ctx, frame),
            }
        }

        ctx.request_repaint();
    }
}

impl App {
    pub fn reset_context(&mut self) {
        let gl = Arc::clone(&self.state.gl);

        self.click_cycle.reset();
        self.audio_system.remove_dead_audio();
        self.state.ui = UiState::default();
        self.map_editor.map = None;
        self.editor.hovered = None;
        self.editor.mesh = None;
        self.editor.part = None;
        self.view.fog_heights = None;
        self.view.spectrogram = None;
        self.view.session = None;
        self.view.mirror_path = None;
        self.view.mirror_id = None;
        self.view.mirror_geometry.clear();
        self.view.env_path = None;
        self.view.meshes.clear();
        self.assembly.hovered = None;
        self.assembly.handles.clear();
        self.selection = Selection::None;
        self.mode = editor::EditorMode::View;
        self.context = editor::EditorContext::None;
        self.history.clear();
        self.rebuild_meshes(&gl);
    }

    fn draw_settings_page(
        lang: &mut LocaleCache,
        ctx: &egui::Context,
        keymaps: &mut KeyMaps,
        settings: &mut SettingsScreen,
    ) -> bool {
        if !egui::SidePanel::new(egui::panel::Side::Left, "setting selector")
            .exact_width(300.)
            .show(ctx, |ui| {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(10.);
                        if ui
                            .add_sized([280., 30.], egui::Button::new(lang.get("back-button")))
                            .clicked()
                        {
                            return false;
                        }
                        ui.add_space(10.);
                        ui.separator();
                        ui.add_space(10.);
                        if ui
                            .add_sized(
                                [280., 30.],
                                egui::Button::new(lang.get("keybinds-label"))
                                    .selected(settings.page == SettingsPage::Keymaps),
                            )
                            .clicked()
                        {
                            settings.page = SettingsPage::Keymaps;
                            settings.cached_binds = Some(keymaps.clone());
                        }
                        if ui
                            .add_sized(
                                [280., 30.],
                                egui::Button::new(lang.get("language-label"))
                                    .selected(settings.page == SettingsPage::Language),
                            )
                            .clicked()
                        {
                            settings.page = SettingsPage::Language;
                        }
                        true
                    },
                )
                .inner
            })
            .inner
        {
            return false;
        }

        egui::CentralPanel::default().show(ctx, |ui| match settings.page {
            SettingsPage::Keymaps => {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(20.);
                        egui::ScrollArea::vertical()
                            .max_width(ui.available_width())
                            .max_height(ui.available_height() - 100.)
                            .show(ui, |ui| {
                                macro_rules! key_label {
                                    ( $name:literal ) => {
                                        ui.add_sized(
                                            [ui.available_width(), 30.],
                                            egui::Label::new(
                                                egui::RichText::new(lang.get($name))
                                                    .strong()
                                                    .size(22.),
                                            ),
                                        );
                                    };
                                }

                                macro_rules! key_row {
                                    ( $attr:tt, $id:literal ) => {
                                        ui.allocate_ui_with_layout(
                                            [ui.available_width(), 20.].into(),
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                ui.allocate_ui_with_layout(
                                                    [ui.available_width() / 2. - 10., 15.].into(),
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.add(egui::Label::new(lang.get($id)));
                                                    },
                                                );
                                                ui.allocate_ui_with_layout(
                                                    [ui.available_width() / 2. - 10., 15.].into(),
                                                    egui::Layout::left_to_right(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.add(egui_keybind::Keybind::new(
                                                            &mut keymaps.$attr,
                                                            $id,
                                                        ))
                                                    },
                                                );
                                            },
                                        );
                                    };
                                }

                                key_label!("keygroup-general");
                                key_row!(toggle_wireframe, "key-toggle-wireframe");
                                key_row!(toggle_grid, "key-toggle-grid");
                                key_row!(toggle_render_style, "key-toggle-render-style");
                                key_row!(deselect, "key-deselect");
                                key_row!(save, "key-save");
                                key_row!(undo, "key-undo");
                                key_row!(redo, "key-redo");

                                ui.add_space(20.);

                                key_label!("keygroup-mesh");
                                key_row!(toggle_vertices, "key-toggle-vertices");
                                key_row!(toggle_mesh_part_back, "key-toggle-mesh-part-back");
                                key_row!(toggle_mesh_part_forward, "key-toggle-mesh-part-forward");
                                key_row!(toggle_edit_component, "key-toggle-edit-component");
                                key_row!(toggle_assembly_view, "key-toggle-assembly-view");
                                key_row!(create_or_remove_triangles, "key-toggle-triangles");
                                key_row!(flip_triangles, "key-flip-triangles");
                                key_row!(create_vertex, "key-create-vertex");

                                ui.add_space(20.);

                                key_label!("keygroup-beatmap");
                                key_row!(toggle_map_playback, "key-toggle-map-playback");
                                key_row!(rotate_map_grid_left, "key-rotate-grid-left");
                                key_row!(rotate_map_grid_right, "key-rotate-grid-right");
                                key_row!(map_fly_forward, "key-fly-forward");
                                key_row!(map_fly_backward, "key-fly-backward");
                                key_row!(map_fly_left, "key-fly-left");
                                key_row!(map_fly_right, "key-fly-right");
                                key_row!(map_fly_up, "key-fly-up");
                                key_row!(map_fly_down, "key-fly-down");

                                ui.add_space(20.);

                                key_label!("keygroup-debug");
                                key_row!(rebuild_meshes, "key-rebuild-meshes");
                            });

                        ui.separator();
                        ui.add_space(20.);

                        if ui
                            .add_sized(
                                [200., 20.],
                                egui::Button::new(lang.get("revert-keys-label")),
                            )
                            .clicked()
                            && let Some(cached) = settings.cached_binds.clone()
                        {
                            *keymaps = cached;
                        }
                        ui.add_space(10.);
                        if ui
                            .add_sized([200., 20.], egui::Button::new(lang.get("reset-keys-label")))
                            .clicked()
                        {
                            *keymaps = KeyMaps::default();
                        }
                    },
                );
            }
            SettingsPage::Language => {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(20.);
                        ui.add_sized(
                            [200., 30.],
                            egui::Label::new(egui::RichText::new(lang.get("select-language"))),
                        );

                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 60.)
                            .max_width(ui.available_width())
                            .show(ui, |ui| {
                                if ui
                                    .add_sized(
                                        [300., 50.],
                                        egui::Button::new("English").selected(lang.locale.matches(
                                            &"en-US".parse::<LanguageIdentifier>().unwrap(),
                                            false,
                                            false,
                                        )),
                                    )
                                    .clicked()
                                {
                                    lang.locale = "en-US".parse().unwrap();
                                }
                            });

                        ui.add_space(ui.available_height() - 60.);
                        ui.label(
                            lang.get_with_args(
                                "language-help",
                                &[
                                    ("url".into(), REPOSITORY.into()),
                                    ("discord".into(), "@westbot".into()),
                                ]
                                .into(),
                            ),
                        );
                    },
                );
            }
        });
        true
    }

    fn draw_welcome_page(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("WelcomeTopBar").show(ctx, |ui| {
            ui.add_space(2.);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.data.locale.get("settings-label").to_string(), |ui| {
                    if ui.button(self.data.locale.get("settings-button")).clicked() {
                        self.open_settings();
                    }
                    let mut minecraft_font: bool = ui.memory_mut(|m| {
                        m.data
                            .get_persisted("use_minecraft_font".into())
                            .unwrap_or(true)
                    });
                    let old = minecraft_font;
                    ui.checkbox(
                        &mut minecraft_font,
                        self.data.locale.get("minecraft-font-label"),
                    );
                    if old != minecraft_font {
                        if minecraft_font {
                            setup_fonts(&[MINECRAFT_F, SOURCE_CODE_F], ctx);
                        } else {
                            setup_fonts(&[SOURCE_CODE_F, MINECRAFT_F], ctx);
                        }
                        ui.memory_mut(|m| {
                            m.data
                                .insert_persisted("use_minecraft_font".into(), minecraft_font)
                        });
                    }
                });
            });
        });

        // egui::TopBottomPanel::bottom("WelcomeBottomPanel")
        //     .frame(Frame::NONE)
        //     .show(ctx, |ui| {
        //
        //     });

        egui::SidePanel::left("WelcomeLeftPanel")
            .resizable(false)
            .exact_width(300.)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.);
                    ui.add(egui::Label::new(egui::RichText::new(self.data.locale.get("recent-projects")).size(20.).strong()));

                    let mut to_open = None;
                    for RecentProject { modified, path, kind } in self.data.recents.iter() {
                        #[cfg(not(feature = "mesh-editor"))]
                        if matches!(kind.kind(), ProjectKind::EnvironmentMesh | ProjectKind::NoteMesh | ProjectKind::SaberMesh) { continue }
                        #[cfg(not(feature = "mapper"))]
                        if matches!(kind.kind(), ProjectKind::Beatmap | ProjectKind::Lightshow) { continue }
                        let ext = path.with_extension("");
                        let Some(label) = ext.file_name() else { continue };
                        let label = label.to_string_lossy();
                        let full_path = path.to_string_lossy();
                        ui.add_space(8.);
                        ui.allocate_ui_with_layout(
                            [280., 80.].into(),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.horizontal(|ui| {
                                    let total_width = 280.0;
                                    let button_width = 50.0;
                                    ui.allocate_ui_with_layout(
                                        [total_width, 20.].into(),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            egui::ScrollArea::horizontal()
                                                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                                                .id_salt(("label_scroll", path))
                                                .max_width(total_width - button_width)
                                                .show(ui, |ui| {
                                                    ui.label(egui::RichText::new(label).strong())
                                                        .on_hover_text(full_path);
                                                });

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button(self.data.locale.get("open")).clicked() {
                                                    to_open = Some((kind, path.to_path_buf()));
                                                }
                                            });
                                        },
                                    );
                                });
                                ui.label(egui::RichText::new(kind.kind().to_string()).weak());
                                ui.label(
                                    egui::RichText::new(
                                        self.data.locale.get_with_args(
                                            "last-opened",
                                            &[(
                                                Cow::Borrowed("modified"),
                                                FluentValue::String(
                                                    modified.to_string().into()
                                                )
                                            )].into()
                                        )
                                    ).small()
                                );
                            }
                        );

                    }
                    if let Some((kind, path)) = to_open {
                        #[allow(unreachable_patterns)]
                        match kind {
                            #[cfg(feature = "mesh-editor")]
                            config::ProjectType::EnvironmentMesh => {
                                self.reset_context();
                                let gl = Arc::clone(&self.state.gl);
                                let _ = self.load_session(&path, &gl);
                                self.context = editor::EditorContext::Model(editor::ModelEditorContext::Environment);
                            }
                            #[cfg(feature = "mesh-editor")]
                            config::ProjectType::SaberMesh => todo!(),
                            #[cfg(feature = "mesh-editor")]
                            config::ProjectType::NoteMesh => todo!(),
                            #[cfg(feature = "mapper")]
                            config::ProjectType::Beatmap { .. } => {
                                self.reset_context();
                                let gl = Arc::clone(&self.state.gl);
                                let rd = RefDuper;
                                let s = unsafe { rd.detach_mut_ref(self) };
                                let _ = s.load_beatmap(&mut self.audio_system, path, &gl, &mut self.render.renderer, self.data.audio_volume);
                                self.context = editor::EditorContext::Map(editor::MapEditorContext::Beatmap);
                            },
                            #[cfg(feature = "mapper")]
                            config::ProjectType::Lightshow => todo!(),
                            _ => {}
                        }
                    }

                })
            });

        // egui::SidePanel::right("WelcomeRightPanel")
        //     .frame(Frame::NONE)
        //     .resizable(false)
        //     .exact_width(300.)
        //     .show(ctx, |ui| {
        //
        //     });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    let height = ui.available_height() / 2.;
                    let val = match (cfg!(feature = "mapper"), cfg!(feature = "mesh-editor")) {
                        (true, true) => 206.25,
                        (true, false) | (false, true) => 100.,
                        (false, false) => 0.,
                    };
                    ui.add_space(height - val);

                    #[cfg(feature = "mesh-editor")]
                    ui.allocate_ui_with_layout(
                        [610., 200.].into(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let ee: Vec<String> = self
                                .data
                                .locale
                                .get("environment-editor")
                                .split("\n")
                                .map(ToString::to_string)
                                .collect();
                            self.draw_context_selector(
                                ctx,
                                ui,
                                "env_edit_scale".into(),
                                ENVIRONMENT_EDITOR_ICON.clone(),
                                &ee,
                                |s| {
                                    s.context = editor::EditorContext::Model(
                                        editor::ModelEditorContext::Environment,
                                    )
                                },
                            );

                            let se: Vec<String> = self
                                .data
                                .locale
                                .get("saber-editor")
                                .split("\n")
                                .map(ToString::to_string)
                                .collect();
                            self.draw_context_selector(
                                ctx,
                                ui,
                                "saber_edit_scale".into(),
                                SABER_EDITOR_ICON.clone(),
                                &se,
                                |_s| (), //s.context = editor::EditorContext::Model(editor::ModelEditorContext::Saber),
                            );

                            let ne: Vec<String> = self
                                .data
                                .locale
                                .get("note-editor")
                                .split("\n")
                                .map(ToString::to_string)
                                .collect();
                            self.draw_context_selector(
                                ctx,
                                ui,
                                "note_edit_scale".into(),
                                NOTE_EDITOR_ICON.clone(),
                                &ne,
                                |_s| (), //s.context = editor::EditorContext::Model(editor::ModelEditorContext::Notes),
                            );
                        },
                    );

                    #[cfg(all(feature = "mapper", feature = "mesh-editor"))]
                    ui.add_space(12.5);

                    #[cfg(feature = "mapper")]
                    ui.allocate_ui_with_layout(
                        [405., 200.].into(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let ee: Vec<String> = self
                                .data
                                .locale
                                .get("beatmap-editor")
                                .split("\n")
                                .map(ToString::to_string)
                                .collect();
                            self.draw_context_selector(
                                ctx,
                                ui,
                                "beatmap_edit_scale".into(),
                                BEATMAP_EDITOR_ICON.clone(),
                                &ee,
                                |s| {
                                    s.context = editor::EditorContext::Map(
                                        editor::MapEditorContext::Beatmap,
                                    )
                                },
                            );

                            let ee: Vec<String> = self
                                .data
                                .locale
                                .get("lightshow-editor")
                                .split("\n")
                                .map(ToString::to_string)
                                .collect();
                            self.draw_context_selector(
                                ctx,
                                ui,
                                "lightshow_edit_scale".into(),
                                MISSING_EDITOR_ICON.clone(),
                                &ee,
                                |_s| (), //s.context = editor::EditorContext::Map(editor::MapEditorContext::Lightshow),
                            );
                        },
                    );

                    ui.ctx().request_repaint();
                },
            );
        });
    }

    pub fn open_settings(&mut self) {
        self.state.settings_screen = Some(SettingsScreen {
            page: SettingsPage::Keymaps,
            cached_binds: Some(self.data.keymaps.clone()),
        });
    }

    fn draw_context_selector(
        &mut self,
        ctx: &egui::Context,
        ui: &mut Ui,
        scale_id: egui::Id,
        img_source: ImageSource,
        overlay_text: &[String],
        on_click: impl Fn(&mut Self),
    ) {
        let scale = ui
            .memory(|m| m.data.get_temp::<f32>(scale_id))
            .unwrap_or(1.);

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(200., 200.),
            egui::Sense::hover() | egui::Sense::click(),
        );

        if response.clicked() {
            self.reset_context();
            on_click(self);
        }

        let center = rect.center().to_vec2();
        let translation = center * (1.0 - scale);

        let visuals = *ui.style().interact(&response);
        let is_hovered = response.hovered();

        ui.with_visual_transform(egui::emath::TSTransform::new(translation, scale), |ui| {
            let painter = ui.painter();

            painter.rect(
                rect.expand(visuals.expansion * 2.),
                6.0,
                Color32::TRANSPARENT,
                egui::Stroke::new(
                    1.,
                    if is_hovered {
                        Color32::from_white_alpha(127)
                    } else {
                        visuals.bg_fill
                    },
                ),
                egui::StrokeKind::Outside,
            );

            egui::Image::new(img_source)
                .corner_radius(6)
                .paint_at(ui, rect);

            let mut y = 8.;
            for line in overlay_text.iter().rev() {
                painter.text(
                    rect.center_bottom() - egui::vec2(0.0, y),
                    egui::Align2::CENTER_BOTTOM,
                    line,
                    egui::FontId::proportional(16.0),
                    Color32::WHITE,
                );
                y += 20.;
            }
        });

        let dt = ctx.input(|i| i.stable_dt);
        ui.memory_mut(|m| {
            if let Some(mut t) = m.data.get_temp::<f32>(scale_id) {
                t = if is_hovered {
                    1.05f32.min(t + 0.45 * dt)
                } else {
                    1.0f32.max(t - 0.45 * dt)
                };
                m.data.insert_temp(scale_id, t);
            } else {
                m.data.insert_temp(scale_id, 1f32);
            }
        });
    }

    fn pad_menu_text(text: &mut [(String, Option<String>)]) {
        let Some(max_width) = text.iter().map(|(t, _)| t.len()).max() else {
            tracing::warn!(target: DB_MAIN, "pad_menu_text was called without any input text");
            return;
        };
        for (text, after) in text.iter_mut() {
            *text = match after {
                Some(keys) => format!("{text:<max_width$} \u{2502} [{keys}]"),
                None => format!("{text:<max_width$} \u{2502}"),
            };
        }
    }

    fn draw_environment_editor(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        shift: bool,
        ctrl: bool,
    ) {
        let gl = frame.gl().unwrap();

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.add_space(2.);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.data.locale.get("file-menu-label").to_string(), |ui| {
                    let mut options = [
                        (self.data.locale.get("create-new-menu-label").to_string(), None),
                        (self.data.locale.get("open-environment-menu-label").to_string(), None),
                        (self.data.locale.get("open-menu-label").to_string(), None),
                        (
                            self.data.locale.get("save-menu-label").to_string(),
                            Some(self.data.keymaps.save.format(&MODIFIER_NAMES, cfg!(target_os = "macos")))
                        ),
                        (self.data.locale.get("menu-label").to_string(), None),
                    ];
                    Self::pad_menu_text(&mut options);
                    let [
                        (create_new, _),
                        (open_env, _),
                        (open_mesh, _),
                        (save, _),
                        (menu, _),
                    ] = options;
                    if ui.button(create_new).clicked() && !self.block_input()
                    {
                        tracing::debug!(target: DB_LOGIC, "Spawning thread for environment creation");
                        let (sx, rx) = mpsc::channel();
                        let title = self.data.locale.get("create-env-file").to_string();
                        let title2 = self.data.locale.get("create-editor-data-file").to_string();
                        std::thread::spawn(move || {
                            let Some(env_path) = rfd::FileDialog::new()
                                .set_title(title)
                                .set_file_name("env")
                                .add_filter("json", &["json"])
                                .save_file() else {
                                    tracing::debug!(target: DB_LOGIC, "Canceled environment creation");
                                    return; // Cancel
                                };
                            let Some(session_path) = rfd::FileDialog::new()
                                .set_title(title2)
                                .set_file_name("my_env")
                                .add_filter("json", &["json"])
                                .save_file() else {
                                    tracing::debug!(target: DB_LOGIC, "Canceled environment creation");
                                    return; // Cancel
                                };

                            tracing::debug!(target: DB_LOGIC, ?env_path, ?session_path, "Sending CreateEnv event");
                            let _ = sx.send(editor::CreateEnv { env_path, session_path });
                        });
                        self.add_routine(Box::new(move |s, _| {
                            match rx.try_recv() {
                                Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                                Err(mpsc::TryRecvError::Disconnected) => RoutineAction::Remove,
                                Ok(CreateEnv { env_path, session_path }) => {
                                    s.view.env_path = Some(env_path);
                                    s.view.session = Some(session_path);
                                    if let Err(e) = s.save_session() {
                                        let st = s.data.locale.get("failed-to-save-environment").to_string();
                                        s.set_status(None, st, 2.);
                                        eprintln!("Failed to save environment/session: {e}");
                                    } else {
                                        let st = s.data.locale.get("created-environment").to_string();
                                        s.set_status(None, st, 2.);
                                    }
                                    RoutineAction::Remove
                                }
                            }
                        }));
                    }
                    if ui.button(open_env).clicked() && !self.block_input()
                    {
                        tracing::debug!(target: DB_LOGIC, "Spawning thread for opening environment");
                        let (sx, rx) = mpsc::channel();
                        let title = self.data.locale.get("open-environment").to_string();
                        std::thread::spawn(move || {
                            if let Some(session) = rfd::FileDialog::new()
                                .set_title(title)
                                .add_filter("json", &["json"])
                                .pick_file()
                            {
                                tracing::debug!(target: DB_LOGIC, ?session, "Sending environment open event");
                                let _ = sx.send(session);
                            }
                        });
                        self.add_routine(Box::new(move |s, gl| {
                            match rx.try_recv() {
                                Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                                Err(mpsc::TryRecvError::Disconnected) => RoutineAction::Remove,
                                Ok(session) => {
                                    if let Err(e) = s.load_session(&session, gl) {
                                        eprintln!("Error loading session: {e}")
                                    }
                                    RoutineAction::Remove
                                }
                            }
                        }));
                    }
                    if ui.button(open_mesh).clicked() && !self.block_input()
                    {
                        tracing::debug!(target: DB_LOGIC, "Spawning thread for opening meshes");
                        let (sx, rx) = mpsc::channel();
                        let title = self.data.locale.get("open-meshes").to_string();
                        std::thread::spawn(move || {
                            if let Some(meshes) = rfd::FileDialog::new()
                                .set_title(title)
                                .add_filter("json", &["json"])
                                .pick_files()
                            {
                                tracing::debug!(target: DB_LOGIC, ?meshes, "Sending mesh open event");
                                let _ = sx.send(meshes);
                            }
                        });
                        self.add_routine(Box::new(move |s, gl| {
                            match rx.try_recv() {
                                Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                                Err(mpsc::TryRecvError::Disconnected) => RoutineAction::Remove,
                                Ok(meshes) => {
                                    for m in meshes.into_iter() {
                                        if let Err(e) = s.load_meshes({ let mut m2 = IndexMap::new(); m2.insert(s.get_unique_mesh_id(), m); m2 }, gl) {
                                            eprintln!("Error loading meshes: {e}");
                                        }
                                    }
                                    RoutineAction::Remove
                                }
                            }
                        }));
                    }
                    if ui.button(save).clicked() {
                        match self.mode {
                            editor::EditorMode::View => {}
                            editor::EditorMode::Assembly | editor::EditorMode::Edit => {}
                        }
                        let _ = self.save_session();
                        ui.close();
                    }
                    if ui.button(menu).clicked() {
                        tracing::debug!(target: DB_LOGIC, "Closing current environment and returning to menu");
                        self.context = editor::EditorContext::None;
                        let gl = Arc::clone(&self.state.gl);
                        close_environment(self, &gl);
                    }
                });
                ui.menu_button(self.data.locale.get("edit").to_string(), |ui| {
                    let mut options = [
                        (
                            self.data.locale.get("undo").to_string(),
                            Some(self.data.keymaps.undo.format(&MODIFIER_NAMES, cfg!(target_os = "macos")))
                        ),
                        (
                            self.data.locale.get("redo").to_string(),
                            Some(self.data.keymaps.redo.format(&MODIFIER_NAMES, cfg!(target_os = "macos")))
                        )
                    ];
                    Self::pad_menu_text(&mut options);
                    let [
                        (undo, _),
                        (redo, _)
                    ] = options;
                    if ui.button(undo).clicked() {
                        self.undo(gl);
                        ui.close();
                    }
                    if ui.button(redo).clicked() {
                        self.redo(gl);
                        ui.close();
                    }
                });
                ui.menu_button(self.data.locale.get("view-menu-label").to_string(), |ui| {
                    let mut options = [
                        (
                            self.data.locale.get("wireframe").to_string(),
                            Some(self.data.keymaps.toggle_wireframe.format(&MODIFIER_NAMES, cfg!(target_os = "macos")))
                        ),
                        (
                            self.data.locale.get("show-grid").to_string(),
                            Some(self.data.keymaps.toggle_grid.format(&MODIFIER_NAMES, cfg!(target_os = "macos")))
                        ),
                        (
                            self.data.locale.get("show-vertices").to_string(),
                            Some(self.data.keymaps.toggle_vertices.format(&MODIFIER_NAMES, cfg!(target_os = "macos")))
                        ),
                    ];
                    Self::pad_menu_text(&mut options);
                    let [
                        (wireframe, _),
                        (show_grid, _),
                        (vertices, _),
                    ] = options;

                    ui.checkbox(&mut self.state.wireframe,  wireframe);
                    ui.checkbox(&mut self.state.show_grid,  show_grid);
                    ui.checkbox(&mut self.state.show_verts, vertices);
                });
                ui.menu_button(self.data.locale.get("settings-label").to_string(), |ui| {
                    if ui.button(self.data.locale.get("settings-button")).clicked() {
                        self.open_settings();
                    }
                    let mut minecraft_font: bool = ui.memory_mut(|m| m.data.get_persisted("use_minecraft_font".into()).unwrap_or(true));
                    let old = minecraft_font;
                    ui.checkbox(&mut minecraft_font, self.data.locale.get("minecraft-font-label"));
                    if old != minecraft_font {
                        if minecraft_font {
                            tracing::debug!(target: DB_LOGIC, "Enabling Minecraft font");
                            setup_fonts(&[MINECRAFT_F, SOURCE_CODE_F], ctx);
                        } else {
                            tracing::debug!(target: DB_LOGIC, "Enabling Source Code font");
                            setup_fonts(&[SOURCE_CODE_F, MINECRAFT_F], ctx);
                        }
                        ui.memory_mut(|m| m.data.insert_persisted("use_minecraft_font".into(), minecraft_font));
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

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |_ui| {
            // ui.horizontal(|ui| {
            //     let display = match self.mode {
            //         editor::EditorMode::View =>
            //             " View       | [Ctrl+S] Save session",
            //         editor::EditorMode::Assembly =>
            //             " Assembly   | [Ctrl+S] Save mesh | [E]dit parts | [I] View ",
            //         editor::EditorMode::Edit =>
            //             " Edit part  | [Ctrl+S] Save mesh | [E] Assembly | [I] View | [C]reate vertex | [N] Add/Remove tris | [R]ewind triangles",
            //     };
            //     ui.label(display);
            //     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            //         ui.label("[F] Vfx | [W]ireframe | [G]rid | [V]ertices \u{0}")
            //     });
            // });
        });

        egui::SidePanel::left("left_panel")
            .exact_width(250.)
            .resizable(false)
            .show(ctx, |ui| {
                ui.allocate_ui(ui.available_size(), |ui| {
                    let h = if self.mode == editor::EditorMode::Edit {
                        75.
                    } else {
                        45.
                    };
                    ui.allocate_exact_size((230., 1.).into(), Sense::empty());
                    ui.allocate_ui(
                        (ui.available_width(), ui.available_height() - h).into(),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("left_p_scroll")
                                .show(ui, |ui| match self.mode {
                                    editor::EditorMode::View => {
                                        scenes::view::draw_view_left(self, ui, gl);
                                    }
                                    editor::EditorMode::Assembly => {
                                        scenes::assembly::draw_assembly_left(self, ui, gl);
                                    }
                                    editor::EditorMode::Edit => {
                                        scenes::edit::draw_edit_left(self, ui, gl);
                                    }
                                });
                        },
                    );

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
                                    MathDragValue::new(&mut target.x, &mut vars).speed(0.01),
                                );
                                ui.add_sized(
                                    [width, 20.],
                                    MathDragValue::new(&mut target.y, &mut vars).speed(0.01),
                                );
                                ui.add_sized(
                                    [width, 20.],
                                    MathDragValue::new(&mut target.z, &mut vars).speed(0.01),
                                );
                            });
                            ui.label(self.data.locale.get("camera-pivot"));
                            if h > 45. {
                                ui.add_space(5.);
                                ui.allocate_ui_with_layout(
                                    [ui.available_width(), 20.].into(),
                                    egui::Layout::bottom_up(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add_sized(
                                                ui.available_size(),
                                                egui::Button::new(
                                                    self.data.locale.get("show-uv-editor"),
                                                )
                                                .selected(self.state.ui.show_uv_window),
                                            )
                                            .clicked()
                                        {
                                            self.state.ui.show_uv_window =
                                                !self.state.ui.show_uv_window;
                                        }
                                    },
                                );
                            }
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
                        scenes::view::draw_view_right(self, ui, gl);
                    }
                    editor::EditorMode::Assembly => {
                        scenes::assembly::draw_assembly_right(self, ui, gl);
                    }
                    editor::EditorMode::Edit => {
                        scenes::edit::draw_edit_right(self, ui, gl);
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
                                let view = s.ref_mut().cam().view_mat();
                                let proj = s.ref_mut().cam().proj_mat(w, h);

                                match s.state.view_style {
                                    editor::ViewStyle::Beatcraft { blackout_sky: true } => {
                                        gl.clear_color(0., 0., 0., 1.);
                                    }
                                    _ => {
                                        gl.clear_color(0.07, 0.08, 0.11, 1.);
                                        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                                    }
                                }

                                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                                gl.enable(glow::DEPTH_TEST);
                                gl.depth_mask(true);

                                match s.mode {
                                    editor::EditorMode::View => {
                                        scenes::view::draw_view_gl(
                                            &s,
                                            gl,
                                            &view,
                                            &proj,
                                            (w as i32, h as i32),
                                        );
                                    }
                                    editor::EditorMode::Assembly => {
                                        scenes::assembly::draw_assembly_gl(
                                            &s,
                                            gl,
                                            &view,
                                            &proj,
                                            (w as i32, h as i32),
                                        );
                                    }
                                    editor::EditorMode::Edit => {
                                        scenes::edit::draw_edit_gl(
                                            &s,
                                            gl,
                                            &view,
                                            &proj,
                                            (w as i32, h as i32),
                                        );
                                    }
                                }

                                if s.state.show_grid && s.state.view_style == ViewStyle::Edit {
                                    s.render.renderer.draw_grid(gl, &view, &proj);
                                }
                            }
                        },
                    )),
                });
            });

        if self.mode == editor::EditorMode::Edit && self.state.ui.show_uv_window {
            egui::Window::new(self.data.locale.get("uv-editor"))
                .id(egui::Id::new("uv_editor"))
                .min_size([200., 200.])
                .pivot(Align2::CENTER_CENTER)
                .default_pos([800., 800.])
                .show(ctx, |ui| {
                    scenes::uv::draw_uv_view(self, ui, ctx, gl);
                });
        }

        if self.mode == editor::EditorMode::View && self.state.ui.show_mirror_window {
            egui::Window::new(self.data.locale.get("mirror-geometry-editor"))
                .id(egui::Id::new("mirror_editor"))
                .min_size([200., 200.])
                .pivot(Align2::CENTER_CENTER)
                .default_pos([200., 800.])
                .show(ctx, |ui| {
                    scenes::mirror::draw_mirror_view(self, ui, ctx, gl, shift, ctrl);
                });
        }
    }
}

pub fn close_environment(s: &mut App, gl: &glow::Context) {
    let meshes = std::mem::take(&mut s.view.meshes);
    for (_, vm) in meshes {
        if let Some(vm) = vm {
            vm.destroy(gl);
        }
    }
    s.view.session = None;
    s.view.env_path = None;
    s.view.mirror_id = None;
    s.view.mirror_path = None;
    s.view.spectrogram = None;
    s.view.fog_heights = None;
    if let Some(vm) = s.render.mirror.take() {
        vm.destroy(gl);
    }
    if let Some(towers) = s.render.spectrogram.take() {
        towers.destroy(gl);
    }
}

const UV_VERT_COLORS: [egui::Color32; 3] = [
    egui::Color32::from_rgb(220, 80, 80),
    egui::Color32::from_rgb(80, 200, 120),
    egui::Color32::from_rgb(80, 150, 220),
];

const UV_HIT_RADIUS: f32 = 6.0;

fn uv_to_screen(uv: glam::Vec2, rect: egui::Rect, pan: glam::Vec2, zoom: f32) -> egui::Pos2 {
    let origin = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.5);
    let centered = (uv - glam::Vec2::splat(0.5) - pan) * zoom;
    origin + egui::vec2(centered.x, centered.y)
}

fn screen_to_uv(pos: egui::Pos2, rect: egui::Rect, pan: glam::Vec2, zoom: f32) -> glam::Vec2 {
    let origin = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.5);
    let delta = pos - origin;
    glam::Vec2::new(delta.x, delta.y) / zoom + pan + glam::Vec2::splat(0.5)
}

fn snap_uv(uv: glam::Vec2, tex_w: u32, tex_h: u32, modifiers: &egui::Modifiers) -> glam::Vec2 {
    let divisor = match (modifiers.ctrl, modifiers.shift) {
        (true, true) => 8.0,
        (true, false) => 2.0,
        (false, true) => 4.0,
        (false, false) => 1.0,
    };
    let step_x = 1.0 / (tex_w as f32 * divisor);
    let step_y = 1.0 / (tex_h as f32 * divisor);
    glam::Vec2::new(
        (uv.x / step_x).round() * step_x,
        (uv.y / step_y).round() * step_y,
    )
}

fn get_or_load_texture<'a>(
    display_id: &str,
    path: &Path,
    ctx: &egui::Context,
    cache: &'a mut HashMap<String, egui::TextureHandle>,
) -> Option<&'a egui::TextureHandle> {
    if !cache.contains_key(display_id) {
        let image = image::open(path).ok()?;
        let image = image.to_rgba8();
        let (w, h) = image.dimensions();
        let pixels = image.into_raw();
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
        let handle = ctx.load_texture(display_id, color_image, egui::TextureOptions::NEAREST);
        cache.insert(display_id.to_string(), handle);
    }
    cache.get(display_id)
}

#[derive(clap::Parser)]
#[command(name = "Beatcraft Editor", version)]
/// The Beatcraft general mesh editor and Beat Saber mapping tool
///
/// This tool is for editing all Beatcraft mesh types.
/// It additionally has tools for general Beat Saber
/// mapping.
/// Will eventually also have support for Noodle and Chroma
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long)]
    debug: bool,
    #[arg(name = "log-file")]
    log_file: Option<PathBuf>,
}

fn build_env_filter(debug: bool) -> EnvFilter {
    let mut filter = if debug {
        EnvFilter::new("off")
            .add_directive("egui_glow=warn".parse().unwrap())
            .add_directive("egui=warn".parse().unwrap())
            .add_directive("beatcraft_editor=debug".parse().unwrap())
            .add_directive(format!("{DB_LOGIC}=debug").parse().unwrap())
            .add_directive(format!("{DB_MAIN}=debug").parse().unwrap())
            .add_directive(format!("{DB_MATH}=debug").parse().unwrap())
            .add_directive(format!("{DB_DATA}=debug").parse().unwrap())
            .add_directive(format!("{DB_RENDER}=debug").parse().unwrap())
            .add_directive(format!("{DB_AUDIO}=debug").parse().unwrap())
            .add_directive(format!("{DB_HISTORY}=debug").parse().unwrap())
    } else {
        EnvFilter::new("info")
    };

    if let Ok(env_value) = std::env::var("BEATCRAFT_LOG") {
        for directive in env_value.split(',').filter(|s| !s.trim().is_empty()) {
            match directive.parse() {
                Ok(d) => filter = filter.add_directive(d),
                Err(e) => eprintln!("Ignoring invalid BEATCRAFT_LOG directive '{directive}': {e}"),
            }
        }
    }

    filter
}

pub fn init_logger(debug: bool, log_file: Option<PathBuf>) -> Option<WorkerGuard> {
    let stdout_layer = fmt::layer().with_filter(build_env_filter(debug));

    let (file_layer, guard) = match log_file {
        Some(path) => match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            Ok(file) => {
                let (non_blocking, guard) = tracing_appender::non_blocking(file);
                let layer = fmt::layer()
                    .with_ansi(false) // no color codes in the file
                    .with_writer(non_blocking)
                    .with_filter(build_env_filter(debug));
                (Some(layer), Some(guard))
            }
            Err(e) => {
                eprintln!("Failed to open log file {}: {e}", path.display());
                (None, None)
            }
        },
        None => (None, None),
    };

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    if debug {
        tracing::debug!(target: DB_MAIN, "Enabled debug logging");
    }

    guard
}

pub fn main() -> Result<(), eframe::Error> {
    let Cli {
        path,
        debug,
        log_file,
    } = Cli::parse();

    let _guard = init_logger(debug, log_file);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size([1440., 860.]),
        multisampling: 4,
        depth_buffer: 24,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, path)))),
    )
}
