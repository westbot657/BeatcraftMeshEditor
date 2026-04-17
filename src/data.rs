use std::collections::HashMap;
use std::ops::Not;
use std::path::PathBuf;

use glam::{Quat, Vec2, Vec3, Vec4};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::easing::Easing;
use crate::editor::{ActionType, Camera, ViewPlacement};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(untagged)]
pub enum VertexId {
    Index(usize),
    Named(String),
}

impl AsRef<VertexId> for VertexId {
    fn as_ref(&self) -> &VertexId {
        self
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(untagged)]
pub enum UvId {
    Index(usize),
    Named(String),
}

impl AsRef<UvId> for UvId {
    fn as_ref(&self) -> &UvId {
        self
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(untagged)]
pub enum NormalId {
    Index(usize),
    Named(String),
}

impl AsRef<NormalId> for NormalId {
    fn as_ref(&self) -> &NormalId {
        self
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VertRefData {
    Full(VertexId, UvId, NormalId),
    WithUv(VertexId, UvId),
    Bare(VertexId),
}

impl VertRefData {
    pub(crate) fn take_all(self) -> (VertexId, Option<UvId>, Option<NormalId>) {
        match self {
            Self::Full(v, u, n) => (v, Some(u), Some(n)),
            Self::WithUv(v, u) => (v, Some(u), None),
            Self::Bare(v) => (v, None, None),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TriangleData {
    pub verts: [VertRefData; 3],
    pub mat: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct StateSet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uv: Option<UvId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalId>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum TriangleEntry {
    Triangle(#[serde(with = "triangle_data_serde")] TriangleData),
    StateSet(StateSet),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComputeVertexData {
    pub points: [VertexId; 2],
    #[serde(default, skip_serializing_if = "Easing::is_default")]
    pub function: Easing,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ComputeNormalData {
    pub points: [VertexId; 3],
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct PartData {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vertices: Vec<Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub named_vertices: IndexMap<String, Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub compute_vertices: IndexMap<String, ComputeVertexData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub uvs: Vec<Vec2>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub named_uvs: IndexMap<String, Vec2>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub normals: Vec<Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub named_normals: IndexMap<String, Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub compute_normals: IndexMap<String, ComputeNormalData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub triangles: Vec<TriangleEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlacementData {
    pub part: String,
    #[serde(default, skip_serializing_if = "is_default_position")]
    pub position: Vec3,
    #[serde(default, skip_serializing_if = "is_default_rotation")]
    pub rotation: Quat,
    #[serde(default = "default_scale", skip_serializing_if = "is_default_scale")]
    pub scale: Vec3,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub remap_data: IndexMap<String, String>,
}

fn default_scale() -> Vec3 {
    Vec3::ONE
}

fn is_default_position(p: &Vec3) -> bool {
    *p == Vec3::ZERO
}

fn is_default_rotation(r: &Quat) -> bool {
    *r == Quat::IDENTITY
}

fn is_default_scale(s: &Vec3) -> bool {
    *s == Vec3::ONE
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
#[serde(default)]
pub struct MaterialData {
    pub material: u8,
    pub texture: u8,
    pub color: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct LightMeshData {
    pub mesh_format: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub credits: Vec<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub parts: IndexMap<String, PartData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mesh: Vec<PlacementData>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub textures: IndexMap<String, String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub data: IndexMap<String, MaterialData>,
    #[serde(skip_serializing_if = "Not::not")]
    pub cull: bool,
    #[serde(skip_serializing_if = "Clone::clone")]
    pub bloom_pass: bool,
    #[serde(skip_serializing_if = "Clone::clone")]
    pub mirror_pass: bool,
    #[serde(skip_serializing_if = "Clone::clone")]
    pub solid_pass: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub bloomfog_style: u8,
}

impl Default for LightMeshData {
    fn default() -> Self {
        Self {
            mesh_format: 1,
            credits: Vec::new(),
            parts: IndexMap::new(),
            mesh: Vec::new(),
            textures: IndexMap::new(),
            data: IndexMap::new(),
            cull: true,
            bloom_pass: true,
            mirror_pass: true,
            solid_pass: true,
            bloomfog_style: 0,
        }
    }
}

fn is_zero(v: &u8) -> bool {
    *v == 0
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub enum EventGroup {
    #[default]
    #[serde(skip_serializing)]
    None,
    #[serde(rename="outer-ring")]
    OuterRing,
    #[serde(rename="inner-ring")]
    InnerRing,
    #[serde(rename="left-spinning")]
    LeftSpinning,
    #[serde(rename="right-spinning")]
    RightSpinning,
}

impl EventGroup {
    pub fn is_none(&self) -> bool {
        *self == EventGroup::None
    }
}

#[derive(Debug, Default, Clone)]
pub enum TypeData {
    Spinning {
        axis: Vec3,
    },
    Rings {
        angles: Vec<f32>,
        deltas: Vec<f32>,
        starts: Option<[f32; 4]>,
    },
    #[default]
    None
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(transparent)]
pub struct IdList {
    list: Vec<LightIdElement>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum LightIdElement {
    GroupName(LightGroup),
    Id(u32)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LightGroup {
    #[serde(rename="left-lasers", alias="left-lights")]
    LeftLasers,
    #[serde(rename="right-lasers", alias="right-lights")]
    RightLasers,
    #[serde(rename="center-lasers", alias="center-light")]
    CenterLasers,
    #[serde(rename="back-lasers", alias="back-lights")]
    BackLasers,
    #[serde(rename="ring-lights", alias="ring-laseers")]
    RingLights,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct EnvPlacementData {
    #[serde(rename="type", skip_serializing_if="EventGroup::is_none")]
    pub typ: EventGroup,
    pub ids: IdList,
    pub position: Vec3,
    pub offset: Vec3,
    pub count: u32,
    #[serde(default, skip_serializing_if = "is_quat_identity")]
    pub rotation: Quat,
    #[serde(rename="rotation-offset")]
    pub rotation_offset: Quat,
    pub orientation: Quat,
    #[serde(rename="orientation-offset")]
    pub orientation_offset: Quat,
    #[serde(rename="id-step")]
    pub id_step: Vec<i32>,
    #[serde(flatten)]
    pub type_data: TypeData,
}

impl Default for EnvPlacementData {
    fn default() -> Self {
        Self {
            typ: Default::default(),
            ids: Default::default(),
            position: Default::default(),
            offset: Default::default(),
            count: 1,
            rotation: Default::default(),
            rotation_offset: Default::default(),
            orientation: Default::default(),
            orientation_offset: Default::default(),
            id_step: Default::default(),
            type_data: Default::default(),
        }
    }
}

impl From<&ViewPlacement> for EnvPlacementData {
    fn from(value: &ViewPlacement) -> Self {
        Self {
            typ: value.action_type.get_type(),
            ids: value.ids.clone(),
            position: value.position,
            offset: value.offset,
            count: value.count,
            rotation: value.rotation,
            rotation_offset: value.rotation_offset,
            orientation: value.orientation,
            orientation_offset: value.orientation_offset,
            id_step: value.id_step.clone(),
            type_data: value.action_type.to_data(),
        }
    }
}

impl EnvPlacementData {
    pub fn to_view(&self, resource_location: Option<String>, path: Option<PathBuf>) -> ViewPlacement {
        ViewPlacement {
            ids: self.ids.clone(),
            id_step: self.id_step.clone(),
            position: self.position,
            offset: self.offset,
            count: self.count,
            rotation: self.rotation,
            rotation_offset: self.rotation_offset,
            orientation: self.orientation,
            orientation_offset: self.orientation_offset,
            action_type: match (&self.typ, &self.type_data) {
                (EventGroup::None, TypeData::None) => ActionType::Static,
                (EventGroup::OuterRing, TypeData::Rings { angles, deltas, starts }) => ActionType::Ring {
                    layer: crate::editor::RingType::Outer,
                    angles: angles.clone(),
                    deltas: deltas.clone(),
                    start: *starts,
                },
                (EventGroup::InnerRing, TypeData::Rings { angles, deltas, starts }) => ActionType::Ring {
                    layer: crate::editor::RingType::Inner,
                    angles: angles.clone(),
                    deltas: deltas.clone(),
                    start: *starts,
                },
                (EventGroup::LeftSpinning, TypeData::Spinning { axis }) => ActionType::Spinning {
                    side: crate::editor::SpinSide::Left,
                    axis: Some(*axis)
                },
                (EventGroup::RightSpinning, TypeData::Spinning { axis }) => ActionType::Spinning {
                    side: crate::editor::SpinSide::Right,
                    axis: Some(*axis)
                },
                (EventGroup::LeftSpinning, TypeData::None) => ActionType::Spinning {
                    side: crate::editor::SpinSide::Left,
                    axis: None
                },
                (EventGroup::RightSpinning, TypeData::None) => ActionType::Spinning {
                    side: crate::editor::SpinSide::Right,
                    axis: None
                },
                _ => ActionType::Static
            },
            resource_location,
            path,
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EnvMeshData {
    MultiPlacement { placements: Vec<EnvPlacementData> },
    SinglePlacement(EnvPlacementData),
    None,
}

impl From<Vec<EnvPlacementData>> for EnvMeshData {
    fn from(mut value: Vec<EnvPlacementData>) -> Self {
        match value.len() {
            0 => Self::None,
            1 => Self::SinglePlacement(value.pop().unwrap()),
            _ => Self::MultiPlacement {
                placements: value
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct CameraData {
    pub target: Vec3,
    pub dist: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl From<CameraData> for Camera {
    fn from(value: CameraData) -> Self {
        Self {
            target: value.target,
            yaw: value.yaw,
            pitch: value.pitch,
            dist: value.dist,
            fov: 100f32.to_radians(),
        }
    }
}

impl From<Camera> for CameraData {
    fn from(value: Camera) -> Self {
        Self {
            target: value.target,
            dist: value.dist,
            yaw: value.yaw,
            pitch: value.pitch,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum TowerStyle {
    #[default]
    #[serde(rename = "cuboid")]
    Cuboid,
}

impl TowerStyle {
    pub fn name(&self) -> &'static str {
        "cuboid"
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SpectrogramData {
    #[serde(skip_serializing_if = "is_vec_zero")]
    pub position: Vec3,
    #[serde(skip_serializing_if = "is_quat_identity")]
    pub rotation: Quat,
    #[serde(skip_serializing_if = "is_vec_zero")]
    pub offset: Vec3,
    #[serde(skip_serializing_if = "i_is_127")]
    pub count: u32,
    pub style: TowerStyle,
    #[serde(rename="half-split", skip_serializing_if="Clone::clone")]
    pub half_split: bool,
    #[serde(rename="level-modifier", skip_serializing_if="f_is_1")]
    pub level_modifier: f32,
    #[serde(rename="base-height", skip_serializing_if="f_is_0")]
    pub base_height: f32,
    #[serde(skip_serializing_if = "Easing::is_default")]
    pub easing: Easing,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror: Option<Vec4>,
}
#[inline]
fn i_is_127(v: &u32) -> bool {
    *v == 127
}
#[inline]
fn f_is_1(v: &f32) -> bool {
    *v == 1.
}
#[inline]
fn f_is_0(v: &f32) -> bool {
    *v == 0.
}

impl Default for SpectrogramData {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            offset: Default::default(),
            count: 127,
            style: TowerStyle::Cuboid,
            half_split: true,
            level_modifier: 1.,
            base_height: 1.,
            easing: Easing::easeLinear,
            mirror: None,
        }
    }
}

fn is_vec_zero(v: &Vec3) -> bool {
    *v == Vec3::ZERO
}

fn is_quat_identity(q: &Quat) -> bool {
    q.is_near_identity()
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct EnvData {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub layout: IndexMap<String, EnvMeshData>,
    pub mirror: Option<String>,
    #[serde(rename = "fog-heights", skip_serializing_if = "Option::is_none")]
    pub fog_heights: Option<[f32; 2]>,
    pub spectrogram: Option<SpectrogramData>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SessionData {
    pub env_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub mesh_paths: HashMap<String, PathBuf>,
    pub camera: CameraData,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub texture_paths: HashMap<String, PathBuf>,
    #[serde(default)]
    pub mirror_path: Option<PathBuf>,

    #[serde(default, skip)]
    pub env: Option<EnvData>,
}


impl Serialize for EnvMeshData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            EnvMeshData::None => {
                // Serialize as empty object {}
                let map = serializer.serialize_map(Some(0))?;
                map.end()
            }
            EnvMeshData::MultiPlacement { placements } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("placements", placements)?;
                map.end()
            }
            EnvMeshData::SinglePlacement(data) => data.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for EnvMeshData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = serde_json::Value::deserialize(deserializer)?;

        match &map {
            serde_json::Value::Object(obj) if obj.is_empty() => Ok(EnvMeshData::None),
            serde_json::Value::Object(obj) if obj.contains_key("placements") => {
                let placements = obj["placements"]
                    .as_array()
                    .ok_or_else(|| serde::de::Error::custom("placements must be an array"))?
                    .iter()
                    .map(|v| EnvPlacementData::deserialize(v).map_err(serde::de::Error::custom))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(EnvMeshData::MultiPlacement { placements })
            }
            serde_json::Value::Object(_) => {
                let data = EnvPlacementData::deserialize(map)
                    .map_err(serde::de::Error::custom)?;
                Ok(EnvMeshData::SinglePlacement(data))
            }
            _ => Err(serde::de::Error::custom("expected an object")),
        }
    }
}

impl Serialize for TypeData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            TypeData::None => {
                let map = serializer.serialize_map(Some(0))?;
                map.end()
            }
            TypeData::Spinning { axis } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("axis", axis)?;
                map.end()
            }
            TypeData::Rings { angles, deltas, starts } => {
                let len = if starts.is_some() { 3 } else { 2 };
                let mut map = serializer.serialize_map(Some(len))?;
                map.serialize_entry("angles", angles)?;
                map.serialize_entry("deltas", deltas)?;
                if let Some(s) = starts {
                    map.serialize_entry("starts", s)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for TypeData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = serde_json::Value::deserialize(deserializer)?;

        match &map {
            serde_json::Value::Object(obj) if obj.is_empty() => Ok(TypeData::None),
            serde_json::Value::Object(obj) if obj.contains_key("axis") => {
                let axis = Vec3::deserialize(&obj["axis"])
                    .map_err(serde::de::Error::custom)?;
                Ok(TypeData::Spinning { axis })
            }
            serde_json::Value::Object(obj) if obj.contains_key("angles") || obj.contains_key("deltas") => {
                let angles = Vec::<f32>::deserialize(&obj["angles"])
                    .map_err(serde::de::Error::custom)?;
                let deltas = Vec::<f32>::deserialize(&obj["deltas"])
                    .map_err(serde::de::Error::custom)?;
                let starts = obj.get("starts")
                    .map(|v| <[f32; 4]>::deserialize(v).map_err(serde::de::Error::custom))
                    .transpose()?;
                Ok(TypeData::Rings { angles, deltas, starts })
            }
            _ => Err(serde::de::Error::custom("expected an object")),
        }
    }
}

mod triangle_data_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{NormalId, TriangleData, UvId, VertRefData, VertexId};

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub(crate) enum Id {
        String(String),
        Usize(usize),
    }

    impl From<&VertexId> for Id {
        fn from(value: &VertexId) -> Self {
            match value {
                VertexId::Index(i) => Self::Usize(*i),
                VertexId::Named(n) => Self::String(n.clone()),
            }
        }
    }

    impl From<&UvId> for Id {
        fn from(value: &UvId) -> Self {
            match value {
                UvId::Index(i) => Self::Usize(*i),
                UvId::Named(n) => Self::String(n.clone()),
            }
        }
    }

    impl From<&NormalId> for Id {
        fn from(value: &NormalId) -> Self {
            match value {
                NormalId::Index(i) => Self::Usize(*i),
                NormalId::Named(n) => Self::String(n.clone()),
            }
        }
    }

    impl From<Id> for VertexId {
        fn from(value: Id) -> Self {
            match value {
                Id::String(n) => VertexId::Named(n),
                Id::Usize(u) => VertexId::Index(u),
            }
        }
    }

    impl From<Id> for UvId {
        fn from(value: Id) -> Self {
            match value {
                Id::String(n) => UvId::Named(n),
                Id::Usize(u) => UvId::Index(u),
            }
        }
    }

    impl From<Id> for NormalId {
        fn from(value: Id) -> Self {
            match value {
                Id::String(n) => NormalId::Named(n),
                Id::Usize(u) => NormalId::Index(u),
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub(crate) enum IdInner {
        Full([Id; 3]),
        Uv([Id; 2]),
        Bare(Id),
    }

    impl From<&VertRefData> for IdInner {
        fn from(value: &VertRefData) -> Self {
            match value {
                VertRefData::Full(vert_id, uv_id, normal_id) => {
                    IdInner::Full([Id::from(vert_id), Id::from(uv_id), Id::from(normal_id)])
                }
                VertRefData::WithUv(vert_id, uv_id) => {
                    IdInner::Uv([Id::from(vert_id), Id::from(uv_id)])
                }
                VertRefData::Bare(n) => IdInner::Bare(Id::from(n)),
            }
        }
    }

    impl From<IdInner> for VertRefData {
        fn from(value: IdInner) -> Self {
            match value {
                IdInner::Full([v, u, n]) => VertRefData::Full(v.into(), u.into(), n.into()),
                IdInner::Uv([v, u]) => VertRefData::WithUv(v.into(), u.into()),
                IdInner::Bare(v) => VertRefData::Bare(v.into()),
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    enum TriData {
        WithMat([IdInner; 4]),
        Tri([IdInner; 3]),
    }

    impl From<&TriangleData> for TriData {
        fn from(t: &TriangleData) -> Self {
            if let Some(mat) = &t.mat && *mat != "default" {
                TriData::WithMat([
                    (&t.verts[0]).into(),
                    (&t.verts[1]).into(),
                    (&t.verts[2]).into(),
                    IdInner::Bare(Id::String(mat.to_string())),
                ])
            } else {
                TriData::Tri([
                    (&t.verts[0]).into(),
                    (&t.verts[1]).into(),
                    (&t.verts[2]).into(),
                ])
            }
        }
    }

    impl From<TriData> for TriangleData {
        fn from(value: TriData) -> Self {
            match value {
                TriData::WithMat([a, b, c, IdInner::Bare(Id::String(mat))]) => TriangleData {
                    verts: [a.into(), b.into(), c.into()],
                    mat: Some(mat),
                },
                TriData::Tri([a, b, c]) => TriangleData {
                    verts: [a.into(), b.into(), c.into()],
                    mat: None,
                },
                _ => unreachable!("Material can only be IdInner::Bare(Id::String)"),
            }
        }
    }

    pub(crate) fn serialize<S: Serializer>(t: &TriangleData, s: S) -> Result<S::Ok, S::Error> {
        TriData::from(t).serialize(s)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<TriangleData, D::Error> {
        Ok(TriData::deserialize(d)?.into())
    }
}

#[cfg(test)]
mod data_tests {
    use anyhow::anyhow;
    use serde_json::Value;

    use crate::data::*;
    use crate::light_mesh::BloomfogStyle;

    #[test]
    fn test_deserialize() -> anyhow::Result<()> {
        let _setup: Value = serde_json::from_str(include_str!("../local/old/test_mesh.json"))?;

        let data = serde_json::to_string(&_setup)?;

        println!("{}", serde_json::to_string_pretty(&_setup)?);

        let value: LightMeshData = serde_json::from_str(&data)?;

        let json = serde_json::to_string(&value)?;

        println!("{}", serde_json::to_string_pretty(&value)?);

        if json == data {
            Ok(())
        } else {
            Err(anyhow!("{data}\n!=\n{json}"))
        }
    }

    #[test]
    fn test_ser() -> anyhow::Result<()> {
        macro_rules! map {
            () => {
                IndexMap::new()
            };
            ( $( $key:literal: $value:expr ),* ) => {
                {
                    let mut map = indexmap::IndexMap::new();
                    $( map.insert($key.to_string(), $value); )*
                    map
                }
            };
        }

        let value = LightMeshData {
            mesh_format: 1,
            credits: vec!["Westbot".to_string()],
            parts: map! {
                "part0": PartData {
                    vertices: vec![],
                    named_vertices: map!{},
                    compute_vertices: map!{},
                    uvs: vec![],
                    named_uvs: map!{},
                    normals: vec![],
                    named_normals: map!{},
                    compute_normals: map!{},
                    triangles: vec![
                        TriangleEntry::StateSet(StateSet {
                            uv: Some(UvId::Index(0)),
                            normal: Some(NormalId::Named("up".to_string()))
                        }),
                        TriangleEntry::Triangle(TriangleData {
                            verts: [
                                VertRefData::Bare(VertexId::Named("v0".to_string())),
                                VertRefData::WithUv(VertexId::Named("v1".to_string()), UvId::Index(1)),
                                VertRefData::WithUv(VertexId::Named("v2".to_string()), UvId::Index(2)),
                            ],
                            mat: None
                        })
                    ]
                }
            },
            mesh: vec![],
            textures: IndexMap::new(),
            data: IndexMap::new(),
            cull: false,
            bloom_pass: true,
            mirror_pass: true,
            solid_pass: true,
            bloomfog_style: BloomfogStyle::default().into(),
        };

        let json = serde_json::to_string(&value)?;

        println!("{json}");
        Ok(())
    }
}
