use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use eframe::glow::{self, Context, HasContext};
use egui::TextBuffer;
use glam::{Mat4, Vec4};

use crate::audio::{Audio, AudioError, AudioMode, AudioSystem};
use crate::data::LightMeshData;
use crate::light_mesh::LightMesh;
use crate::render::{GpuMesh, MeshDrawCall, Renderer};
use crate::{DB_DATA, DB_LOGIC, DB_MAIN, RefDuper, UnsafeMutRef, editor, get_data_folder};
use crate::editor::{App, EditorContext, RoutineAction, ViewStyle};

use self::data::v2::{CharacteristicSetV2, DifficultyBeatmapV2};
use self::data::{BeatmapFile, InfoFile, MapCharacteristic, MapDifficulty};
use self::object::{BeatmapController, GameObject};

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
        let path = &value.beatmap_filename;
        tracing::debug!(target: DB_DATA, ?path, "Loaded Beatmap difficulty: {}", value.difficulty);
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
    pub info: Option<InfoFile>,
    pub audio_info_path: Option<PathBuf>,
    pub audio: Option<std::sync::Arc<Audio>>,
    pub cover_image: Option<PathBuf>,
    pub sets: Vec<BeatmapProjectSet>,
    pub controller: Option<BeatmapController>,
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
    pub mesh_set: BeatmapMeshSet,
    pub scroll_step: f32,
}

#[derive(thiserror::Error, Debug)]
pub enum MapLoadError {
    #[error("Given path is not a directory, or can't be read: {0}")]
    FileNotADirectory(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("JSON Parse Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Audio Error: {0}")]
    AudioError(#[from] AudioError),
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

        renderer.rebuild_atlases(gl);

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
    pub fn new(audio_system: &mut AudioSystem, map: Option<PathBuf>, gl: &Context, renderer: &mut Renderer) -> Result<Self, MapLoadError> {

        let mut s = Self {
            map: None,
            mesh_set: BeatmapMeshSet::new(gl, renderer)?,
            scroll_step: 0.25,
        };

        if let Some(map) = map {
            s.load(audio_system, map, gl, renderer)?
        }

        Ok(s)
    }

    pub fn load(&mut self, s: &mut AudioSystem, map: PathBuf, gl: &Context, renderer: &mut Renderer) -> Result<(), MapLoadError> {
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
        let audio_info_path = None;
        let mut info_data = None;
        let mut audio_path = None;

        if let Some(path) = info_file.as_deref() {
            let data = std::fs::read(path)?;
            let info: InfoFile = serde_json::from_slice(&data)?;

            match &info {
                InfoFile::V2(v2) => {
                    sets = v2.difficulty_beatmap_sets.iter().map(Into::into).collect();
                    cover_image = Some(PathBuf::from(&v2.cover_image_filename));
                    audio_path = Some(PathBuf::from(&v2.song_filename));
                }
            }

            info_data = Some(info);
        }

        let mut audio = None;
        if let Some(path) = audio_path {
            audio = Some(Audio::new(s, &map.join(path), AudioMode::Full)?);
        }

        let project = BeatmapProject {
            folder: map,
            info_path: info_file,
            info: info_data,
            audio_info_path,
            audio,
            cover_image,
            sets,
            controller: None,
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
                                    if let Err(e) = s.load_beatmap(&mut s2.audio_system, folder, gl, &mut s2.render.renderer) {
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
                        self.map_editor.map = None;
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("scrub_controls")
            .exact_height(150.)
            .resizable(false)
            .show(ctx, |ui| {

                ui.label("Seek controls")

            });

        egui::SidePanel::left("left_panel")
            .exact_width(300.)
            .resizable(false)
            .show(ctx, |ui| {
                //
            });

        egui::SidePanel::right("right_panel")
            .exact_width(300.)
            .resizable(false)
            .show(ctx, |ui| {
                //
            });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                let rd = RefDuper;
                let self2 = unsafe { rd.detach_mut_ref(self) };
                match self2.map_editor.map.as_mut() {
                    None => {
                        // No map selected
                        ui.label("TODO: recent map selector");
                    },
                    Some(map) => {
                        if map.controller.is_none() {
                            ui.label(map.folder.to_string_lossy().as_str());

                            #[allow(clippy::single_match)]
                            match map.info.as_mut() {
                                Some(i) => {
                                    match i {
                                        InfoFile::V2(v2) => {
                                            ui.label("Info V2");
                                            ui.label(&v2.song_name);
                                            ui.label(&v2.song_sub_name);
                                            ui.horizontal(|ui| {
                                                ui.label(format!(
                                                    "Artist: {}  BPM: {:.2}  Mappers: {}",
                                                    v2.song_author_name,
                                                    v2.bpm,
                                                    v2.level_author_name
                                                ));
                                            });
                                        },
                                    }
                                    ui.add_space(15.);
                                },
                                None => {},
                            }

                            ui.horizontal(|ui| {
                                for set in map.sets.iter_mut() {
                                    ui.allocate_ui_with_layout(
                                        [200., 50.].into(), egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.label(set.set.display_name());
                                            ui.separator();
                                            for diff in set.diffs.iter_mut() {
                                                if ui.button(diff.difficulty.display_name()).clicked() {
                                                    let path = diff.beatmap_file.as_deref().unwrap();
                                                    let path = map.folder.join(path);
                                                    let data = std::fs::read(path).unwrap();
                                                    let diff2: BeatmapFile = serde_json::from_slice(&data).unwrap();
                                                    map.controller = Some(BeatmapController::new(
                                                        map.info.as_ref().unwrap(),
                                                        diff,
                                                        &diff2
                                                    ).unwrap());
                                                }
                                            }
                                        }
                                    );
                                }
                            });

                            // no difficulty selected
                        } else {
                            let rect = ui.available_rect_before_wrap();
                            self.state.vp_rect = rect;

                            let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                            self.handle_3d_input(&resp, ctx, gl);

                            if let Some(audio) = map.audio.as_ref()
                                && audio.is_playing() {
                                    let sec = audio.position_seconds();
                                    let beat = sec * (map.info.as_ref().unwrap().bpm() / 60.);
                                    self.render.renderer.beatmap.seek(beat);
                                }

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
                            });


                        }
                    },
                }
                
            });

    }

    pub fn load_beatmap(&mut self, audio_system: &mut AudioSystem, folder: PathBuf, gl: &Context, renderer: &mut Renderer) -> Result<(), MapLoadError> {
        self.map_editor.load(audio_system, folder, gl, renderer)
    }
}

