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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TriangleData {
    #[serde(with = "vert_ref_serde")]
    pub a: VertRefData,
    #[serde(with = "vert_ref_serde")]
    pub b: VertRefData,
    #[serde(with = "vert_ref_serde")]
    pub c: VertRefData,
    pub mat: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum TriangleEntry {
    Triangle(TriangleData),
    StateSet {
        uv: Option<UvId>,
        normal: Option<NormalId>,
    }
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
pub struct ComputeNormalData {

}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct PartData {
    vertices: Vec<Vec3>,
    named_vertices: IndexMap<String, Vec3>,
    compute_vertices: IndexMap<String, ComputeVertexData>,
    uvs: Vec<Vec2>,
    named_uvs: IndexMap<String, Vec2>,
    normals: Vec<Vec3>,
    named_normals: IndexMap<String, Vec3>,
    compute_normals: IndexMap<String, ComputeNormalData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlacementData {
    part: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    position: Option<Vec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rotation: Option<Quat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    normal: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct MaterialData {
    material: usize,
    texture: usize,
    color: usize,
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
}


mod vert_ref_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{NormalId, UvId, VertId, VertRefData};

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    enum Id {
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

    pub(crate) fn serialize<S: Serializer>(v: &VertRefData, s: S) -> Result<S::Ok, S::Error> {
        match v {
            VertRefData::Full(vert_id, uv_id, normal_id) => {
                vec![
                    Id::from(vert_id),
                    uv_id.into(),
                    normal_id.into(),
                ].serialize(s)
            },
            VertRefData::WithUv(vert_id, uv_id) => {
                vec![
                    Id::from(vert_id),
                    uv_id.into(),
                ].serialize(s)
            },
            VertRefData::Bare(vert_id) => vert_id.serialize(s),
        }
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IdInner {
        Full([Id; 3]),
        Uv([Id; 2]),
        Bare(Id)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<VertRefData, D::Error> {
        Ok(match IdInner::deserialize(d)? {
            IdInner::Full([v, u, n]) => VertRefData::Full(v.into(), u.into(), n.into()),
            IdInner::Uv([v, u]) => VertRefData::WithUv(v.into(), u.into()),
            IdInner::Bare(v) => VertRefData::Bare(v.into())
        })
    }
}



#[cfg(test)]
mod data_tests {
    #[test]
    fn test_deserialize() -> anyhow::Result<()> {

        let data = include_str!("../test_mesh.json");


        Ok(())
    }

}


