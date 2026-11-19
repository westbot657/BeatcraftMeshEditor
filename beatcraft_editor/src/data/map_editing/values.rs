use crate::data::map_editing::resolver::{LoopTracker, ResolveError, Value};
use beatmap_core::beatmap::NoteColor;
use bs_mapping_data::custom_data::{ColorVec4, OptVec2, OptVec3, OptVec4, RotationVec3};
use bs_mapping_data::{ArcMidAnchorMode, Color, CutDirection};
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub enum BoolValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Bool(bool),
}

impl Default for BoolValue {
    fn default() -> Self {
        Self::Bool(false)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum OptionalBoolValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Bool(bool),
    #[default]
    #[serde(untagged)]
    None,
}

#[derive(Serialize, Deserialize)]
pub enum U8Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    U8(u8),
}

#[derive(Serialize, Deserialize)]
pub enum F32Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    F32(f32),
}

#[derive(Serialize, Deserialize)]
pub enum I32Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    I32(i32),
}

#[derive(Serialize, Deserialize, Default)]
pub enum OptionalF32Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    F32(f32),
    #[default]
    #[serde(untagged)]
    None,
}

#[derive(Serialize, Deserialize)]
pub enum CutDirectionValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    CutDir(CutDirection),
}

#[derive(Serialize, Deserialize)]
pub enum ArcMidAnchorModeValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Mode(ArcMidAnchorMode),
}

#[derive(Serialize, Deserialize)]
pub enum OptVec2Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Vec2(OptVec2)
}

impl Default for OptVec2Value {
    fn default() -> Self {
        Self::Vec2(Default::default())
    }
}

#[derive(Serialize, Deserialize)]
pub enum OptVec3Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Vec3(OptVec3)
}

impl Default for OptVec3Value {
    fn default() -> Self {
        Self::Vec3(Default::default())
    }
}

#[derive(Serialize, Deserialize)]
pub enum OptVec4Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Vec4(OptVec4)
}

impl Default for OptVec4Value {
    fn default() -> Self {
        Self::Vec4(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize)]
pub enum OptionalColorValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Rgba([F32Value; 4]),
    #[serde(untagged)]
    Rgb([F32Value; 3]),
    #[default]
    #[serde(untagged)]
    None,
}

#[derive(Serialize, Deserialize)]
pub enum NoteTypeValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    Color(Color),
}

impl F32Value {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<f32, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::F32(f) => Ok(*f),
        }
    }
}

impl I32Value {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<i32, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::I32(i) => Ok(*i),
        }
    }
}

impl BoolValue {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<bool, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt)
                }
            }
            Self::Bool(b) => Ok(*b)
        }
    }
}

impl OptionalBoolValue {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Option<bool>, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::Bool(b) => Ok(Some(*b)),
            Self::None => Ok(None)
        }
    }
}

impl U8Value {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<u8, ResolveError> {
        match self {
            U8Value::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            U8Value::U8(u) => Ok(*u),
        }
    }
}

impl OptionalF32Value {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Option<f32>, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::F32(f) => Ok(Some(*f)),
            Self::None => Ok(None),
        }
    }
}

impl CutDirectionValue {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<CutDirection, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::CutDir(cut_direction) => Ok(*cut_direction),
        }
    }
}

impl ArcMidAnchorModeValue {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<ArcMidAnchorMode, ResolveError> {
        match self {
            ArcMidAnchorModeValue::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            },
            ArcMidAnchorModeValue::Mode(arc_mid_anchor_mode) => Ok(*arc_mid_anchor_mode),
        }
    }
}

impl NoteTypeValue {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<NoteColor, ResolveError> {
        match self {
            NoteTypeValue::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            NoteTypeValue::Color(color) => Ok((*color).into()),
        }
    }
}

impl OptVec2Value {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<OptVec2, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::Vec2(v) => Ok(*v),
        }
    }
}

impl OptVec3Value {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<OptVec3, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::Vec3(v) => Ok(*v),
        }
    }
}

