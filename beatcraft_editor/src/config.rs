use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::DateTime;
use egui::{Key, KeyboardShortcut, Modifiers};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader, static_loader};
use serde::{Deserialize, Serialize};

use crate::DB_DATA;
use crate::beatmap::data::InfoFile;

static_loader! {
    static LOCALES = {
        locales: "src/assets/locales",
        fallback_language: "en-US",
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RawAppData {
    recents: RawRecentProjects,
    audio_volume: f32,
    #[serde(default)]
    keymaps: KeyMaps,

    locale: String,

    #[serde(flatten)]
    /// Intended to catch any malformed data to prevent intact data from being lost.
    catch_all: Option<serde_json::Map<String, serde_json::Value>>,
}

impl Default for RawAppData {
    fn default() -> Self {
        Self {
            recents: RawRecentProjects(Default::default()),
            audio_volume: 0.5,
            keymaps: Default::default(),
            locale: "en-US".to_string(),
            catch_all: None,
        }
    }
}

#[derive(Default, Debug)]
pub struct AppData {
    pub recents: RecentProjects,
    pub audio_volume: f32,
    pub keymaps: KeyMaps,
    pub locale: LocaleCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawRecentProjects(Vec<RawRecentProject>);

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RecentProjects(Vec<RecentProject>);

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename = "lowercase")]
pub enum ProjectKind {
    EnvironmentMesh,
    SaberMesh,
    NoteMesh,
    Beatmap,
    Lightshow,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectType {
    EnvironmentMesh,
    SaberMesh,
    NoteMesh,
    Beatmap { img: Option<String> },
    Lightshow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaps {
    pub toggle_vertices: KeyboardShortcut,
    pub toggle_wireframe: KeyboardShortcut,
    pub toggle_grid: KeyboardShortcut,
    pub toggle_render_style: KeyboardShortcut,
    pub toggle_mesh_part_back: KeyboardShortcut,
    pub toggle_mesh_part_forward: KeyboardShortcut,
    pub toggle_edit_component: KeyboardShortcut,
    pub toggle_assembly_view: KeyboardShortcut,
    pub create_or_remove_triangles: KeyboardShortcut,
    pub flip_triangles: KeyboardShortcut,
    pub create_vertex: KeyboardShortcut,

    pub toggle_map_playback: KeyboardShortcut,
    pub rotate_map_grid_left: KeyboardShortcut,
    pub rotate_map_grid_right: KeyboardShortcut,
    pub map_fly_forward: Key,
    pub map_fly_backward: Key,
    pub map_fly_left: Key,
    pub map_fly_right: Key,
    pub map_fly_up: Key,
    pub map_fly_down: Key,

    pub deselect: KeyboardShortcut,
    pub save: KeyboardShortcut,
    pub undo: KeyboardShortcut,
    pub redo: KeyboardShortcut,
    pub rebuild_meshes: KeyboardShortcut,
}

impl Default for KeyMaps {
    fn default() -> Self {
        Self {
            toggle_vertices: KeyboardShortcut::new(Modifiers::NONE, Key::V),
            toggle_wireframe: KeyboardShortcut::new(Modifiers::NONE, Key::L),
            toggle_grid: KeyboardShortcut::new(Modifiers::NONE, Key::G),
            toggle_render_style: KeyboardShortcut::new(Modifiers::NONE, Key::F),
            toggle_mesh_part_back: KeyboardShortcut::new(Modifiers::NONE, Key::ArrowLeft),
            toggle_mesh_part_forward: KeyboardShortcut::new(Modifiers::NONE, Key::ArrowRight),
            toggle_edit_component: KeyboardShortcut::new(Modifiers::NONE, Key::E),
            toggle_assembly_view: KeyboardShortcut::new(Modifiers::NONE, Key::I),
            create_or_remove_triangles: KeyboardShortcut::new(Modifiers::NONE, Key::N),
            flip_triangles: KeyboardShortcut::new(Modifiers::NONE, Key::R),
            create_vertex: KeyboardShortcut::new(Modifiers::NONE, Key::C),
            toggle_map_playback: KeyboardShortcut::new(Modifiers::NONE, Key::Space),
            rotate_map_grid_left: KeyboardShortcut::new(Modifiers::NONE, Key::ArrowLeft),
            rotate_map_grid_right: KeyboardShortcut::new(Modifiers::NONE, Key::ArrowRight),
            map_fly_forward: Key::W,
            map_fly_backward: Key::S,
            map_fly_left: Key::A,
            map_fly_right: Key::D,
            map_fly_up: Key::E,
            map_fly_down: Key::Q,
            deselect: KeyboardShortcut::new(Modifiers::NONE, Key::Escape),
            save: KeyboardShortcut::new(Modifiers::CTRL, Key::S),
            undo: KeyboardShortcut::new(Modifiers::CTRL, Key::Z),
            redo: KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::Z),
            rebuild_meshes: KeyboardShortcut::new(Modifiers::ALT, Key::R),
        }
    }
}

#[derive(Debug, Default)]
pub struct LocaleCache {
    pub locale: LanguageIdentifier,
    pub cache: HashMap<&'static str, String>,
}

impl LocaleCache {
    pub fn new(lang: &str) -> Self {
        Self {
            locale: LanguageIdentifier::from_str(lang).unwrap_or_default(),
            cache: Default::default(),
        }
    }

    pub fn get(&mut self, key: &'static str) -> &str {
        self.cache
            .entry(key)
            .or_insert_with(|| LOCALES.lookup(&self.locale, key))
    }

    pub fn get_with_args(
        &self,
        key: &'static str,
        args: &HashMap<Cow<'static, str>, FluentValue>,
    ) -> String {
        LOCALES.lookup_with_args(&self.locale, key, args)
    }

    pub fn set(&mut self, lang: LanguageIdentifier) {
        self.locale = lang;
        self.cache.clear();
    }
}

impl ProjectType {
    pub fn kind(&self) -> ProjectKind {
        match self {
            ProjectType::EnvironmentMesh => ProjectKind::EnvironmentMesh,
            ProjectType::SaberMesh => ProjectKind::SaberMesh,
            ProjectType::NoteMesh => ProjectKind::NoteMesh,
            ProjectType::Beatmap { .. } => ProjectKind::Beatmap,
            ProjectType::Lightshow => ProjectKind::Lightshow,
        }
    }
}

impl Display for ProjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectKind::EnvironmentMesh => write!(f, "Environment Mesh"),
            ProjectKind::SaberMesh => write!(f, "Saber Mesh"),
            ProjectKind::NoteMesh => write!(f, "Note Mesh"),
            ProjectKind::Beatmap => write!(f, "Beatmap"),
            ProjectKind::Lightshow => write!(f, "Lightshow"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawRecentProject {
    modified: i64,
    path: PathBuf,
    kind: ProjectKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentProject {
    pub modified: DateTime<chrono::Utc>,
    pub path: PathBuf,
    pub kind: ProjectType,
}

impl From<RawAppData> for AppData {
    fn from(value: RawAppData) -> Self {
        tracing::debug!(target: DB_DATA, ?value, "Loading from raw data");
        Self {
            recents: value.recents.into(),
            audio_volume: value.audio_volume,
            keymaps: value.keymaps,
            locale: LocaleCache::new(&value.locale),
        }
    }
}

impl From<RawRecentProjects> for RecentProjects {
    fn from(value: RawRecentProjects) -> Self {
        Self(value.0.into_iter().map(Into::into).collect())
    }
}

impl From<RawRecentProject> for RecentProject {
    fn from(value: RawRecentProject) -> Self {
        let kind = match value.kind {
            ProjectKind::EnvironmentMesh => ProjectType::EnvironmentMesh,
            ProjectKind::SaberMesh => ProjectType::SaberMesh,
            ProjectKind::NoteMesh => ProjectType::NoteMesh,
            ProjectKind::Beatmap => match fs::read_dir(&value.path) {
                Err(_) => ProjectType::Beatmap { img: None },
                Ok(iterator) => {
                    let mut src = None;
                    for file in iterator.flatten() {
                        let f = file.file_name().to_string_lossy().to_lowercase();
                        if f != "info.dat" {
                            continue;
                        }
                        if let Ok(data) = fs::read(file.path())
                            && let Ok(info) = serde_json::from_slice::<InfoFile>(&data)
                        {
                            src = Some(match info {
                                InfoFile::V2(v2) => v2.cover_image_filename.clone(),
                                InfoFile::V4(v4) => v4.cover_image_filename.clone(),
                            });
                        }
                    }
                    ProjectType::Beatmap { img: src }
                }
            },
            ProjectKind::Lightshow => ProjectType::Lightshow,
        };
        Self {
            modified: DateTime::from_timestamp(value.modified, 0).unwrap(),
            path: value.path,
            kind,
        }
    }
}

impl Deref for RecentProjects {
    type Target = Vec<RecentProject>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RecentProjects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&AppData> for RawAppData {
    fn from(value: &AppData) -> Self {
        tracing::debug!(target: DB_DATA, ?value, "Saving app data");
        Self {
            recents: (&value.recents).into(),
            audio_volume: value.audio_volume,
            keymaps: value.keymaps.clone(),
            locale: value.locale.locale.to_string(),
            catch_all: None,
        }
    }
}

impl From<&RecentProjects> for RawRecentProjects {
    fn from(value: &RecentProjects) -> Self {
        Self(value.0.iter().map(Into::into).collect())
    }
}

impl From<&RecentProject> for RawRecentProject {
    fn from(value: &RecentProject) -> Self {
        let kind = match value.kind {
            ProjectType::EnvironmentMesh => ProjectKind::EnvironmentMesh,
            ProjectType::SaberMesh => ProjectKind::SaberMesh,
            ProjectType::NoteMesh => ProjectKind::NoteMesh,
            ProjectType::Beatmap { .. } => ProjectKind::Beatmap,
            ProjectType::Lightshow => ProjectKind::Lightshow,
        };
        Self {
            modified: value.modified.timestamp(),
            path: value.path.clone(),
            kind,
        }
    }
}

impl AppData {
    pub fn mark_project_modified(&mut self, project: &Path, kind: ProjectKind) {
        'find_existing: {
            for proj in self.recents.iter_mut() {
                if proj.path == project && proj.kind.kind() == kind {
                    proj.modified = chrono::Utc::now();
                    break 'find_existing;
                }
            }
            self.recents.push(
                RawRecentProject {
                    modified: chrono::Utc::now().timestamp(),
                    path: project.to_path_buf(),
                    kind,
                }
                .into(),
            )
        }
    }
}
