use std::collections::HashMap;

use glam::{Quat, Vec4};
use serde::{Deserialize, Serialize};

use crate::beatmap::object::{BeatmapController, ColorNote, NoteColor, RawNoteColor, RuntimeData};
use crate::{DB_DATA, beatmap};
use crate::beatmap::data::v2::{self, V2Note};
use crate::beatmap::data::{ArcMidAnchorMode, BeatmapFile, Color, CutDirection, ObstacleV2Type, Sentinel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectSource {
    #[deprecated = "re-route JSON through editor system."]
    Json { index: u32 },
    Element { index: u32 },
    TemplatePlacement { index_of_placement: usize, index_of_element: u32, },
    TemplateDefinition { name: String, index: u32, },
}

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
    I32(i32),
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
pub enum OptionalF32Value {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(untagged)]
    F32(f32),
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
    pub beat: F32Value,
    pub inputs: HashMap<String, Value>,
    pub mirror_y: bool,
    pub swap_colors: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub elements: Vec<DataElement>,
    pub values: HashMap<String, Value>,
    pub inputs: Vec<String>,
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

#[derive(Serialize, Deserialize)]
pub struct NoteData {
    pub beat: F32Value,
    pub cut_direction: CutDirectionValue,
    pub x: F32Value,
    pub y: F32Value,
    pub lane_rotation_deg: F32Value,
    pub note_type: NoteTypeValue,
    pub color: OptionalColorValue,
    pub angle_offset_deg: F32Value,
}

#[derive(Serialize, Deserialize)]
pub struct BombData {
    pub beat: F32Value,
    pub x: F32Value,
    pub y: F32Value,
    pub lane_rotation_deg: F32Value,
    pub color: OptionalColorValue,
}

#[derive(Serialize, Deserialize)]
pub struct ObstacleData {
    pub beat: F32Value,
    pub x: F32Value,
    pub y: F32Value,
    pub duration: F32Value,
    pub lane_rotation_deg: F32Value,
    pub width: F32Value,
    pub height: F32Value,
    /// if set, this overrides the length calculated from the duration
    pub length: OptionalF32Value,
    pub color: OptionalColorValue,
}

#[derive(Serialize, Deserialize)]
pub struct ObstacleTextData {
    pub beat: F32Value,
    pub text: F32Value,
    pub x: F32Value,
    pub y: F32Value,
    pub duration: F32Value,
    pub lane_rotation_deg: F32Value,
    pub width: F32Value,
    pub height: F32Value,
    /// if set, this overrides the length calculated from the duration
    pub length: OptionalF32Value,
    pub color: OptionalColorValue,
}

#[derive(Serialize, Deserialize)]
pub struct ChainData {
    pub beat: F32Value,
    pub cut_direction: CutDirectionValue,
    pub x: F32Value,
    pub y: F32Value,
    pub head_lane_rotation_deg: F32Value,
    pub tail_beat: F32Value,
    pub tx: F32Value,
    pub ty: F32Value,
    pub tail_lane_rotation_deg: F32Value,
    pub squish_factor: F32Value,
    pub link_count: U8Value,
    pub note_type: NoteTypeValue,
    pub color: OptionalColorValue,
}

#[derive(Serialize, Deserialize)]
pub struct ArcData {
    pub beat: F32Value,
    pub cut_direction: CutDirectionValue,
    pub x: F32Value,
    pub y: F32Value,
    pub lane_rotation_deg: F32Value,
    pub tail_beat: F32Value,
    pub tx: F32Value,
    pub ty: F32Value,
    pub mid_anchor_mode: ArcMidAnchorMode,
    pub note_type: NoteTypeValue,
    pub color: OptionalColorValue,
}

trait Resolver: Sized {
    fn resolve(value: &Value, values: &HashMap<String, Value>) -> Result<Self, ResolveError>;
}

impl Resolver for f32 {
    fn resolve(value: &Value, values: &HashMap<String, Value>) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::F32(f)) => Ok(*f),
            Value::BaseValue(BaseValue::I32(i)) => Ok(*i as f32),
            Value::BaseValue(BaseValue::Reference(r)) => match values.get(r) {
                None => Err(ResolveError::MissingValue(r.to_string())),
                Some(v) => v.resolve(values),
            },
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for Option<f32> {
    fn resolve(value: &Value, values: &HashMap<String, Value>) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::F32(f)) => Ok(Some(*f)),
            Value::BaseValue(BaseValue::I32(i)) => Ok(Some(*i as f32)),
            Value::None => Ok(None),
            Value::BaseValue(BaseValue::Reference(r)) => match values.get(r) {
                None => Err(ResolveError::MissingValue(r.to_string())),
                Some(v) => v.resolve(values),
            },
            _ => Err(ResolveError::WrongType(value.type_name()))
        }
    }
}