impl OptVec4Value {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<OptVec4, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::Vec4(v) => Ok(*v),
        }
    }
}

impl OptionalColorValue {
    pub fn resolve<'l>(
        &'l self,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Option<Vec4>, ResolveError> {
        match self {
            Self::Reference(r) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Self::Rgba([r, g, b, a]) => {
                let rlt = lt.clone();
                let glt = lt.clone();
                let blt = lt.clone();
                let r: f32 = r.resolve(values, rlt)?;
                let g: f32 = g.resolve(values, glt)?;
                let b: f32 = b.resolve(values, blt)?;
                let a: f32 = a.resolve(values, lt)?;
                Ok(Some(Vec4::new(r, g, b, a)))
            }
            Self::Rgb([r, g, b]) => {
                let rlt = lt.clone();
                let glt = lt.clone();
                let r: f32 = r.resolve(values, rlt)?;
                let g: f32 = g.resolve(values, glt)?;
                let b: f32 = b.resolve(values, lt)?;
                Ok(Some(Vec4::new(r, g, b, 1.)))
            }
            Self::None => Ok(None),
        }
    }
}

impl From<f32> for F32Value {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<i32> for F32Value {
    fn from(value: i32) -> Self {
        Self::F32(value as f32)
    }
}

impl From<i32> for I32Value {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<u8> for U8Value {
    fn from(value: u8) -> Self {
        Self::U8(value)
    }
}

impl From<CutDirection> for CutDirectionValue {
    fn from(value: CutDirection) -> Self {
        Self::CutDir(value)
    }
}

impl From<Color> for NoteTypeValue {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

impl From<bool> for BoolValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<bool> for OptionalBoolValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Option<bool>> for OptionalBoolValue {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(b) => Self::Bool(b),
            None => Self::None,
        }
    }
}

impl From<Option<f32>> for OptionalF32Value {
    fn from(value: Option<f32>) -> Self {
        match value {
            Some(v) => Self::F32(v),
            None => Self::None,
        }
    }
}

impl From<Option<RotationVec3>> for OptVec3Value {
    fn from(value: Option<RotationVec3>) -> Self {
        match value {
            Some(RotationVec3::Vec3(v)) => Self::Vec3(v),
            Some(RotationVec3::Y(y)) => Self::Vec3(OptVec3([None, Some(y), None])),
            None => Self::Vec3(OptVec3([None, None, None]))
        }
    }
}

impl From<RotationVec3> for OptVec3Value {
    fn from(value: RotationVec3) -> Self {
        match value {
            RotationVec3::Vec3(v) => Self::Vec3(v),
            RotationVec3::Y(y) => Self::Vec3(OptVec3([None, Some(y), None]))
        }
    }
}

impl From<ColorVec4> for OptionalColorValue {
    fn from(value: ColorVec4) -> Self {
        match value {
            ColorVec4::Vec4(v) => Self::Rgba([v.x.into(), v.y.into(), v.z.into(), v.w.into()]),
            ColorVec4::Vec3(v) => Self::Rgb([v.x.into(), v.y.into(), v.z.into()]),
        }
    }
}

impl From<Option<ColorVec4>> for OptionalColorValue {
    fn from(value: Option<ColorVec4>) -> Self {
        match value {
            Some(ColorVec4::Vec4(v)) => Self::Rgba([v.x.into(), v.y.into(), v.z.into(), v.w.into()]),
            Some(ColorVec4::Vec3(v)) => Self::Rgb([v.x.into(), v.y.into(), v.z.into()]),
            None => Self::None,
        }
    }
}

impl From<OptVec2> for OptVec2Value {
    fn from(value: OptVec2) -> Self {
        Self::Vec2(value)
    }
}

impl From<OptVec3> for OptVec3Value {
    fn from(value: OptVec3) -> Self {
        Self::Vec3(value)
    }
}

impl From<OptVec4> for OptVec4Value {
    fn from(value: OptVec4) -> Self {
        Self::Vec4(value)
    }
}