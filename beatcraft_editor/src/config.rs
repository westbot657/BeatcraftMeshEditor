use std::fmt::Display;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::beatmap::data::InfoFile;

#[derive(Serialize, Deserialize)]
pub struct RawAppData {
    recents: RawRecentProjects,
    audio_volume: f32,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AppData {
    pub recents: RecentProjects,
    pub audio_volume: f32,
}

#[derive(Serialize, Deserialize)]
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
    Beatmap {
        img: Option<String>,
    },
    Lightshow,
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

#[derive(Serialize, Deserialize)]
pub struct RawRecentProject {
    modified: i64,
    path: PathBuf,
    kind: ProjectKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentProject {
    /// Unix Timestamp
    pub modified: DateTime<chrono::Utc>,
    pub path: PathBuf,
    pub kind: ProjectType,
}

impl From<RawAppData> for AppData {
    fn from(value: RawAppData) -> Self {
        Self {
            recents: value.recents.into(),
            audio_volume: value.audio_volume,
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
            ProjectKind::Beatmap => {
                match fs::read_dir(&value.path) {
                    Err(_) => ProjectType::Beatmap { img: None },
                    Ok(iterator) => {
                        let mut src = None;
                        for file in iterator.flatten() {
                            let f = file.file_name().to_string_lossy().to_lowercase();
                            if f != "info.dat" { continue }
                            if let Ok(data) = fs::read(file.path())
                            && let Ok(info) = serde_json::from_slice::<InfoFile>(&data) {
                                src = Some(match info {
                                    InfoFile::V2(v2) => v2.cover_image_filename.clone(),
                                    InfoFile::V4(v4) => v4.cover_image_filename.clone(),
                                });
                            }
                        }
                        ProjectType::Beatmap { img: src }
                    }
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
        Self {
            recents: (&value.recents).into(),
            audio_volume: value.audio_volume,
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
            self.recents.push(RawRecentProject {
                modified: chrono::Utc::now().timestamp(),
                path: project.to_path_buf(),
                kind,
            }.into())
        }
    }
}

