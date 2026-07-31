use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use eframe::glow::Context;

use crate::audio::Audio;
use crate::{DB_DATA, DB_LOGIC, DB_MAIN};
use crate::editor::{App, EditorContext, RoutineAction};

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

pub struct BeatmapEditor {
    pub map: Option<BeatmapProject>
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

impl BeatmapEditor {
    pub fn new(map_gl: Option<(PathBuf, &Context)>) -> Result<Self, MapLoadError> {
        let mut s = Self {
            map: None,
        };

        if let Some((map, gl)) = map_gl {
            s.load(map, gl)?
        }

        Ok(s)
    }

    pub fn load(&mut self, map: PathBuf, gl: &Context) -> Result<(), MapLoadError> {
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
                                    if let Err(e) = s.load_beatmap(folder, gl) {
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
            .exact_height(100.)
            .show(ctx, |ui| {

                ui.label("Seek controls")

            });
    }

    pub fn load_beatmap(&mut self, folder: PathBuf, gl: &Context) -> Result<(), MapLoadError> {
        self.map_editor.load(folder, gl)
    }
}

