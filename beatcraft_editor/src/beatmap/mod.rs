use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use eframe::glow::{self, Context, HasContext};
use egui::Response;
use glam::Mat4;

use crate::audio::Audio;
use crate::data::LightMeshData;
use crate::light_mesh::LightMesh;
use crate::render::{GpuMesh, Renderer};
use crate::{DB_DATA, DB_LOGIC, DB_MAIN, RefDuper, UnsafeMutRef, editor, get_data_folder};
use crate::editor::{App, EditorContext, RoutineAction, ViewMesh, ViewStyle};

use self::data::v2::{CharacteristicSetV2, DifficultyBeatmapV2};
use self::data::{InfoFile, MapCharacteristic, MapDifficulty};

pub mod event;
pub mod object;
pub mod data;
pub mod render;
#[cfg(test)]
pub mod tests;

pub struct BeatmapProjectDiff {
    pub difficulty: MapDifficulty,
    pub rank: u8,
    pub beatmap_file: Option<PathBuf>,
    pub njs: f32,
    pub njs_offset: f32,
    pub custom_data: Option<serde_json::Value>,
}

pub struct BeatmapProjectSet {
    pub set: MapCharacteristic,
    pub diffs: Vec<BeatmapProjectDiff>,
}

impl From<&CharacteristicSetV2> for BeatmapProjectSet {
    fn from(value: &CharacteristicSetV2) -> Self {
        Self {
            set: value.beatmap_characteristic_name.clone(),
            diffs: value.difficulty_beatmaps.iter().map(Into::into).collect(),
        }
    }
}

impl From<&DifficultyBeatmapV2> for BeatmapProjectDiff {
    fn from(value: &DifficultyBeatmapV2) -> Self {
        Self {
            difficulty: value.difficulty.clone(),
            rank: value.difficulty_rank,
            beatmap_file: Some(PathBuf::from(&value.beatmap_filename)),
            njs: value.note_jump_movement_speed,
            njs_offset: value.note_jump_start_beat_offset,
            custom_data: value.custom_data.clone(),
        }
    }
}

pub struct BeatmapProject {
    pub folder: PathBuf,
    pub info_path: Option<PathBuf>,
    pub audio: Option<std::sync::Arc<Audio>>,
    pub cover_image: Option<PathBuf>,
    pub sets: Vec<BeatmapProjectSet>
}

pub struct BeatmapMeshSet {
    pub note_mesh: GpuMesh,
    pub bomb_mesh: GpuMesh,
    pub chain_head_mesh: GpuMesh,
    pub chain_body_mesh: GpuMesh,
    pub arrow_mesh: GpuMesh,
    pub dot_mesh: GpuMesh,
    pub chain_dot_mesh: GpuMesh,
    obstacle_mesh: GpuMesh,
}

pub struct BeatmapEditor {
    pub map: Option<BeatmapProject>,
    pub mesh_set: BeatmapMeshSet
}

