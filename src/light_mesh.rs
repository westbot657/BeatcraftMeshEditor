use glam::{Quat, Vec2, Vec3};
use indexmap::IndexMap;

use crate::data::MaterialData;
use crate::easing::Easing;

#[derive(Debug)]
pub struct ComputeVertex {
    pub points: [VertexId; 2],
    pub function: Easing,
    pub delta: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

#[derive(Debug)]
pub struct ComputeNormal {
    pub points: [VertexId; 3],
}

#[derive(Debug)]
pub struct Vertices {
    pub indexed: Vec<Vec3>,
    pub named: IndexMap<String, Vec3>,
    pub compute: IndexMap<String, ComputeVertex>,
}

#[derive(Debug)]
pub struct UVs {
    pub indexed: Vec<Vec2>,
    pub named: IndexMap<String, Vec2>,
}

#[derive(Debug)]
pub struct Normals {
    pub indexed: Vec<Vec3>,
    pub named: IndexMap<String, Vec3>,
    pub compute: IndexMap<String, ComputeNormal>,
}

#[derive(Debug)]
pub enum VertexId {
    Index(usize),
    Name(String),
}

#[derive(Debug)]
pub enum UVId {
    Index(usize),
    Name(String),
}

#[derive(Debug)]
pub enum NormalId {
    Index(usize),
    Name(String),
}

#[derive(Debug)]
pub struct Vertex {
    pub vertex: VertexId,
    pub uv: UVId,
    pub normal: NormalId,
}

#[derive(Debug)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub material: String
}


#[derive(Debug)]
pub struct Part {
    pub vertices: Vertices,
    pub uvs: UVs,
    pub normals: Normals,
    pub triangles: Vec<Triangle>
}

#[derive(Debug)]
pub struct Placement {
    pub part: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Default)]
pub struct LightMesh {
    pub credits: Vec<String>,
    pub parts: IndexMap<String, Part>,
    pub mesh: Vec<Placement>,
    pub textures: IndexMap<String, String>,
    pub data: IndexMap<String, MaterialData>,
    pub cull: bool,
}


