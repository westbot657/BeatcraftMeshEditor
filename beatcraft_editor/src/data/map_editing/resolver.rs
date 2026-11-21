use beatmap_core::beatmap::{ArcCustomData, BombCustomData, ChainCustomData, CustomData, NoteColor, NoteCustomData, ObstacleCustomData};
use bs_mapping_data::custom_data::{OptVec2, OptVec3, OptVec4};
use bs_mapping_data::{ArcMidAnchorMode, Color, CutDirection};
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use beatmap_core::RuntimeData;
use beatmap_core::tracks_rs::animation::track::PathPropertiesMap;

#[derive(Clone, Debug, Default)]
pub struct LoopTracker<'l> {
    visited: Vec<&'l str>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum BaseValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    String(String),
    #[serde(untagged)]
    I32(i32),
    #[serde(untagged)]
    F32(f32),
    #[serde(untagged)]
    Bool(bool),
    #[serde(untagged)]
    None,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    BaseValue(BaseValue),
    Vec2([BaseValue; 2]),
    Vec3([BaseValue; 3]),
    Vec4([BaseValue; 4]),
}

impl<'l> LoopTracker<'l> {
    pub(crate) fn check(&mut self, entry: &'l str) -> Result<(), ResolveError> {
        if self.visited.contains(&entry) {
            Err(ResolveError::ReferenceLoop(self.visited.join(" -> ")))
        } else {
            self.visited.push(entry);
            Ok(())
        }
    }
}

pub(crate) trait Resolver: Sized {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError>;
}

impl Resolver for f32 {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::F32(f)) => Ok(*f),
            Value::BaseValue(BaseValue::I32(i)) => Ok(*i as f32),
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for Option<f32> {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::F32(f)) => Ok(Some(*f)),
            Value::BaseValue(BaseValue::I32(i)) => Ok(Some(*i as f32)),
            Value::BaseValue(BaseValue::None) => Ok(None),
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for i32 {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::I32(i)) => Ok(*i),
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for bool {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::Bool(b)) => Ok(*b),
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for u8 {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        let i: i32 = i32::resolve(value, values, lt)?;
        if !(0..=255).contains(&i) {
            return Err(ResolveError::OutOfRange {
                got: i,
                low: 0,
                high: 255,
            });
        }
        Ok(i as u8)
    }
}

impl Resolver for String {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::String(s)) => Ok(s.clone()),
            Value::BaseValue(BaseValue::Reference(r)) => match values.get(r) {
                None => Err(ResolveError::MissingValue(r.to_string())),
                Some(v) => {
                    lt.check(r)?;
                    v.resolve(values, lt)
                }
            },
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for Option<bool> {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::Bool(b)) => Ok(Some(*b)),
            Value::BaseValue(BaseValue::None) => Ok(None),
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for OptVec2 {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Value::Vec2([x, y]) => {
                let xlt = lt.clone();
                let x: Option<f32> = Value::BaseValue(x.clone()).resolve(values, xlt)?;
                let y: Option<f32> = Value::BaseValue(y.clone()).resolve(values, lt)?;
                Ok(OptVec2([x, y]))
            }
            Value::BaseValue(BaseValue::F32(f)) => {
                Ok(OptVec2([Some(*f), None]))
            }
            Value::BaseValue(BaseValue::I32(i)) => {
                Ok(OptVec2([Some(*i as f32), None]))
            }
            Value::BaseValue(BaseValue::None) => {
                Ok(OptVec2([None, None]))
            }
            _ => Err(ResolveError::WrongType(value.type_name()))
        }
    }
}

impl Resolver for OptVec3 {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Value::Vec3([x, y, z]) => {
                let xlt = lt.clone();
                let ylt = lt.clone();
                let x: Option<f32> = Value::BaseValue(x.clone()).resolve(values, xlt)?;
                let y: Option<f32> = Value::BaseValue(y.clone()).resolve(values, ylt)?;
                let z: Option<f32> = Value::BaseValue(z.clone()).resolve(values, lt)?;
                Ok(OptVec3([x, y, z]))
            }
            Value::Vec2([x, y]) => {
                let xlt = lt.clone();
                let x: Option<f32> = Value::BaseValue(x.clone()).resolve(values, xlt)?;
                let y: Option<f32> = Value::BaseValue(y.clone()).resolve(values, lt)?;
                Ok(OptVec3([x, y, None]))
            }
            Value::BaseValue(BaseValue::F32(f)) => {
                Ok(OptVec3([Some(*f), None, None]))
            }
            Value::BaseValue(BaseValue::I32(i)) => {
                Ok(OptVec3([Some(*i as f32), None, None]))
            }
            Value::BaseValue(BaseValue::None) => {
                Ok(OptVec3([None, None, None]))
            }
            _ => Err(ResolveError::WrongType(value.type_name()))
        }
    }
}