fn draw_map_gl(
    s: &UnsafeMutRef<App>, gl: &glow::Context,
    view: &Mat4, proj: &Mat4,
    window: (i32, i32),
) {

    let controller = s.ref_mut().map_editor.map.as_mut().unwrap().controller.as_mut().unwrap();

    let rd = RefDuper;
    let controller = unsafe { rd.detach_mut_ref(controller) };

    let mut note_instances = Vec::new();
    let mut arrow_instances = Vec::new();
    let mut dot_instances = Vec::new();
    let mut chain_dot_instances = Vec::new();

    for note in controller.color_notes.iter() {
        let beatmap = &s.render.renderer.beatmap;
        if let Some(mat) = note.animate_simple(beatmap.beat(), &controller.runtime_data, beatmap) {
            let inst = note.get_instance(Vec4::ZERO, mat);
            note_instances.push(inst.into());
            match note.arrow_type() {
                object::ArrowType::None => {},
                object::ArrowType::Arrow => arrow_instances.push(inst.into()),
                object::ArrowType::Dot => dot_instances.push(inst.into()),
                object::ArrowType::ChainDot => chain_dot_instances.push(inst.into()),
            }
        }
        
    }

    let m = &s.map_editor.mesh_set.note_mesh;
    let a = &s.map_editor.mesh_set.arrow_mesh;
    let d = &s.map_editor.mesh_set.dot_mesh;
    let cd = &s.map_editor.mesh_set.chain_dot_mesh;

    let calls = vec![
        MeshDrawCall {
            mesh: m, 
            instances: note_instances.clone(),
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false
        },
        MeshDrawCall {
            mesh: a,
            instances: arrow_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
        },
        MeshDrawCall {
            mesh: d,
            instances: dot_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
        },
        MeshDrawCall {
            mesh: cd,
            instances: chain_dot_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
        }
    ];

    s.ref_mut().render.renderer.draw_meshes(
        gl, view, proj,
        &calls,
        None,
        false,
        false
    );

    match s.state.view_style {
        ViewStyle::Edit => {

        },
        ViewStyle::Beatcraft { .. } => {

        },
    }
}



