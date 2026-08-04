use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use chrono::DateTime;
use serde::{Deserialize, Serialize};

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
    pub kind: ProjectKind,
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
        Self {
            modified: DateTime::from_timestamp(value.modified, 0).unwrap(),
            path: value.path,
            kind: value.kind,
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
        Self {
            modified: value.modified.timestamp(),
            path: value.path.clone(),
            kind: value.kind,
        }
    }
}

impl AppData {
    pub fn mark_project_modified(&mut self, project: &Path, kind: ProjectKind) {
        'find_existing: {
            for proj in self.recents.iter_mut() {
                if proj.path == project && proj.kind == kind {
                    proj.modified = chrono::Utc::now();
                    break 'find_existing;
                }
            }
            self.recents.push(RecentProject {
                modified: chrono::Utc::now(),
                path: project.to_path_buf(),
                kind,
            })
        }
    }
}

