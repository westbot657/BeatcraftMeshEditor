use std::path::PathBuf;
use std::sync::mpsc;
use std::{fs, thread};

use eframe::glow::{self, Context, HasContext};
use egui::{ImageSource, TextBuffer};
use glam::{Mat4, Vec4};
use indexmap::IndexMap;

use crate::audio::{Audio, AudioError, AudioMode, AudioSystem};
use crate::config::{ProjectKind, ProjectType};
use crate::data::LightMeshData;
use crate::light_mesh::LightMesh;
use crate::render::{GpuMesh, GridType, MeshDrawCall, Renderer};
use crate::{DB_AUDIO, DB_DATA, DB_LOGIC, DB_MAIN, MISSING_EDITOR_ICON, RefDuper, UnsafeMutRef, editor, get_data_folder};
use crate::editor::{App, EditorContext, RoutineAction, ViewStyle};

use self::data::song_core::DifficultyBeatmapCustomDataV2;
use self::data::v2::{CharacteristicSetV2, DifficultyBeatmapV2};
use self::data::{AudioDataFile, BeatmapFile, InfoFile, MapCharacteristic, MapDifficulty};
use self::object::{BeatmapController, GameObject, ObjectType};

pub mod event;
pub mod object;
pub mod data;
pub mod render;
#[cfg(test)]
pub mod tests;

