use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RawAppData {
    recents: RawRecentProjects,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AppData {
    pub recents: RecentProjects,
}

#[derive(Serialize, Deserialize)]
pub struct RawRecentProjects(Vec<RawRecentProject>);

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RecentProjects(Vec<RecentProject>);

#[derive(Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum ProjectKind {
    EnvironmentMesh,
    SaberMesh,
    NoteMesh,
    Beatmap,
    Lightshow,
}

#[derive(Serialize, Deserialize)]
pub struct RawRecentProject {
    modified: i64,
    path: PathBuf,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct RecentProject {
    /// Unix Timestamp
    pub modified: DateTime<chrono::Utc>,
    pub path: PathBuf,
}

impl From<RawAppData> for AppData {
    fn from(value: RawAppData) -> Self {
        Self {
            recents: value.recents.into(),
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
            path: value.path.clone()
        }
    }
}

