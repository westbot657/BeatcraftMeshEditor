use std::collections::HashMap;

use glam::{Vec2, Vec4};

use super::map_editing::{EditingData, ResolveError, Value};

pub mod v2;
pub mod v3;
pub mod v4;

pub trait Exportable: Sized {
    fn export(data: &EditingData, mods: &[Mod], values: HashMap<String, Value>) -> ExportResult<Self>;
}

pub enum ExportWarning {
    /// For things like precision placement/noodle being ignored
    Misplacement {
        beat: f32,
        expected: Vec2,
        effective: Vec2,
    },
    ObstacleSize {
        beat: f32,
        expected: Vec2,
        effective: Vec2,
    },
    /// For things like ignored noodle values like scale, dissolve, etc
    IgnoredAttribute {
        beat: f32,
        name: String,
        value: String,
    },
    /// For chroma
    IgnoredColor {
        beat: f32,
        color: Vec4,
    },
    /// For non-v4 exports that have njs changes
    NJSChange {
        beat: f32,
        value: f32,
    },
    /// Odd beat alignment
    BeatAlignment {
        beat: f32,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("{0}")]
    ResolveError(#[from] ResolveError),
}

pub enum ExportResult<V> {
    Ok {
        output: V,
        warnings: Vec<ExportWarning>,
    },
    Err {
        errors: Vec<ExportError>,
        warnings: Vec<ExportWarning>,
    }
}

pub enum Mod {
    Chroma,
    Noodle,
    Vivify,
    PrecisionPlace,
}

impl EditingData {
    pub fn export<V: Exportable>(&self, mods: &[Mod], values: HashMap<String, Value>) -> ExportResult<V> {
        V::export(self, mods, values)
    }
}