pub struct BeatmapProjectDiff {
    pub difficulty: MapDifficulty,
    pub beatmap_file: Option<PathBuf>,
    pub njs: f32,
    pub njs_offset: f32,
    pub custom_data: Option<DifficultyBeatmapCustomDataV2>,
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
    pub audio_info: Option<AudioDataFile>,
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
static OBSTACLE: &[u8] = include_bytes!("../assets/meshes/obstacle.json");

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
            ($data:ident, $name:literal) => {
                {
                    tracing::debug!(target: DB_MAIN, "Loading default mesh for {}", $name);
                    let lm: LightMeshData = serde_json::from_slice($data)?;
                    let lm: LightMesh = lm.into();
                    let mut mesh = GpuMesh::empty(gl);
                    mesh.set_from_full_light_mesh(gl, &lm, &renderer.texture_paths, &renderer.atlas_map);
                    mesh
                }
            };
        }

        let note_mesh = setup_mesh!(DEFAULT_NOTE, "Color Note");
        let bomb_mesh = setup_mesh!(DEFAULT_NOTE, "Bomb Note");
        let chain_head_mesh = setup_mesh!(DEFAULT_CHAIN_HEAD, "Chain Note Head");
        let chain_body_mesh = setup_mesh!(DEFAULT_CHAIN_LINK, "Chain Note Link");
        let arrow_mesh = setup_mesh!(DEFAULT_ARROW, "Arrow");
        let dot_mesh = setup_mesh!(DEFAULT_DOT, "Dot");
        let chain_dot_mesh = setup_mesh!(DEFAULT_CHAIN_DOT, "Chain Dot");
        let obstacle_mesh = setup_mesh!(OBSTACLE, "Obstacle");

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
    pub fn new(audio_system: &mut AudioSystem, map: Option<PathBuf>, gl: &Context, renderer: &mut Renderer, volume: f32) -> Result<Self, MapLoadError> {

        let mut s = Self {
            map: None,
            mesh_set: BeatmapMeshSet::new(gl, renderer)?,
            scroll_step: 0.125,
        };

        if let Some(map) = map {
            s.load(audio_system, map, gl, renderer, volume)?
        }

        Ok(s)
    }

    pub fn load(&mut self, s: &mut AudioSystem, map: PathBuf, gl: &Context, renderer: &mut Renderer, volume: f32) -> Result<(), MapLoadError> {
        let span = tracing::debug_span!("load beatmap");
        let _guard = span.enter();

        tracing::debug!(target: DB_DATA, "Loading beatmap");

        if !map.is_dir() {
            return Err(MapLoadError::FileNotADirectory(map.to_string_lossy().to_string()))
        }

        let mut info_file = None;
        let mut audio_info_path = None;
        for file in map.read_dir()? {
            let file = file?;
            let name = file.file_name().to_string_lossy().to_lowercase();

            if name == "info.dat" {
                info_file = Some(file.path());
            }
            if name == "bpminfo.dat" {
                audio_info_path = Some(file.path());
            }
        }

        let mut sets = Vec::new();
        let mut cover_image = None;
        let mut audio_info = None;
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
                },
                InfoFile::V4(v4) => {
                    let mut sts = IndexMap::new();
                    for diff in v4.difficulty_beatmaps.iter() {
                        let entry = sts.entry(&diff.characteristic).or_insert(Vec::new());
                        entry.push(BeatmapProjectDiff {
                            difficulty: diff.difficulty.clone(),
                            beatmap_file: Some(PathBuf::from(&diff.beatmap_data_filename)),
                            njs: diff.note_jump_movement_speed,
                            njs_offset: diff.note_jump_start_beat_offset,
                            custom_data: None,
                        });
                    }
                    for (ch, diffs) in sts.into_iter() {
                        sets.push(BeatmapProjectSet {
                            set: ch.clone(),
                            diffs,
                        });
                    }
                    audio_info_path = Some(PathBuf::from(&v4.audio.audio_data_filename));
                    cover_image = Some(PathBuf::from(&v4.cover_image_filename));
                    audio_path = Some(PathBuf::from(&v4.audio.song_filename));
                }
            }

            info_data = Some(info);
        }

        if let Some(path) = audio_info_path.as_deref() {
            let path = map.join(path);
            let data = std::fs::read(path)?;
            let info: AudioDataFile = serde_json::from_slice(&data)?;
            audio_info = Some(info);
        }

        let mut audio = None;
        if let Some(path) = audio_path {
            let ad = Audio::new(s, &map.join(path), AudioMode::Full)?;
            ad.set_volume(volume);
            audio = Some(ad);
        }

        let project = BeatmapProject {
            folder: map,
            info_path: info_file,
            info: info_data,
            audio_info_path,
            audio_info,
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

    fn await_beatmap_open(&mut self) {
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
                    if let Some(map) = s.map_editor.map.take()
                    && let Some(audio) = map.audio {
                        audio.stop();
                        drop(audio);
                        s.audio_system.remove_dead_audio();
                    }
                    s.render.renderer.beatmap.seek(0.);
                    if let Err(e) = s.load_beatmap(&mut s2.audio_system, folder, gl, &mut s2.render.renderer, s2.data.audio_volume) {
                        s.set_status(None, "Failed to load beatmap", 2.);
                        tracing::error!(target: DB_MAIN, "Failed to load beatmap: {e}");
                    }
                    RoutineAction::Remove
                }
            }
        }));

    }

    pub fn draw_beatmap_editor(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, shift: bool, ctrl: bool, alt: bool) {
        let gl = frame.gl().unwrap();

        egui::TopBottomPanel::top("menu_bar_beatmap_editor").show(ctx, |ui| {
            ui.add_space(2.);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open map\u{2026} \u{2502}").clicked() {
                        self.await_beatmap_open();
                    }
                    if ui.button("Menu      \u{2502}").clicked() {
                        tracing::debug!(target: DB_LOGIC, "Returning to menu");
                        self.context = EditorContext::None;
                        self.render.renderer.beatmap.seek(0.);
                        if let Some(map) = self.map_editor.map.take()
                            && let Some(audio) = map.audio {
                                audio.stop();
                                drop(audio);
                                self.audio_system.remove_dead_audio();
                                self.state.playback_speed = 1.;
                            }
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("scrub_controls")
            .exact_height(150.)
            .resizable(false)
            .show(ctx, |ui| {

                ui.add_space(5.);

                let mut rect = ui.available_rect_before_wrap();
                rect.set_height(50.);

                let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());

                if resp.hovered() {
                    let scroll = ctx.input(|i| i.raw_scroll_delta.y);
                    if scroll != 0.0 {
                        if alt {
                            if let Some(map) = self.map_editor.map.as_ref()
                                && let Some(controller) = map.controller.as_ref()
                                && let Some(audio) = map.audio.as_ref()
                                && let Some(length_secs) = audio.length_seconds()
                            {
                                let length_beats = controller.runtime_data.seconds_to_beat(length_secs);
                                let cursor = self.render.renderer.beatmap.beat() / length_beats;
                                self.render.renderer.beatmap.spectrogram_zoom(scroll, cursor);
                            }
                        } else {
                            self.render.renderer.beatmap.scroll(scroll.signum() * self.map_editor.scroll_step);
                        }
                    }
                }

                if (resp.clicked() || resp.dragged())
                    && let Some(pos) = resp.interact_pointer_pos()
                    && let Some(map) = self.map_editor.map.as_ref()
                    && let Some(controller) = map.controller.as_ref()
                    && let Some(audio) = map.audio.as_ref()
                    && let Some(length_secs) = audio.length_seconds()
                {
                    let frac_x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);

                    let (start, end) = self.render.renderer.beatmap.spectrogram_range();
                    let u = start + (end - start) * frac_x;
                    let seek_secs = length_secs * u;
                    let beat = controller.runtime_data.seconds_to_beat(seek_secs);

                    let _ = audio.seek(seek_secs);
                    self.render.renderer.beatmap.seek(beat);
                }

                let s = unsafe { UnsafeMutRef::new(self) };
                ui.painter().add(egui::PaintCallback {
                    rect,
                    callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
                        move |_info, painter| {
                            if let Some(map) = s.map_editor.map.as_ref()
                            && let Some(controller) = map.controller.as_ref()
                            && let Some(audio) = map.audio.as_ref()
                            && let Some(length) = audio.length_seconds() {
                                let gl = painter.gl();
                                let sec = controller.runtime_data.beat_to_seconds(s.render.renderer.beatmap.beat());
                                s.render.renderer.beatmap.render_spectrogram_ui(
                                    &mut s.ref_mut().render.renderer, gl, audio, sec, length,
                                );
                            }
                        }
                    ))
                });

                ui.add_space(5.);

                let vol = (self.data.audio_volume * 100.) as u32;
                let mut volume = vol;
                ui.add_sized(
                    [150., 20.],
                    egui::Slider::new(&mut volume, 0..=100).suffix("%").text("Audio volume")
                );
                if volume != vol {
                    let vol = volume as f32 / 100.;
                    self.data.audio_volume = vol;
                    if let Some(map) = self.map_editor.map.as_ref()
                        && let Some(audio) = map.audio.as_ref() {
                            audio.set_volume(vol);
                    }
                }


                let spd = (self.state.playback_speed * 100.) as u32;
                let mut speed = spd;
                ui.add_sized(
                    [150., 20.],
                    egui::Slider::new(&mut speed, 0..=200).suffix("%").text("Playback speed")
                );
                if speed != spd {
                    let spd = speed as f32 / 100.;
                    self.state.playback_speed = spd;
                    if let Some(map) = self.map_editor.map.as_ref()
                    && let Some(audio) = map.audio.as_ref() {
                        audio.set_speed(spd);
                    }
                }

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
            .frame(egui::Frame::default().fill(ctx.theme().default_visuals().panel_fill).inner_margin(0.))
            .show(ctx, |ui| {
                let rd = RefDuper;
                let self2 = unsafe { rd.detach_mut_ref(self) };
                match self2.map_editor.map.as_mut() {
                    None => {
                        // No map selected

                        ui.allocate_ui_with_layout(
                            [ui.available_width(), 100.].into(),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.add_space(5.);
                                if ui.button("Open Map Folder...").clicked() {
                                    self.await_beatmap_open();
                                }
                                ui.allocate_space(ui.available_size());
                            }
                        );

                        let mut to_open = None;
                        egui::ScrollArea::horizontal()
                            .max_width(ui.available_width())
                            .id_salt("recent beatmap panel")
                            .show(ui, |ui| {
                                ui.allocate_ui_with_layout(
                                    [ui.available_width(), 200.].into(),
                                    egui::Layout::left_to_right(egui::Align::Min),
                                    |ui| {
                                        for (modified, path, img) in self.data.recents
                                            .iter()
                                            .filter_map(|p| if let ProjectType::Beatmap{img} = &p.kind { Some((p.modified, &p.path, img)) } else { None })
                                        {
                                            let ext = path.with_extension("");
                                            let Some(label) = ext.file_name() else { continue };
                                            let label = label.to_string_lossy();
                                            let full_path = path.to_string_lossy();
                                            ui.allocate_ui_with_layout(
                                                [225., 200.].into(),
                                                egui::Layout::top_down(egui::Align::Center),
                                                |ui| {
                                                    if let Some(img) = img {
                                                        ui.image(format!("file://{}", path.join(img).to_string_lossy()));
                                                    } else {
                                                        ui.image(MISSING_EDITOR_ICON.clone());
                                                    }
                                                    ui.label(egui::RichText::new(label).strong())
                                                        .on_hover_text(full_path);
                                                    ui.label(modified.to_string());
                                                    if ui.button("Open").clicked() {
                                                        to_open = Some(path);
                                                    }
                                                }
                                            );
                                        }
                                    }
                                );
                            });

                        if let Some(path) = to_open {
                            let _ = self2.load_beatmap(&mut self.audio_system, path.clone(), &self.state.gl, &mut self.render.renderer, self.data.audio_volume);
                        }
                    },
                    Some(map) => {
                        match map.controller.as_ref() {
                            None => {
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
                                            InfoFile::V4(v4) => {
                                                ui.label("Info V4");
                                                ui.label(&v4.song.title);
                                                ui.label(&v4.song.sub_title);
                                                ui.label(format!(
                                                    "Artist: {}  BPM: {:.2}",
                                                    v4.song.author,
                                                    v4.audio.bpm,
                                                ));
                                            }
                                        }
                                        ui.add_space(15.);
                                    },
                                    None => {},
                                }

                                draw_map_diffs(ui, map);
                            },
                            Some(controller) => {
                                let rect = ui.available_rect_before_wrap();
                                self.state.vp_rect = rect;

                                let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                                self.handle_3d_input(&resp, ctx, gl);

                                if let Some(audio) = map.audio.as_ref()
                                    && audio.is_playing() {
                                        let sec = audio.position_seconds();
                                        let beat = controller.runtime_data.seconds_to_beat(sec);
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
                            },
                        }
                    },
                }

            });

    }

    pub fn load_beatmap(&mut self, audio_system: &mut AudioSystem, folder: PathBuf, gl: &Context, renderer: &mut Renderer, volume: f32) -> Result<(), MapLoadError> {
        let f2 = folder.clone();
        self.map_editor.load(audio_system, folder, gl, renderer, volume)?;
        self.data.mark_project_modified(&f2, crate::config::ProjectKind::Beatmap);
        Ok(())
    }
}