impl Resolver for OptVec4 {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Value::Vec4([x, y, z, w]) => {
                let xlt = lt.clone();
                let ylt = lt.clone();
                let zlt = lt.clone();
                let x: Option<f32> = Value::BaseValue(x.clone()).resolve(values, xlt)?;
                let y: Option<f32> = Value::BaseValue(y.clone()).resolve(values, ylt)?;
                let z: Option<f32> = Value::BaseValue(z.clone()).resolve(values, zlt)?;
                let w: Option<f32> = Value::BaseValue(w.clone()).resolve(values, lt)?;
                Ok(OptVec4([x, y, z, w]))
            }
            Value::Vec3([x, y, z]) => {
                let xlt = lt.clone();
                let ylt = lt.clone();
                let x: Option<f32> = Value::BaseValue(x.clone()).resolve(values, xlt)?;
                let y: Option<f32> = Value::BaseValue(y.clone()).resolve(values, ylt)?;
                let z: Option<f32> = Value::BaseValue(z.clone()).resolve(values, lt)?;
                Ok(OptVec4([x, y, z, None]))
            }
            Value::Vec2([x, y]) => {
                let xlt = lt.clone();
                let x: Option<f32> = Value::BaseValue(x.clone()).resolve(values, xlt)?;
                let y: Option<f32> = Value::BaseValue(y.clone()).resolve(values, lt)?;
                Ok(OptVec4([x, y, None, None]))
            }
            Value::BaseValue(BaseValue::F32(f)) => {
                Ok(OptVec4([Some(*f), None, None, None]))
            }
            Value::BaseValue(BaseValue::I32(i)) => {
                Ok(OptVec4([Some(*i as f32), None, None, None]))
            }
            Value::BaseValue(BaseValue::None) => {
                Ok(OptVec4([None, None, None, None]))
            }
            _ => Err(ResolveError::WrongType(value.type_name()))
        }
    }
}

impl Resolver for Option<Vec4> {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Value::Vec3([r, g, b]) => {
                let rlt = lt.clone();
                let glt = lt.clone();
                let r: f32 = Value::BaseValue(r.clone()).resolve(values, rlt)?;
                let g: f32 = Value::BaseValue(g.clone()).resolve(values, glt)?;
                let b: f32 = Value::BaseValue(b.clone()).resolve(values, lt)?;
                Ok(Some(Vec4::new(r, g, b, 1.)))
            }
            Value::Vec4([r, g, b, a]) => {
                let rlt = lt.clone();
                let glt = lt.clone();
                let blt = lt.clone();
                let r: f32 = Value::BaseValue(r.clone()).resolve(values, rlt)?;
                let g: f32 = Value::BaseValue(g.clone()).resolve(values, glt)?;
                let b: f32 = Value::BaseValue(b.clone()).resolve(values, blt)?;
                let a: f32 = Value::BaseValue(a.clone()).resolve(values, lt)?;
                Ok(Some(Vec4::new(r, g, b, a)))
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for CutDirection {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::I32(i)) => {
                if !(0..=8).contains(i) {
                    return Err(ResolveError::OutOfRange {
                        got: *i,
                        low: 0,
                        high: 8,
                    });
                }
                let u = *i as u8;
                let cd = CutDirection::try_from(u).unwrap();
                Ok(cd)
            }
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for ArcMidAnchorMode {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError>
    {
        match value {
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            },
            Value::BaseValue(BaseValue::I32(i)) => {
                if !(0..=2).contains(i) {
                    return Err(ResolveError::OutOfRange {
                        got: *i,
                        low: 0,
                        high: 2
                    });
                }
                let mode = ArcMidAnchorMode::try_from(*i as u8).unwrap();
                Ok(mode)
            },
            _ => Err(ResolveError::WrongType(value.type_name()))
        }
    }
}

impl Resolver for NoteColor {
    fn resolve<'l>(
        value: &'l Value,
        values: &HashMap<String, Value>,
        mut lt: LoopTracker<'l>,
    ) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::Reference(r)) => {
                lt.check(r)?;
                match values.get(r) {
                    None => Err(ResolveError::MissingValue(r.to_string())),
                    Some(v) => v.resolve(values, lt),
                }
            }
            Value::BaseValue(BaseValue::I32(i)) => {
                if !(0..=1).contains(i) {
                    return Err(ResolveError::OutOfRange {
                        got: *i,
                        low: 0,
                        high: 1,
                    });
                }
                let u = *i as u8;
                let c = Color::try_from(u).unwrap();
                Ok(c.into())
            }
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Value {
    pub(crate) fn resolve<T: Resolver>(
        &self,
        values: &HashMap<String, Value>,
        lt: LoopTracker,
    ) -> Result<T, ResolveError> {
        T::resolve(self, values, lt)
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::BaseValue(base_value) => match base_value {
                BaseValue::Reference(_) => "Reference",
                BaseValue::String(_) => "String",
                BaseValue::I32(_) => "i32",
                BaseValue::F32(_) => "f32",
                BaseValue::Bool(_) => "bool",
                BaseValue::None => "None",
            },
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Vec4(_) => "Vec4",
        }
    }
}

