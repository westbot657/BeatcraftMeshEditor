use std::ops::Not;
use std::path::PathBuf;

use glam::{Quat, Vec2, Vec3};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::easing::Easing;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum VertId {
    Index(usize),
    Named(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum UvId {
    Index(usize),
    Named(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum NormalId {
    Index(usize),
    Named(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum VertRefData {
    Full(VertId, UvId, NormalId),
    WithUv(VertId, UvId),
    Bare(VertId),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TriangleData {
    pub verts: [VertRefData; 3],
    pub mat: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct StateSet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    uv: Option<UvId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    normal: Option<NormalId>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum TriangleEntry {
    Triangle(#[serde(with = "triangle_data_serde")] TriangleData),
    StateSet(StateSet)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComputeVertexData {
    points: [VertId; 2],
    #[serde(default, skip_serializing_if = "Easing::is_default")]
    function: Easing,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    delta: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    y: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    z: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct ComputeNormalData {
    points: [VertId; 3],
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct PartData {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    vertices: Vec<Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    named_vertices: IndexMap<String, Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    compute_vertices: IndexMap<String, ComputeVertexData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    uvs: Vec<Vec2>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    named_uvs: IndexMap<String, Vec2>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    normals: Vec<Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    named_normals: IndexMap<String, Vec3>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    compute_normals: IndexMap<String, ComputeNormalData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    triangles: Vec<TriangleEntry>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlacementData {
    part: String,
    #[serde(default, skip_serializing_if = "is_default_position")]
    position: Vec3,
    #[serde(default, skip_serializing_if = "is_default_rotation")]
    rotation: Quat,
    #[serde(default = "default_scale", skip_serializing_if = "is_default_scale")]
    scale: Vec3,
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

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct MaterialData {
    material: u8,
    texture: u8,
    color: u8,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct LightMeshData {
    mesh_format: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    credits: Vec<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    parts: IndexMap<String, PartData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    mesh: Vec<PlacementData>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    textures: IndexMap<String, String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    data: IndexMap<String, MaterialData>,
    #[serde(default, skip_serializing_if = "Not::not")]
    cull: bool
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SessionPlacementData {
    position: Vec3,
    rotation: Quat,
    count: usize,
    offset_pos: Vec3,
    offset_rot: Quat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionMeshData {
    path: PathBuf,
    placements: Vec<SessionPlacementData>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CameraData {
    target: Vec3,
    dist: f32,
    yaw: f32,
    pitch: f32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SessionData {
    meshes: Vec<SessionMeshData>,
    camera: CameraData,
}

mod triangle_data_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{NormalId, TriangleData, UvId, VertId, VertRefData};

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub(crate) enum Id {
        String(String),
        Usize(usize),
    }

    impl From<&VertId> for Id {
        fn from(value: &VertId) -> Self {
            match value {
                VertId::Index(i) => Self::Usize(*i),
                VertId::Named(n) => Self::String(n.clone())
            }
        }
    }

    impl From<&UvId> for Id {
        fn from(value: &UvId) -> Self {
            match value {
                UvId::Index(i) => Self::Usize(*i),
                UvId::Named(n) => Self::String(n.clone())
            }
        }
    }

    impl From<&NormalId> for Id {
        fn from(value: &NormalId) -> Self {
            match value {
                NormalId::Index(i) => Self::Usize(*i),
                NormalId::Named(n) => Self::String(n.clone())
            }
        }
    }

    impl From<Id> for VertId {
        fn from(value: Id) -> Self {
            match value {
                Id::String(n) => VertId::Named(n),
                Id::Usize(u) => VertId::Index(u)
            }
        }
    }

    impl From<Id> for UvId {
        fn from(value: Id) -> Self {
            match value {
                Id::String(n) => UvId::Named(n),
                Id::Usize(u) => UvId::Index(u)
            }
        }
    }

    impl From<Id> for NormalId {
        fn from(value: Id) -> Self {
            match value {
                Id::String(n) => NormalId::Named(n),
                Id::Usize(u) => NormalId::Index(u)
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub(crate) enum IdInner {
        Full([Id; 3]),
        Uv([Id; 2]),
        Bare(Id)
    }

    impl From<&VertRefData> for IdInner {
        fn from(value: &VertRefData) -> Self {
            match value {
                VertRefData::Full(vert_id, uv_id, normal_id) => {
                    IdInner::Full([Id::from(vert_id), Id::from(uv_id), Id::from(normal_id)])
                },
                VertRefData::WithUv(vert_id, uv_id) => {
                    IdInner::Uv([Id::from(vert_id), Id::from(uv_id)])
                },
                VertRefData::Bare(n) => IdInner::Bare(Id::from(n))
            }
        }
    }

    impl From<IdInner> for VertRefData {
        fn from(value: IdInner) -> Self {
            match value {
                IdInner::Full([v, u, n]) => VertRefData::Full(v.into(), u.into(), n.into()),
                IdInner::Uv([v, u]) => VertRefData::WithUv(v.into(), u.into()),
                IdInner::Bare(v) => VertRefData::Bare(v.into())
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
            if let Some(mat) = &t.mat {
                TriData::WithMat([
                    (&t.verts[0]).into(),
                    (&t.verts[1]).into(),
                    (&t.verts[2]).into(),
                    IdInner::Bare(Id::String(mat.to_string()))
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
                TriData::WithMat([a, b, c, IdInner::Bare(Id::String(mat))]) => {
                    TriangleData { verts: [a.into(), b.into(), c.into()], mat: Some(mat) }
                }
                TriData::Tri([a, b, c]) => {
                    TriangleData { verts: [a.into(), b.into(), c.into()], mat: None }
                }
                _ => unreachable!("Material can only be IdInner::Bare(Id::String)")
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

    #[test]
    fn test_deserialize() -> anyhow::Result<()> {

        let _setup: Value = serde_json::from_str(include_str!("../test_mesh.json"))?;

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
            parts: map!{
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
                                VertRefData::Bare(VertId::Named("v0".to_string())),
                                VertRefData::WithUv(VertId::Named("v1".to_string()), UvId::Index(1)),
                                VertRefData::WithUv(VertId::Named("v2".to_string()), UvId::Index(2)),
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
        };

        let json = serde_json::to_string(&value)?;

        println!("{json}");
        Ok(())
    }

}


