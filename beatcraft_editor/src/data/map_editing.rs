use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::beatmap::data::Sentinel;

#[derive(Serialize, Deserialize)]
pub enum DataElement {
    Note(NoteData),
    Bomb(BombData),
    Obstacle(ObstacleData),
    Chain(ChainData),
    Arc(ArcData),
    Template(TemplatePlacement),
    ObstacleText(ObstacleTextData),
}

#[derive(Serialize, Deserialize)]
pub enum BaseValue {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    String(String),
    #[serde(untagged)]
    F32(f32),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    BaseValue(BaseValue),
    Vec2([BaseValue; 2]),
    Vec3([BaseValue; 3]),
    Vec4([BaseValue; 4]),
}

/// Data specific to a single beatmap
#[derive(Default, Serialize, Deserialize)]
pub struct EditingData {
    _id: Sentinel<10>,
    pub version: Sentinel<1>,
    pub elements: Vec<DataElement>,
    pub values: HashMap<String, Value>,
    pub templates: HashMap<String, Template>,
}

/// Global, cross-beatmap data
#[derive(Default, Serialize, Deserialize)]
pub struct GlobalEditingData {
    _id: Sentinel<100>,
    pub version: Sentinel<1>,
    pub values: HashMap<String, Value>,
    pub templates: HashMap<String, Template>,
}

#[derive(Serialize, Deserialize)]
pub struct TemplatePlacement {
    pub template: String,
    pub inputs: HashMap<String, Value>,
    pub mirror_y: bool,
    pub swap_colors: bool,
    pub reverse_order: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub elements: Vec<DataElement>,
    pub values: HashMap<String, Value>,
    pub inputs: Vec<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub enum NoteColorData {
    #[default]
    Red,
    Blue,
    CustomRed(Value),
    CustomBlue(Value),
}

#[derive(Default, Serialize, Deserialize)]
pub enum ObjectColorData {
    #[default]
    Default,
    Custom(Value),
}

#[derive(Serialize, Deserialize)]
pub struct NoteData {
    pub beat: Value,
    pub cut_direction: Value,
    pub x: Value,
    pub y: Value,
    pub color: NoteColorData,
}

#[derive(Serialize, Deserialize)]
pub struct BombData {
    pub beat: Value,
    pub x: Value,
    pub y: Value,
    pub color: ObjectColorData,
}

#[derive(Serialize, Deserialize)]
pub struct ObstacleData {
    pub beat: Value,
    pub x: Value,
    pub y: Value,
    pub duration: Value,
    pub width: Value,
    pub height: Value,
}

#[derive(Serialize, Deserialize)]
pub struct ObstacleTextData {
    pub beat: Value,
    pub text: Value,
    pub x: Value,
    pub y: Value,
    pub width: Value,
    pub height: Value,
}

#[derive(Serialize, Deserialize)]
pub struct ChainData {
    pub beat: Value,
    pub cut_direction: Value,
    pub x: Value,
    pub y: Value,
    pub tail_beat: Value,
    pub tx: Value,
    pub ty: Value,
}

#[derive(Serialize, Deserialize)]
pub struct ArcData {
    pub beat: Value,
    pub cut_direction: Value,
    pub x: Value,
    pub y: Value,
    pub tail_beat: Value,
    pub tx: Value,
    pub ty: Value,
    pub mid_anchor_mode: Value,
}