#[derive(thiserror::Error, Debug)]
pub enum MapLoadError {
    #[error("Given path is not a directory, or can't be read: {0}")]
    FileNotADirectory(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("JSON Parse Error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

static DEFAULT_NOTE: &[u8] = include_bytes!("../assets/meshes/color_note.json");
static DEFAULT_ARROW: &[u8] = include_bytes!("../assets/meshes/arrow.json");
static DEFAULT_DOT: &[u8] = include_bytes!("../assets/meshes/color_note_dot.json");
static DEFAULT_CHAIN_HEAD: &[u8] = include_bytes!("../assets/meshes/chain_note_head.json");
static DEFAULT_CHAIN_LINK: &[u8] = include_bytes!("../assets/meshes/chain_note_link.json");
static DEFAULT_CHAIN_DOT: &[u8] = include_bytes!("../assets/meshes/chain_note_link_dot.json");
static CUBE: &[u8] = include_bytes!("../assets/meshes/cube.json");

static NOTE_TEXTURE: &[u8] = include_bytes!("../assets/textures/color_note.png");
static ARROW_TEXTURE: &[u8] = include_bytes!("../assets/textures/arrow.png");

impl BeatmapMeshSet {
    pub fn new(gl: &Context, renderer: &mut Renderer) -> Result<Self, MapLoadError> {

        let dir = get_data_folder().unwrap();

        let _ = std::fs::create_dir_all(&dir);

        let note_tex = dir.join("color_note.png");
        let arrow_tex = dir.join("arrow.png");

        std::fs::write(&note_tex, NOTE_TEXTURE).unwrap();
        std::fs::write(&arrow_tex, ARROW_TEXTURE).unwrap();

        renderer.texture_paths.insert("builtin:color_note".to_string(), note_tex);
        renderer.texture_paths.insert("builtin:arrow".to_string(), arrow_tex);

        macro_rules! setup_mesh {
            ($data:ident) => {
                {
                    let lm: LightMeshData = serde_json::from_slice($data)?;
                    let lm: LightMesh = lm.into();
                    let mut mesh = GpuMesh::empty(gl);
                    mesh.set_from_full_light_mesh(gl, &lm, &renderer.texture_paths, &renderer.atlas_map);
                    mesh
                }
            };
        }

        let note_mesh = setup_mesh!(DEFAULT_NOTE);
        let bomb_mesh = setup_mesh!(DEFAULT_NOTE);
        let chain_head_mesh = setup_mesh!(DEFAULT_CHAIN_HEAD);
        let chain_body_mesh = setup_mesh!(DEFAULT_CHAIN_LINK);
        let arrow_mesh = setup_mesh!(DEFAULT_ARROW);
        let dot_mesh = setup_mesh!(DEFAULT_DOT);
        let chain_dot_mesh = setup_mesh!(DEFAULT_CHAIN_DOT);
        let obstacle_mesh = setup_mesh!(CUBE);

        Ok(Self {
            note_mesh,
            bomb_mesh,
            chain_head_mesh,
            chain_body_mesh,
            arrow_mesh,
            dot_mesh,
            chain_dot_mesh,
            obstacle_mesh,
        })
    }
}

impl BeatmapEditor {
    pub fn new(map: Option<PathBuf>, gl: &Context, renderer: &mut Renderer) -> Result<Self, MapLoadError> {

        let mut s = Self {
            map: None,
            mesh_set: BeatmapMeshSet::new(gl, renderer)?,
        };

        if let Some(map) = map {
            s.load(map, gl, renderer)?
        }

        Ok(s)
    }

    pub fn load(&mut self, map: PathBuf, gl: &Context, renderer: &mut Renderer) -> Result<(), MapLoadError> {
        let span = tracing::debug_span!("load beatmap");
        let _guard = span.enter();

        tracing::debug!(target: DB_DATA, "Loading beatmap");

        if !map.is_dir() {
            return Err(MapLoadError::FileNotADirectory(map.to_string_lossy().to_string()))
        }

        let mut info_file = None;
        for file in map.read_dir()? {
            let file = file?;
            let name = file.file_name().to_string_lossy().to_lowercase();

            if name == "info.dat" {
                info_file = Some(file.path());
            }
        }

        let mut sets = Vec::new();
        let mut cover_image = None;

        if let Some(path) = info_file.as_deref() {
            let data = std::fs::read(path)?;
            let info: InfoFile = serde_json::from_slice(&data)?;

            match info {
                InfoFile::V2(v2) => {
                    sets = v2.difficulty_beatmap_sets.iter().map(Into::into).collect();
                    cover_image = Some(PathBuf::from(v2.cover_image_filename));
                }
            }

        }

        let project = BeatmapProject {
            folder: map,
            info_path: info_file,
            audio: None,
            cover_image,
            sets,
        };

        self.map = Some(project);

        Ok(())
    }
}

impl App {
    pub fn draw_beatmap_editor(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, shift: bool, ctrl: bool) {
        let gl = frame.gl().unwrap();

        egui::TopBottomPanel::top("menu_bar_beatmap_editor").show(ctx, |ui| {
            ui.add_space(2.);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open map\u{2026} \u{2502}").clicked() {
                        tracing::debug!(target: DB_LOGIC, "Spawning thread for opening beatmap project");
                        let (sx, rx) = mpsc::channel();
                        thread::spawn(move || {
                            let Some(map_folder) = rfd::FileDialog::new()
                                .set_title("Open Beatmap...")
                                .pick_folder() else {
                                    tracing::debug!(target: DB_LOGIC, "Canceled opening beatmap");
                                    return;
                                };
                            tracing::debug!(target: DB_LOGIC, ?map_folder, "Opening beatmap");
                            let _ = sx.send(map_folder);
                        });
                        self.add_routine(Box::new(move |s, gl| {
                            match rx.try_recv() {
                                Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    RoutineAction::Remove
                                }
                                Ok(folder) => {
                                    let rd = RefDuper;
                                    let s2 = unsafe { rd.detach_mut_ref(s) };
                                    if let Err(e) = s.load_beatmap(folder, gl, &mut s2.render.renderer) {
                                        s.set_status(None, "Failed to load beatmap", 2.);
                                        tracing::error!(target: DB_MAIN, "Failed to load beatmap: {e}");
                                    }
                                    RoutineAction::Remove
                                }
                            }
                        }));
                    }
                    if ui.button("Menu      \u{2502}").clicked() {
                        tracing::debug!(target: DB_LOGIC, "Returning to menu");
                        self.context = EditorContext::None;
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("scrub_controls")
            .exact_height(150.)
            .show(ctx, |ui| {

                ui.label("Seek controls")

            });

        egui::SidePanel::left("left_panel")
            .exact_width(300.)
            .show(ctx, |ui| {
                //
            });

        egui::SidePanel::right("right_panel")
            .exact_width(300.)
            .show(ctx, |ui| {
                //
            });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.state.vp_rect = rect;

                let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                self.handle_3d_input(&resp, ctx, gl);

                let s = unsafe { UnsafeMutRef::new(self) };

                ui.painter().add(egui::PaintCallback {
                    rect,
                    callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
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


                                draw_map_gl(&s, gl, &view, &proj, (w as i32, h as i32));

                                if s.state.show_grid && s.state.view_style == ViewStyle::Edit {
                                    s.render.renderer.draw_map_grid(gl, &view, &proj);
                                }
                            }

                        }
                    ))
                })

            });

    }

    pub fn load_beatmap(&mut self, folder: PathBuf, gl: &Context, renderer: &mut Renderer) -> Result<(), MapLoadError> {
        self.map_editor.load(folder, gl, renderer)
    }
}

fn draw_map_gl(
    s: &UnsafeMutRef<App>, gl: &glow::Context,
    view: &Mat4, proj: &Mat4,
    window: (i32, i32),
) {

    

    match s.state.view_style {
        ViewStyle::Edit => {

        },
        ViewStyle::Beatcraft { .. } => todo!(),
    }
}