impl crate::data::map_editing::object_data::CustomData {
    pub fn resolve(
        &self,
        values: &HashMap<String, Value>,
        rt: &mut RuntimeData,
        point_definitions: &HashMap<String, serde_json::Value>,
    ) -> Result<CustomData, ResolveError> {
        let animation = match &self.animation {
            Some(a) => Some(PathPropertiesMap::deserialize(
                match a {
                    serde_json::Value::String(s) => point_definitions.get(s).ok_or(ResolveError::MissingValue(format!("track: {s}")))?.clone(),
                    _ => a.clone()
                },
                &mut rt.base_provider_context
            ).unwrap()),
            None => None,
        };
        Ok(CustomData {
            custom_njs_offset: self.custom_njs_offset.resolve(values, Default::default())?,
            custom_njs: self.custom_njs.resolve(values, Default::default())?,
            local_rotation: self.local_rotation.resolve(values, Default::default())?.to_quat(glam::EulerRot::YXZ),
            world_rotation: self.world_rotation.resolve(values, Default::default())?.to_quat(glam::EulerRot::YXZ),
            coordinates: self.coordinates.resolve(values, Default::default())?,
            animation,
        })
    }
}

impl crate::data::map_editing::object_data::NoteCustomData {
    pub fn resolve(
        &self,
        values: &HashMap<String, Value>,
        rt: &mut RuntimeData,
        point_definitions: &HashMap<String, serde_json::Value>,
    ) -> Result<NoteCustomData, ResolveError> {
        Ok(NoteCustomData {
            base: self.base.resolve(values, rt, point_definitions)?,
            disable_note_gravity: self.disable_note_gravity.resolve(values, Default::default())?,
            disable_note_look: self.disable_note_look.resolve(values, Default::default())?,
            spawn_effect: self.spawn_effect.resolve(values, Default::default())?,
            disable_debris: self.disable_debris.resolve(values, Default::default())?,
        })
    }
}

impl crate::data::map_editing::object_data::BombCustomData {
    pub fn resolve(
        &self,
        values: &HashMap<String, Value>,
        rt: &mut RuntimeData,
        point_definitions: &HashMap<String, serde_json::Value>,
    ) -> Result<BombCustomData, ResolveError> {
        Ok(BombCustomData {
            base: self.base.resolve(values, rt, point_definitions)?,
            disable_note_gravity: self.disable_note_gravity.resolve(values, Default::default())?,
        })
    }
}

impl crate::data::map_editing::object_data::ObstacleCustomData {
    pub fn resolve(
        &self,
        values: &HashMap<String, Value>,
        rt: &mut RuntimeData,
        point_definitions: &HashMap<String, serde_json::Value>,
    ) -> Result<ObstacleCustomData, ResolveError> {
        Ok(ObstacleCustomData {
            base: self.base.resolve(values, rt, point_definitions)?,
        })
    }
}

impl crate::data::map_editing::object_data::ChainCustomData {
    pub fn resolve(
        &self,
        values: &HashMap<String, Value>,
        rt: &mut RuntimeData,
        point_definitions: &HashMap<String, serde_json::Value>,
    ) -> Result<ChainCustomData, ResolveError> {
        Ok(ChainCustomData {
            base: self.base.resolve(values, rt, point_definitions)?,
            disable_note_gravity: self.disable_note_gravity.resolve(values, Default::default())?,
            disable_note_look: self.disable_note_look.resolve(values, Default::default())?,
            spawn_effect: self.spawn_effect.resolve(values, Default::default())?,
            disable_debris: self.disable_debris.resolve(values, Default::default())?,
            tail_coordinates: self.tail_coordinates.resolve(values, Default::default())?,
        })
    }
}

impl crate::data::map_editing::object_data::ArcCustomData {
    pub fn resolve(
        &self,
        values: &HashMap<String, Value>,
        rt: &mut RuntimeData,
        point_definitions: &HashMap<String, serde_json::Value>,
    ) -> Result<ArcCustomData, ResolveError> {
        Ok(ArcCustomData {
            base: self.base.resolve(values, rt, point_definitions)?,
        })
    }
}

impl From<CutDirection> for BaseValue {
    fn from(value: CutDirection) -> Self {
        let value: u8 = value.into();
        Self::I32(value as i32)
    }
}

impl From<f32> for BaseValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<i32> for BaseValue {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<u8> for BaseValue {
    fn from(value: u8) -> Self {
        Self::I32(value as i32)
    }
}

impl<T: Into<BaseValue>> From<T> for Value {
    fn from(value: T) -> Self {
        Self::BaseValue(value.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ResolveError {
    #[error("Value not present: {0}")]
    MissingValue(String),
    #[error("Invalid type for value: {0}")]
    WrongType(&'static str),
    #[error("Invalid value: {got}, expected between {low} and {high}")]
    OutOfRange { got: i32, low: i32, high: i32 },
    #[error("Reference Loop: {0}")]
    ReferenceLoop(String),
}