impl Resolver for i32 {
    fn resolve(value: &Value, values: &HashMap<String, Value>) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::I32(i)) => Ok(*i),
            Value::BaseValue(BaseValue::Reference(r)) => match values.get(r) {
                None => Err(ResolveError::MissingValue(r.to_string())),
                Some(v) => v.resolve(values),
            },
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for String {
    fn resolve(value: &Value, values: &HashMap<String, Value>) -> Result<Self, ResolveError> {
        match value {
            Value::BaseValue(BaseValue::String(s)) => Ok(s.clone()),
            Value::BaseValue(BaseValue::Reference(r)) => match values.get(r) {
                None => Err(ResolveError::MissingValue(r.to_string())),
                Some(v) => v.resolve(values),
            },
            _ => Err(ResolveError::WrongType(value.type_name())),
        }
    }
}

impl Resolver for OptionalColorValue {
    fn resolve(value: &Value, values: &HashMap<String, Value>) -> Result<Self, ResolveError> {
        todo!()
    }
}

impl Value {
    fn resolve<T: Resolver>(&self, values: &HashMap<String, Value>) -> Result<T, ResolveError> {
        T::resolve(self, values)
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::BaseValue(base_value) => match base_value {
                BaseValue::Reference(_) => "Reference",
                BaseValue::String(_) => "String",
                BaseValue::I32(_) => "i32",
                BaseValue::F32(_) => "f32",
            },
            Value::Vec2(_) => "Vec2",
            Value::Vec3(_) => "Vec3",
            Value::Vec4(_) => "Vec4",
            Value::None => "None",
        }
    }
}

impl F32Value {
    pub fn resolve(&self, values: &HashMap<String, Value>) -> Result<f32, ResolveError> {
        match self {
            F32Value::Reference(r) => match values.get(r) {
                None => Err(ResolveError::MissingValue(r.to_string())),
                Some(v) => v.resolve(values),
            },
            F32Value::F32(f) => Ok(*f),
        }
    }
}

impl OptionalF32Value {
    pub fn resolve(&self, values: &HashMap<String, Value>) -> Result<Option<f32>, ResolveError> {
        match self {
            OptionalF32Value::Reference(r) => match values.get(r) {
                None => Err(ResolveError::MissingValue(r.to_string())),
                Some(v) => v.resolve(values),
            },
            OptionalF32Value::F32(f) => Ok(Some(*f)),
            OptionalF32Value::None => Ok(None),
        }
    }
}

impl CutDirectionValue {
    pub fn resolve(&self, values: &HashMap<String, Value>) -> Result<CutDirection, ResolveError> {
        todo!()
    }
}

impl NoteTypeValue {
    pub fn resolve(&self, values: &HashMap<String, Value>) -> Result<NoteColor, ResolveError> {
        todo!()
    }
}