fn draw_map_diffs(ui: &mut egui::Ui, map: &mut BeatmapProject) {
    ui.horizontal(|ui| {
        for set in map.sets.iter_mut() {
            ui.allocate_ui_with_layout(
                [200., 50.].into(),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(set.set.display_name());
                    ui.separator();
                    for diff in set.diffs.iter_mut() {
                        if ui.button(diff.difficulty.display_name()).clicked() {
                            let path = diff.beatmap_file.as_deref().unwrap();
                            let path = map.folder.join(path);
                            let data = std::fs::read(path).unwrap();
                            match serde_json::from_slice::<BeatmapFile>(&data) {
                                Ok(diff2) => {
                                    if let Some(audio) = map.audio.as_ref() {
                                        if let Some(sample_count) = audio.sample_count() {
                                            let bpm_regions = match map.audio_info.as_ref() {
                                                None => Vec::new(),
                                                Some(info) => {
                                                    let r = info.bpm_regions();
                                                    tracing::debug!(target: DB_DATA, ?r, "Loaded BPM regions:");
                                                    r
                                                }
                                            };
                                            map.controller = Some(BeatmapController::new(
                                                map.info.as_ref().unwrap(),
                                                diff,
                                                &diff2,
                                                bpm_regions,
                                                sample_count,
                                                audio.sample_rate,
                                            ).unwrap());
                                        } else {
                                            tracing::warn!(target: DB_AUDIO, "Audio sample_count is None");
                                        }

                                    }
                                }
                                Err(e) => {
                                    tracing::error!(target: DB_DATA, "Error loading beatmap file:\n{}", e);
                                }
                            }
                        }
                    }
                }
            );
        }
    });

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
    let mut bomb_instances = Vec::new();
    let mut obstacle_instances = Vec::new();
    let mut chain_head_instances = Vec::new();
    let mut chain_tail_instances = Vec::new();

    let mut arrow_instances = Vec::new();
    let mut dot_instances = Vec::new();
    let mut chain_dot_instances = Vec::new();

    let beatmap = &s.render.renderer.beatmap;
    let beat = beatmap.beat();

    for (t, object) in controller.color_notes.iter().map(|x| (ObjectType::ColorNote, x as &dyn GameObject))
        .chain(controller.bomb_notes.iter().map(|x| (ObjectType::BombNote, x as &dyn GameObject)))
        .chain(controller.obstacles.iter().map(|x| (ObjectType::Obstacle, x as &dyn GameObject)))
        .chain(controller.chain_notes.iter().map(|x| (ObjectType::ChainHead, x as &dyn GameObject))) {

        let wp = Mat4::IDENTITY;

        if let Some(mat) = match s.state.view_style {
            ViewStyle::Edit => object.animate_simple(wp, beat, &controller.runtime_data, beatmap),
            ViewStyle::Beatcraft { .. } => object.animate_complex(wp, beat, &controller.runtime_data),
        } {
            let inst = object.get_instance(Vec4::ZERO, mat, &controller.runtime_data.color_scheme);
            match t {
                ObjectType::ColorNote => note_instances.push(inst.into()),
                ObjectType::BombNote => bomb_instances.push(inst.into()),
                ObjectType::Obstacle => obstacle_instances.push(inst.into()),
                ObjectType::ChainHead => {
                    chain_head_instances.push(inst.into());
                    arrow_instances.push(inst.into());
                    let Some(chain) = object.upcast_chain_head() else { unreachable!("only chain note heads are marked as ChainHead") };
                    for link in chain.get_links() {
                        if let Some(mat) = match s.state.view_style {
                            ViewStyle::Edit => link.animate_simple(wp, beat, &controller.runtime_data, beatmap),
                            ViewStyle::Beatcraft { .. } => link.animate_complex(wp, beat, &controller.runtime_data),
                        } {
                            let inst = link.get_instance(Vec4::ZERO, mat, &controller.runtime_data.color_scheme);
                            chain_tail_instances.push(inst.into());
                            chain_dot_instances.push(inst.into());
                        }
                    }
                }
                _ => {}
            }
            match object.arrow_type() {
                object::ArrowType::None => {},
                object::ArrowType::Arrow => arrow_instances.push(inst.into()),
                object::ArrowType::Dot => dot_instances.push(inst.into()),
                object::ArrowType::ChainDot => chain_dot_instances.push(inst.into()),
            }
        }
    }

    let m = &s.map_editor.mesh_set.note_mesh;
    let b = &s.map_editor.mesh_set.bomb_mesh;
    let o = &s.map_editor.mesh_set.obstacle_mesh;
    let c = &s.map_editor.mesh_set.chain_head_mesh;
    let cl = &s.map_editor.mesh_set.chain_body_mesh;

    let a = &s.map_editor.mesh_set.arrow_mesh;
    let d = &s.map_editor.mesh_set.dot_mesh;
    let cd = &s.map_editor.mesh_set.chain_dot_mesh;

    let calls = vec![
        MeshDrawCall {
            mesh: m,
            instances: note_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
        },
        MeshDrawCall {
            mesh: c,
            instances: chain_head_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
        },
        MeshDrawCall {
            mesh: cl,
            instances: chain_tail_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
        },
        MeshDrawCall {
            mesh: a,
            instances: arrow_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: true,
            mirror: false,
            obstacle: false,
        },
        MeshDrawCall {
            mesh: d,
            instances: dot_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: true,
            mirror: false,
            obstacle: false,
        },
        MeshDrawCall {
            mesh: cd,
            instances: chain_dot_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: true,
            mirror: false,
            obstacle: false,
        },
        MeshDrawCall {
            mesh: b,
            instances: bomb_instances,
            wireframe: false,
            cull: true,
            bloomfog: false,
            solid: true,
            bloom: false,
            mirror: false,
            obstacle: false,
        },
        MeshDrawCall {
            mesh: o,
            instances: obstacle_instances,
            wireframe: false,
            cull: false,
            bloomfog: false,
            solid: false,
            bloom: false,
            mirror: false,
            obstacle: true,
        }
    ];

    match s.state.view_style {
        ViewStyle::Edit => {
            s.ref_mut().render.renderer.draw_meshes(
                gl, view, proj,
                &calls,
                None,
                false,
                false,
            );
        },
        ViewStyle::Beatcraft { blackout_sky } => {
            s.ref_mut().render.renderer.draw_meshes_fancy(
                gl, view, proj,
                &calls,
                window, if s.state.show_grid { GridType::BeatGrid } else { GridType::None },
                s.render.mirror.as_ref(), false,
                s.view.fog_heights.unwrap_or([-50., -30.]),
                true,
                if blackout_sky { (0., 0., 0., 1.) } else { (0.07, 0.08, 0.11, 1.) }
            );
        },
    }

    match s.state.view_style {
        ViewStyle::Edit => {

        },
        ViewStyle::Beatcraft { .. } => {

        },
    }
}