impl OptionalColorValue {
    pub fn resolve(&self, values: &HashMap<String, Value>) -> Result<Option<Vec4>, ResolveError> {
        todo!()
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

impl<T: Into<BaseValue>> From<T> for Value
{
    fn from(value: T) -> Self {
        Self::BaseValue(value.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CanonicalizationError {
    #[error("{0}")]
    ResolveError(#[from] ResolveError),
    #[error("Template not found: {0}")]
    InvalidTemplate(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ResolveError {
    #[error("Value not present: {0}")]
    MissingValue(String),
    #[error("Invalid type for value: {0}")]
    WrongType(&'static str),
}

impl EditingData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_beatmap(diff: &BeatmapFile) -> Self {

        let mut elements = Vec::new();

        match diff {
            BeatmapFile::V2(v2) => {
                let mut rotations = Vec::new();
                for event in v2.events.iter() {
                    if let v2::V2Event::SpawnRotation(rot) = event {
                        rotations.push(*rot);
                    }
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
                elements.reserve(v2.notes.len() + v2.obstacles.len() + v2.arcs.len());
                for note in v2.notes.iter() {
                    match note {
                        V2Note::Note(note) => {
                            let note_type = note.typ.into();
                            let beat = note.time;
                            let mut rotation_lane_deg = 0i32;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat)
                                    || (rot.beat == beat && rot.execution_time.is_early())
                                {
                                    rotation_lane_deg += rot.rotation_angle.get_degrees();
                                } else {
                                    break 'rot;
                                }
                            }
                            elements.push(
                                DataElement::Note(NoteData {
                                    beat: beat.into(),
                                    cut_direction: note.cut_direction.into(),
                                    x: note.line_index.into(),
                                    y: note.line_layer.into(),
                                    lane_rotation_deg: rotation_lane_deg.into(),
                                    note_type,
                                    color: OptionalColorValue::None,
                                    angle_offset_deg: 0f32.into(),
                                })
                            );

                        },
                        V2Note::Bomb(bomb) => {
                            let beat = bomb.beat;
                            let mut rotation_lane_deg = 0i32;
                            'rot: for rot in rotations.iter() {
                                if (rot.beat < beat)
                                    || (rot.beat == beat && rot.execution_time.is_early())
                                {
                                    rotation_lane_deg += rot.rotation_angle.get_degrees();
                                } else {
                                    break 'rot;
                                }
                            }
                            elements.push(
                                DataElement::Bomb(BombData {
                                    beat: beat.into(),
                                    x: bomb.line_index.into(),
                                    y: bomb.line_layer.into(),
                                    lane_rotation_deg: rotation_lane_deg.into(),
                                    color: OptionalColorValue::None,
                                })
                            );

                        },
                    }
                }
                for obst in v2.obstacles.iter() {
                    let (x, y, width, height) = match obst.typ {
                        ObstacleV2Type::FullHeight => (
                            obst.line_index, obst.line_layer,
                            obst.width, 5.,
                        ),
                        ObstacleV2Type::Crouch => (
                            obst.line_index, obst.line_layer + 2.,
                            obst.width, 3.,
                        ),
                        ObstacleV2Type::Free => (
                            obst.line_index, obst.line_layer,
                            obst.width, obst.height,
                        ),
                    };
                    let beat = obst.beat;
                    let mut lane_rotation_deg = 0;
                    'rot: for rot in rotations.iter() {
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation_angle.get_degrees();
                        } else {
                            break 'rot;
                        }
                    }
                    elements.push(
                        DataElement::Obstacle(ObstacleData {
                            beat: beat.into(),
                            x: x.into(),
                            y: y.into(),
                            duration: obst.duration.into(),
                            lane_rotation_deg: lane_rotation_deg.into(),
                            width: width.into(),
                            height: height.into(),
                            length: OptionalF32Value::None,
                            color: OptionalColorValue::None,
                        })
                    );
                }
            },
            BeatmapFile::V3(v3) => {
                let mut rotations = Vec::new();
                for event in v3.rotation_events.iter() {
                    rotations.push(*event);
                }
                rotations.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
                let mut chain_masks = Vec::with_capacity(v3.chains.len());
                for chain in v3.chains.iter() {
                    let note_type = chain.color.into();
                    let beat = chain.head_beat;
                    let t_beat = chain.tail_beat;
                    let mut lane_rotation_deg = 0.;
                    let mut tail_lane_rotation = 0.;
                    let mb = beat.max(t_beat);
                    for rot in rotations.iter() {
                        if rot.beat > mb {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                        if (rot.beat < t_beat) || (rot.beat == t_beat && rot.execution_time.is_early())
                        {
                            tail_lane_rotation += rot.rotation;
                        }

                    }
                    chain_masks.push((
                        chain.color, chain.head_beat,
                        chain.head_line_index, chain.head_line_layer,
                        chain.head_cut_direction,
                    ));
                    elements.push(
                        DataElement::Chain(ChainData {
                            beat: beat.into(),
                            cut_direction: chain.head_cut_direction.into(),
                            x: chain.head_line_index.into(),
                            y: chain.head_line_layer.into(),
                            head_lane_rotation_deg: lane_rotation_deg.into(),
                            tail_beat: chain.tail_beat.into(),
                            tx: chain.tail_line_index.into(),
                            ty: chain.tail_line_layer.into(),
                            tail_lane_rotation_deg: tail_lane_rotation.into(),
                            squish_factor: chain.squish_factor.into(),
                            link_count: chain.slice_count.into(),
                            note_type,
                            color: OptionalColorValue::None,
                        })
                    );
                }
                for note in v3.color_notes.iter() {
                    if chain_masks.iter().any(|(color, beat, x, y, c)| {
                        *color == note.color && *beat == note.beat
                        && *x == note.line_index && *y == note.line_layer
                        && *c == note.cut_direction
                    }) {
                        continue;
                    }
                    let note_type = note.color.into();
                    let beat = note.beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    elements.push(
                        DataElement::Note(NoteData {
                            beat: beat.into(),
                            cut_direction: note.cut_direction.into(),
                            x: note.line_index.into(),
                            y: note.line_layer.into(),
                            lane_rotation_deg: lane_rotation_deg.into(),
                            note_type,
                            color: OptionalColorValue::None,
                            angle_offset_deg: note.angle_offset.into(),
                        })
                    );
                }
                for bomb in v3.bomb_notes.iter() {
                    let beat = bomb.beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    elements.push(
                        DataElement::Bomb(BombData {
                            beat: beat.into(),
                            x: bomb.line_index.into(),
                            y: bomb.line_layer.into(),
                            lane_rotation_deg: lane_rotation_deg.into(),
                            color: OptionalColorValue::None,
                        })
                    );
                }
                for obst in v3.obstacles.iter() {
                    let beat = obst.beat;
                    let mut lane_rotation_deg = 0.;
                    for rot in rotations.iter() {
                        if rot.beat > beat {
                            break;
                        }
                        if (rot.beat < beat) || (rot.beat == beat && rot.execution_time.is_early())
                        {
                            lane_rotation_deg += rot.rotation;
                        }
                    }
                    elements.push(
                        DataElement::Obstacle(ObstacleData {
                            beat: beat.into(),
                            x: obst.line_index.into(),
                            y: obst.line_layer.into(),
                            duration: obst.duration.into(),
                            lane_rotation_deg: lane_rotation_deg.into(),
                            width: obst.width.into(),
                            height: obst.height.into(),
                            length: OptionalF32Value::None,
                            color: OptionalColorValue::None,
                        })
                    );
                }

            },
            BeatmapFile::V4(v4) => {
                let mut chain_masks = Vec::with_capacity(v4.chains.len());
                for chain in v4.chains.iter() {
                    let Some(head_data) = v4
                        .color_notes_data
                        .get(chain.head_note_metadata_index as usize)
                    else {
                        tracing::warn!(target: DB_DATA, "chain note references invalid color note data indes: {}", chain.head_note_metadata_index);
                        continue;
                    };
                    let Some(data) = v4.chains_data.get(chain.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "chain note references invalid chain note data index: {}", chain.metadata_index);
                        continue;
                    };
                    let note_type = head_data.color.into();
                    chain_masks.push((
                        head_data.color, chain.head_beat,
                        head_data.line_index, head_data.line_layer,
                        head_data.cut_direction,
                    ));
                    elements.push(
                        DataElement::Chain(ChainData {
                            beat: chain.head_beat.into(),
                            cut_direction: head_data.cut_direction.into(),
                            x: head_data.line_index.into(),
                            y: head_data.line_layer.into(),
                            head_lane_rotation_deg: chain.head_rotation_lane.into(),
                            tail_beat: chain.tail_beat.into(),
                            tx: data.tail_line_index.into(),
                            ty: data.tail_line_layer.into(),
                            tail_lane_rotation_deg: chain.tail_rotation_lane.into(),
                            squish_factor: data.squish_factor.into(),
                            link_count: data.slice_count.into(),
                            note_type,
                            color: OptionalColorValue::None,
                        })
                    );
                }
                for note in v4.color_notes.iter() {
                    let Some(data) = v4.color_notes_data.get(note.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "color note references invalid note data index: {}", note.metadata_index);
                        continue;
                    };

                    if chain_masks.iter().any(|(color, beat, x, y, c)| {
                        *color == data.color && *beat == note.beat
                        && *x == data.line_index && *y == data.line_layer
                        && *c == data.cut_direction
                    }) {
                        continue;
                    }
                    elements.push(
                        DataElement::Note(NoteData {
                            beat: note.beat.into(),
                            cut_direction: data.cut_direction.into(),
                            x: data.line_index.into(),
                            y: data.line_layer.into(),
                            lane_rotation_deg: note.rotation_lane.into(),
                            note_type: data.color.into(),
                            color: OptionalColorValue::None,
                            angle_offset_deg: data.angle_offset.into(),
                        })
                    );
                }
                for bomb in v4.bomb_notes.iter() {
                    let Some(data) = v4.bomb_notes_data.get(bomb.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "bomb note references invalid bomb data index: {}", bomb.metadata_index);
                        continue;
                    };
                    elements.push(
                        DataElement::Bomb(BombData {
                            beat: bomb.beat.into(),
                            x: data.line_index.into(),
                            y: data.line_layer.into(),
                            lane_rotation_deg: bomb.rotation_lane.into(),
                            color: OptionalColorValue::None,
                        })
                    );
                }
                for obst in v4.obstacles.iter() {
                    let Some(data) = v4.obstacles_data.get(obst.metadata_index as usize) else {
                        tracing::warn!(target: DB_DATA, "obstacle references invalid obstcale data index: {}", obst.metadata_index);
                        continue;
                    };
                    elements.push(
                        DataElement::Obstacle(ObstacleData {
                            beat: obst.beat.into(),
                            x: data.line_index.into(),
                            y: data.line_layer.into(),
                            duration: data.duration.into(),
                            lane_rotation_deg: obst.rotation_lane.into(),
                            width: data.width.into(),
                            height: data.height.into(),
                            length: OptionalF32Value::None,
                            color: OptionalColorValue::None,
                        })
                    );
                }
            },
        }

        // don't sort elements cuz it's lowkey pointless
        // since it can be sorted when exported or canonicalized

        Self {
            _id: Sentinel,
            version: Sentinel,
            elements,
            values: Default::default(),
            templates: Default::default(),
        }

    }

    fn random_quat(rng: &mut rand::rngs::ThreadRng) -> Quat {
        use rand::RngExt;
        Quat::from_euler(
            glam::EulerRot::ZYX,
            rng.random::<f32>() * 1.2 - 0.6,
            rng.random::<f32>() * 1.2 - 0.6,
            rng.random::<f32>() * 1.2 - 0.6,
        )
    }

    pub fn canonicalize(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        runtime_data: RuntimeData,
        map_values: &HashMap<String, Value>,
    ) -> Result<BeatmapController, CanonicalizationError> {

        let mut color_notes = Vec::new();
        let mut bomb_notes = Vec::new();
        let mut obstacles = Vec::new();
        let mut chain_notes = Vec::new();

        for element in self.elements.iter() {
            match element {
                DataElement::Note(note_data) => {
                    let mut color = note_data.note_type.resolve(&self.values)?;
                    if let Some(rgba) = note_data.color.resolve(&self.values)? {
                        color = match color {
                            NoteColor::Red | NoteColor::CustomRed(_) => NoteColor::CustomRed(rgba),
                            NoteColor::Blue | NoteColor::CustomBlue(_) => NoteColor::CustomBlue(rgba),
                        }
                    }
                    color_notes.push(ColorNote {
                        spawn_orientation: Self::random_quat(rng),
                        beat: note_data.beat.resolve(&self.values)?,
                        color,
                        cut_direction: note_data.cut_direction.resolve(&self.values)?,
                        angle_offset_deg: todo!(),
                        grid_pos: todo!(),
                        lane_rotation_deg: todo!(),
                        dissolve: todo!(),
                        index: todo!(),
                        source: todo!(),
                    })
                },
                DataElement::Bomb(bomb_data) => todo!(),
                DataElement::Obstacle(obstacle_data) => todo!(),
                DataElement::Chain(chain_data) => todo!(),
                DataElement::Arc(arc_data) => todo!(),
                DataElement::Template(template_placement) => todo!(),
                DataElement::ObstacleText(obstacle_text_data) => todo!(),
            }
        }

        Ok(BeatmapController {
            runtime_data,
            color_notes,
            bomb_notes,
            obstacles,
            chain_notes
        })
    }



}










