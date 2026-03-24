use core::f32;
use std::collections::{HashMap, hash_map};
use std::fs;
use std::hash::Hash;
use std::path::Path;

use anyhow::anyhow;
use glam::{FloatExt, Mat4, Quat, Vec2, Vec3};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;

use crate::data::{
    ComputeNormalData, ComputeVertexData, LightMeshData, MaterialData, 
    NormalId, PartData, PlacementData, StateSet, TriangleData,
    TriangleEntry, UvId, VertRefData, VertexId
};
use crate::easing::Easing;
use crate::editor::DataSwap;

#[derive(Debug, Clone)]
pub struct ComputeVertex {
    pub points: [VertexId; 2],
    pub function: Easing,
    pub delta: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

impl From<ComputeVertexData> for ComputeVertex {
    fn from(value: ComputeVertexData) -> Self {
        Self {
            points: value.points,
            function: value.function,
            delta: value.delta,
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<ComputeVertex> for ComputeVertexData {
    fn from(value: ComputeVertex) -> Self {
        Self {
            points: value.points,
            function: value.function,
            delta: value.delta,
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComputeNormal {
    pub points: [VertexId; 3],
}

impl From<ComputeNormalData> for ComputeNormal {
    fn from(value: ComputeNormalData) -> Self {
        Self {
            points: value.points,
        }
    }
}

impl From<ComputeNormal> for ComputeNormalData {
    fn from(value: ComputeNormal) -> Self {
        Self {
            points: value.points,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Vertices {
    pub indexed: Vec<Vec3>,
    pub named: IndexMap<String, Vec3>,
    pub compute: IndexMap<String, ComputeVertex>,
}

impl Vertices {
    pub fn get_vec(&self, part: &Part, include_compute: bool) -> Vec<(VertexId, Vec3)> {
        let mut out = Vec::new();

        for (id, vert) in self.indexed.iter().enumerate() {
            out.push((VertexId::Index(id), *vert))
        }

        for (name, vert) in self.named.iter() {
            out.push((VertexId::Named(name.clone()), *vert));
        }

        if include_compute {
            for (name, comp) in self.compute.iter() {
                out.push((VertexId::Named(name.clone()), comp.compute(part).unwrap()))
            }
        }

        out
    }

    pub fn get_mut_vec(&mut self) -> Vec<(VertexId, &mut Vec3)> {
        let mut out = Vec::new();
        for (id, vert) in self.indexed.iter_mut().enumerate() {
            out.push((VertexId::Index(id), vert));
        }

        for (name, vert) in self.named.iter_mut() {
            out.push((VertexId::Named(name.clone()), vert))
        }

        out
    }

}

impl
    From<(
        Vec<Vec3>,
        IndexMap<String, Vec3>,
        IndexMap<String, ComputeVertexData>,
    )> for Vertices
{
    fn from(
        value: (
            Vec<Vec3>,
            IndexMap<String, Vec3>,
            IndexMap<String, ComputeVertexData>,
        ),
    ) -> Self {
        Self {
            indexed: value.0,
            named: value.1,
            compute: value.2.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl From<Vertices>
    for (
        Vec<Vec3>,
        IndexMap<String, Vec3>,
        IndexMap<String, ComputeVertexData>,
    )
{
    fn from(value: Vertices) -> Self {
        (
            value.indexed,
            value.named,
            value
                .compute
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct UVs {
    pub indexed: Vec<Vec2>,
    pub named: IndexMap<String, Vec2>,
}

impl From<(Vec<Vec2>, IndexMap<String, Vec2>)> for UVs {
    fn from(value: (Vec<Vec2>, IndexMap<String, Vec2>)) -> Self {
        Self {
            indexed: value.0,
            named: value.1,
        }
    }
}

impl From<UVs> for (Vec<Vec2>, IndexMap<String, Vec2>) {
    fn from(value: UVs) -> Self {
        (value.indexed, value.named)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Normals {
    pub indexed: Vec<Vec3>,
    pub named: IndexMap<String, Vec3>,
    pub compute: IndexMap<String, ComputeNormal>,
}

impl
    From<(
        Vec<Vec3>,
        IndexMap<String, Vec3>,
        IndexMap<String, ComputeNormalData>,
    )> for Normals
{
    fn from(
        value: (
            Vec<Vec3>,
            IndexMap<String, Vec3>,
            IndexMap<String, ComputeNormalData>,
        ),
    ) -> Self {
        Self {
            indexed: value.0,
            named: value.1,
            compute: value.2.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl From<Normals>
    for (
        Vec<Vec3>,
        IndexMap<String, Vec3>,
        IndexMap<String, ComputeNormalData>,
    )
{
    fn from(value: Normals) -> Self {
        (
            value.indexed,
            value.named,
            value
                .compute
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub vertex: VertexId,
    pub uv: UvId,
    pub normal: NormalId,
}

impl VertRefData {
    pub(crate) fn resolve(self, defaults: &(UvId, NormalId)) -> Vertex {
        let (v, u, n) = self.take_all();
        Vertex {
            vertex: v,
            uv: u.unwrap_or_else(|| defaults.0.clone()),
            normal: n.unwrap_or_else(|| defaults.1.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub material: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Triangles(pub Vec<Triangle>);

impl From<Vec<TriangleEntry>> for Triangles {
    fn from(value: Vec<TriangleEntry>) -> Self {
        let mut defaults = (UvId::Index(0), NormalId::Index(0));
        let mut tris = Vec::new();

        for tri in value {
            match tri {
                TriangleEntry::StateSet(StateSet { uv, normal }) => {
                    if let Some(uv) = uv {
                        defaults.0 = uv
                    };
                    if let Some(normal) = normal {
                        defaults.1 = normal
                    };
                }
                TriangleEntry::Triangle(TriangleData {
                    verts: [a, b, c],
                    mat,
                }) => tris.push(Triangle {
                    vertices: [
                        a.resolve(&defaults),
                        b.resolve(&defaults),
                        c.resolve(&defaults),
                    ],
                    material: mat,
                }),
            }
        }

        Self(tris)
    }
}

impl From<Triangles> for Vec<TriangleEntry> {
    fn from(value: Triangles) -> Self {
        let mut sections = IndexMap::new();

        // Data size optimization loop
        for Triangle {
            vertices:
                [
                    Vertex {
                        vertex: v0,
                        uv: u0,
                        normal: n0,
                    },
                    Vertex {
                        vertex: v1,
                        uv: u1,
                        normal: n1,
                    },
                    Vertex {
                        vertex: v2,
                        uv: u2,
                        normal: n2,
                    },
                ],
            material,
        } in value.0
        {
            let matching_normals = n0 == n1 && n1 == n2;
            let matching_uvs = u0 == u1 && u1 == u2;
            if matching_uvs && matching_normals {
                let key = (u0.clone(), n0.clone());
                if !sections.contains_key(&key) {
                    sections.insert(key.clone(), Vec::new());
                }
                let list = unsafe { sections.get_mut(&key).unwrap_unchecked() };

                list.push(TriangleData {
                    verts: [
                        VertRefData::Bare(v0),
                        VertRefData::Bare(v1),
                        VertRefData::Bare(v2),
                    ],
                    mat: material,
                });
            } else if !matching_uvs && matching_normals {
                let common = if u1 == u2 { &u1 } else { &u0 };

                let key = (common.clone(), n0.clone());
                if !sections.contains_key(&key) {
                    sections.insert(key.clone(), Vec::new());
                }
                let list = unsafe { sections.get_mut(&key).unwrap_unchecked() };

                let v2 = if u2 == *common {
                    VertRefData::Bare(v2)
                } else {
                    VertRefData::WithUv(v2, u2)
                };
                let v0_m = u0 == *common;
                let v1 = if u1 == *common {
                    VertRefData::Bare(v1)
                } else {
                    VertRefData::WithUv(v1, u1)
                };
                let v0 = if v0_m {
                    VertRefData::Bare(v0)
                } else {
                    VertRefData::WithUv(v0, u0)
                };

                list.push(TriangleData {
                    verts: [v0, v1, v2],
                    mat: material,
                });
            } else {
                let uc = if u1 == u2 { &u1 } else { &u0 };
                let nc = if n1 == n2 { &n1 } else { &n0 };

                let key = (uc.clone(), nc.clone());
                if !sections.contains_key(&key) {
                    sections.insert(key.clone(), Vec::new());
                }
                let list = unsafe { sections.get_mut(&key).unwrap_unchecked() };

                let v2 = if u2 == *uc {
                    if n2 == *nc {
                        VertRefData::Bare(v2)
                    } else {
                        VertRefData::Full(v2, u2, n2)
                    }
                } else if n2 == *nc {
                    VertRefData::WithUv(v2, u2)
                } else {
                    VertRefData::Full(v2, u2, n2)
                };
                let v0_u = u0 == *uc;
                let v0_n = n0 == *nc;
                let v1 = if u1 == *uc {
                    if n1 == *nc {
                        VertRefData::Bare(v1)
                    } else {
                        VertRefData::Full(v1, u1, n1)
                    }
                } else if n1 == *nc {
                    VertRefData::WithUv(v1, u1)
                } else {
                    VertRefData::Full(v1, u1, n1)
                };
                let v0 = if v0_u {
                    if v0_n {
                        VertRefData::Bare(v0)
                    } else {
                        VertRefData::Full(v0, u0, n0)
                    }
                } else if v0_n {
                    VertRefData::WithUv(v0, u0)
                } else {
                    VertRefData::Full(v0, u0, n0)
                };
                list.push(TriangleData {
                    verts: [v0, v1, v2],
                    mat: material,
                });
            }
        }

        // Final data packing
        let mut entries = Vec::new();
        let mut current = (UvId::Index(0), NormalId::Index(0));

        for (section, tris) in sections {
            let uv = if section.0 == current.0 {
                None
            } else {
                current.0 = section.0.clone();
                Some(section.0)
            };
            let normal = if section.1 == current.1 {
                None
            } else {
                current.1 = section.1.clone();
                Some(section.1)
            };

            let set = StateSet { uv, normal };

            entries.push(TriangleEntry::StateSet(set));

            for tri in tris {
                entries.push(TriangleEntry::Triangle(tri));
            }
        }

        entries
    }
}

#[derive(Copy, Clone, Debug)]
struct HashableVec3(Vec3);
#[derive(Copy, Clone, Debug)]
struct HashableVec2(Vec2);
impl ComputeVertex {
    fn as_hashable(&self) -> (&[VertexId; 2], Easing, [Option<u32>; 4]) {
        (
            &self.points,
            self.function,
            [
                self.delta.map(f32::to_bits),
                self.x.map(f32::to_bits),
                self.y.map(f32::to_bits),
                self.z.map(f32::to_bits),
            ],
        )
    }
}
impl Hash for ComputeVertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_hashable().hash(state)
    }
}
impl PartialEq for ComputeVertex {
    fn eq(&self, other: &Self) -> bool {
        self.as_hashable().eq(&other.as_hashable())
    }
}
impl Eq for ComputeVertex {}

impl HashableVec2 {
    fn as_hashable(&self) -> [u32; 2] {
        [self.0.x.to_bits(), self.0.y.to_bits()]
    }
}
impl Hash for HashableVec2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_hashable().hash(state);
    }
}
impl PartialEq for HashableVec2 {
    fn eq(&self, other: &Self) -> bool {
        self.as_hashable().eq(&other.as_hashable())
    }
}
impl Eq for HashableVec2 {}

impl HashableVec3 {
    fn as_hashable(&self) -> [u32; 3] {
        [self.0.x.to_bits(), self.0.y.to_bits(), self.0.y.to_bits()]
    }
}
impl Hash for HashableVec3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_hashable().hash(state);
    }
}
impl PartialEq for HashableVec3 {
    fn eq(&self, other: &Self) -> bool {
        self.as_hashable().eq(&other.as_hashable())
    }
}
impl Eq for HashableVec3 {}

impl From<Vec2> for HashableVec2 {
    fn from(value: Vec2) -> Self {
        Self(value)
    }
}
impl From<HashableVec2> for Vec2 {
    fn from(value: HashableVec2) -> Self {
        value.0
    }
}
impl From<Vec3> for HashableVec3 {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}
impl From<HashableVec3> for Vec3 {
    fn from(value: HashableVec3) -> Self {
        value.0
    }
}

impl ComputeVertex {
    fn compute(&self, part: &Part) -> anyhow::Result<Vec3> {
        let a = part.resolve_vertex(&self.points[0])?;
        let b = part.resolve_vertex(&self.points[1])?;

        let dt = self.delta.unwrap_or(0.);
        let dt = self.function.apply(dt);
        let mut c = a.lerp(b, dt);

        let delta = b - a;
        let vx = if delta.x == 0. { 0. } else { delta.x.signum() };
        let vy = if delta.y == 0. { 0. } else { delta.y.signum() };
        let vz = if delta.z == 0. { 0. } else { delta.z.signum() };

        if let Some(x) = self.x {
            c.x = a.x + x * vx;
            let dx = f32::inverse_lerp(0., b.x - a.x, vx * x);
            if self.y.is_none() {
                c.y = a.y.lerp(b.y, dx);
            }
            if self.z.is_none() {
                c.z = a.z.lerp(b.z, dx);
            }
        }
        if let Some(y) = self.y {
            c.y = a.y + y * vy;
            let dy = f32::inverse_lerp(0., b.y - a.y, vy * y);
            if self.x.is_none() {
                c.x = a.x.lerp(b.x, dy);
            }
            if self.z.is_none() {
                c.z = a.z.lerp(b.z, dy);
            }
        }
        if let Some(z) = self.z {
            c.z = a.z + z * vz;
            let dz = f32::inverse_lerp(0., b.z - a.z, vz * z);
            if self.x.is_none() {
                c.x = a.x.lerp(b.x, dz);
            }
            if self.y.is_none() {
                c.y = a.y.lerp(b.y, dz);
            }
        }
        Ok(c)
    }
}

impl ComputeNormal {
    fn compute(&self, part: &Part) -> anyhow::Result<Vec3> {
        let a = part.resolve_vertex(&self.points[0])?;
        let b = part.resolve_vertex(&self.points[1])?;
        let c = part.resolve_vertex(&self.points[2])?;

        let ab = b - a;
        let ac = c - a;

        Ok(ab.cross(ac).normalize())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Part {
    pub vertices: Vertices,
    pub uvs: UVs,
    pub normals: Normals,
    pub triangles: Triangles,
}

impl Part {
    pub fn resolve_vertex(&self, id: &VertexId) -> anyhow::Result<Vec3> {
        match id {
            VertexId::Index(i) => self.vertices.indexed.get(*i).copied().ok_or_else(|| {
                anyhow!(
                    "Invalid index {i} for Vertices of length {}",
                    self.vertices.indexed.len()
                )
            }),
            VertexId::Named(n) => {
                if let Some(v) = self.vertices.named.get(n) {
                    Ok(*v)
                } else {
                    self.vertices
                        .compute
                        .get(n)
                        .ok_or_else(|| anyhow!("Invalid name '{n}' for Vertices"))?
                        .compute(self)
                }
            }
        }
    }

    pub fn resolve_normal(&self, id: &NormalId) -> anyhow::Result<Vec3> {
        match id {
            NormalId::Index(i) => Ok(self.normals.indexed.get(*i).copied().unwrap_or(Vec3::Y)),
            NormalId::Named(n) => {
                if let Some(v) = self.normals.named.get(n) {
                    Ok(*v)
                } else if let Some(v) = self.normals.compute.get(n) {
                    v.compute(self)
                } else {
                    Ok(Vec3::Y)
                }
            }
        }
    }

    pub fn rename_data(&mut self, swap: &DataSwap<String>) {
        for tri in self.triangles.0.iter_mut() {
            if let Some(mat) = tri.material.as_mut() && *mat == swap.from {
                *mat = swap.to.clone();
            }
        }
    }

    pub fn dedupe_data(&mut self) {
        let mut v_index_remap = HashMap::new();
        let mut v_index_updated = Vec::new();
        let mut v_index_backmap = HashMap::new();
        let mut v_named_remap = HashMap::new();
        let mut v_named_updated = IndexMap::new();
        let mut v_named_backmap = HashMap::new();
        let mut v_comp_remap = HashMap::new();
        let mut v_comp_updated = IndexMap::new();
        let mut v_comp_backmap = HashMap::new();

        let mut u_index_remap = HashMap::new();
        let mut u_index_updated = Vec::new();
        let mut u_index_backmap = HashMap::new();
        let mut u_named_remap = HashMap::new();
        let mut u_named_updated = IndexMap::new();
        let mut u_named_backmap = HashMap::new();

        let mut n_index_remap = HashMap::new();
        let mut n_index_updated = Vec::new();
        let mut n_index_backmap = HashMap::new();
        let mut n_named_remap = HashMap::new();
        let mut n_named_updated = IndexMap::new();
        let mut n_named_backmap = HashMap::new();
        let mut n_comp_remap = HashMap::new();
        let mut n_comp_updated = IndexMap::new();
        let mut n_comp_backmap = HashMap::new();

        for (i, v) in self.vertices.indexed.iter().enumerate() {
            let h: HashableVec3 = (*v).into();
            match v_index_remap.entry(h) {
                hash_map::Entry::Vacant(e) => {
                    v_index_updated.push(v);
                    e.insert(v_index_updated.len() - 1);
                }
                hash_map::Entry::Occupied(e) => {
                    v_index_backmap.insert(i, *e.get());
                }
            }
        }
        for (k, v) in self.vertices.named.iter() {
            let h: HashableVec3 = (*v).into();
            match v_named_remap.entry(h) {
                hash_map::Entry::Vacant(e) => {
                    v_named_updated.insert(k.clone(), v);
                    e.insert(k.clone());
                }
                hash_map::Entry::Occupied(e) => {
                    v_named_backmap.insert(k.clone(), e.get().clone());
                }
            }
        }
        for (k, v) in self.vertices.compute.iter() {
            match v_comp_remap.entry(v) {
                hash_map::Entry::Vacant(e) => {
                    v_comp_updated.insert(k.clone(), v);
                    e.insert(k.clone());
                }
                hash_map::Entry::Occupied(e) => {
                    v_comp_backmap.insert(k.clone(), e.get().clone());
                }
            }
        }

        for (i, v) in self.uvs.indexed.iter().enumerate() {
            let h: HashableVec2 = (*v).into();
            match u_index_remap.entry(h) {
                hash_map::Entry::Vacant(e) => {
                    u_index_updated.push(v);
                    e.insert(u_index_updated.len() - 1);
                }
                hash_map::Entry::Occupied(e) => {
                    u_index_backmap.insert(i, *e.get());
                }
            }
        }
        for (k, v) in self.uvs.named.iter() {
            let h: HashableVec2 = (*v).into();
            match u_named_remap.entry(h) {
                hash_map::Entry::Vacant(e) => {
                    u_named_updated.insert(k.clone(), v);
                    e.insert(k.clone());
                }
                hash_map::Entry::Occupied(e) => {
                    u_named_backmap.insert(k.clone(), e.get().clone());
                }
            }
        }

        for (i, v) in self.normals.indexed.iter().enumerate() {
            let h: HashableVec3 = (*v).into();
            match n_index_remap.entry(h) {
                hash_map::Entry::Vacant(e) => {
                    n_index_updated.push(v);
                    e.insert(n_index_updated.len() - 1);
                }
                hash_map::Entry::Occupied(e) => {
                    n_index_backmap.insert(i, *e.get());
                }
            }
        }
        for (k, v) in self.normals.named.iter() {
            let h: HashableVec3 = (*v).into();
            match n_named_remap.entry(h) {
                hash_map::Entry::Vacant(e) => {
                    n_named_updated.insert(k.clone(), v);
                    e.insert(k.clone());
                }
                hash_map::Entry::Occupied(e) => {
                    n_named_backmap.insert(k.clone(), e.get().clone());
                }
            }
        }
        for (k, v) in self.normals.compute.iter() {
            match n_comp_remap.entry(v) {
                hash_map::Entry::Vacant(e) => {
                    n_comp_updated.insert(k.clone(), v);
                    e.insert(k.clone());
                }
                hash_map::Entry::Occupied(e) => {
                    n_comp_backmap.insert(k.clone(), e.get().clone());
                }
            }
        }

        for tri in self.triangles.0.iter_mut() {
            tri.remap(
                &v_index_backmap,
                &v_named_backmap,
                &v_comp_backmap,
                &u_index_backmap,
                &u_named_backmap,
                &n_index_backmap,
                &n_named_backmap,
                &n_comp_backmap,
            );
        }
    }
}

impl Triangle {
    #[allow(clippy::too_many_arguments)]
    fn remap(
        &mut self,
        vi: &HashMap<usize, usize>,
        vn: &HashMap<String, String>,
        vc: &HashMap<String, String>,
        ui: &HashMap<usize, usize>,
        un: &HashMap<String, String>,
        ni: &HashMap<usize, usize>,
        nn: &HashMap<String, String>,
        nc: &HashMap<String, String>,
    ) {
        for v in self.vertices.iter_mut() {
            match &mut v.vertex {
                VertexId::Index(i) => *i = *vi.get(i).unwrap_or(i),
                VertexId::Named(n) => *n = vn.get(n).or_else(|| vc.get(n)).unwrap_or(n).clone(),
            }
            match &mut v.uv {
                UvId::Index(i) => *i = *ui.get(i).unwrap_or(i),
                UvId::Named(n) => *n = un.get(n).unwrap_or(n).clone(),
            }
            match &mut v.normal {
                NormalId::Index(i) => *i = *ni.get(i).unwrap_or(i),
                NormalId::Named(n) => *n = nn.get(n).or_else(|| nc.get(n)).unwrap_or(n).clone(),
            }
        }
    }
}

impl From<PartData> for Part {
    fn from(value: PartData) -> Self {
        Self {
            vertices: (value.vertices, value.named_vertices, value.compute_vertices).into(),
            uvs: (value.uvs, value.named_uvs).into(),
            normals: (value.normals, value.named_normals, value.compute_normals).into(),
            triangles: value.triangles.into(),
        }
    }
}

impl From<Part> for PartData {
    fn from(value: Part) -> Self {
        let (vertices, named_vertices, compute_vertices) = value.vertices.into();
        let (uvs, named_uvs) = value.uvs.into();
        let (normals, named_normals, compute_normals) = value.normals.into();

        Self {
            vertices,
            named_vertices,
            compute_vertices,
            uvs,
            named_uvs,
            normals,
            named_normals,
            compute_normals,
            triangles: value.triangles.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Placement {
    pub part: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub remap_data: IndexMap<String, String>,
}

impl Placement {
    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}

impl From<PlacementData> for Placement {
    fn from(value: PlacementData) -> Self {
        Self {
            part: value.part,
            position: value.position,
            rotation: value.rotation,
            scale: value.scale,
            remap_data: value.remap_data,
        }
    }
}

impl From<Placement> for PlacementData {
    fn from(value: Placement) -> Self {
        Self {
            part: value.part,
            position: value.position,
            rotation: value.rotation,
            scale: value.scale,
            remap_data: value.remap_data,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum BloomfogStyle {
    #[default]
    BloomOnly = 0,
    Everything = 1,
    Nothing = 2,
}

impl BloomfogStyle {
    pub fn label(&self) -> &'static str {
        match self {
            BloomfogStyle::BloomOnly => "Bloom Only",
            BloomfogStyle::Everything => "Everything",
            BloomfogStyle::Nothing => "Nothing",
        }
    }
}

impl From<u8> for BloomfogStyle {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::BloomOnly,
            1 => Self::Everything,
            _ => Self::Nothing,
        }
    }
}

impl From<BloomfogStyle> for u8 {
    fn from(value: BloomfogStyle) -> Self {
        match value {
            BloomfogStyle::BloomOnly => 0,
            BloomfogStyle::Everything => 1,
            BloomfogStyle::Nothing => 2,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LightMesh {
    pub credits: Vec<String>,
    pub parts: IndexMap<String, Part>,
    pub placements: Vec<Placement>,
    pub textures: IndexMap<String, String>,
    pub data: IndexMap<String, MaterialData>,
    pub cull: bool,
    pub do_bloom: bool,
    pub do_mirroring: bool,
    pub bloomfog_style: BloomfogStyle,
    pub do_solid: bool,
    pub part_names: Vec<String>,
}

impl LightMesh {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let raw: LightMeshData = serde_json::from_str(&raw)?;
        Ok(raw.into())
    }

    pub fn rename_data(&mut self, swap: &DataSwap<String>) {
        for (key, _) in self.data.iter_mut2() {
            if *key == swap.from {
                *key = swap.to.clone();
                break;
            }
        }
        for part in self.parts.values_mut() {
            part.rename_data(swap);
        }
    }

    pub fn rename_part(&mut self, swap: &DataSwap<String>) {
        for (name, _) in self.parts.iter_mut2() {
            if *name == swap.from {
                *name = swap.to.clone();
            }
        }
        for placement in self.placements.iter_mut() {
            if placement.part == swap.from {
                placement.part = swap.to.clone();
            }
        }
    }

}

impl From<crate::data::LightMeshData> for LightMesh {
    fn from(value: crate::data::LightMeshData) -> Self {
        let parts: IndexMap<String, Part> = value
            .parts
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        let part_names = parts.keys().cloned().collect();
        Self {
            credits: value.credits,
            parts,
            placements: value.mesh.into_iter().map(Into::into).collect(),
            textures: value.textures,
            data: value.data,
            cull: value.cull,
            do_bloom: value.bloom_pass,
            do_mirroring: value.mirror_pass,
            do_solid: value.solid_pass,
            bloomfog_style: value.bloomfog_style.into(),
            part_names,
        }
    }
}

impl From<LightMesh> for crate::data::LightMeshData {
    fn from(value: LightMesh) -> Self {
        Self {
            mesh_format: 1,
            credits: value.credits,
            parts: value
                .parts
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            mesh: value.placements.into_iter().map(Into::into).collect(),
            textures: value.textures,
            data: value.data,
            cull: value.cull,
            bloom_pass: value.do_bloom,
            mirror_pass: value.do_mirroring,
            solid_pass: value.do_solid,
            bloomfog_style: value.bloomfog_style.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LightMeshPartSnapshot {
    pub idx: usize,
    pub name: String,
    pub part: Box<Part>,
}

#[derive(Clone, Debug)]
pub struct LightMeshPlacementSnapshot {
    pub view_idx: usize,
    pub placements: Vec<Placement>,
}

#[derive(Clone, Debug)]
pub struct LightMeshMetaSnapshot {
    pub idx: usize,
    pub credits: Vec<String>,
    pub textures: IndexMap<String, String>,
    pub data: IndexMap<String, MaterialData>,
    pub cull: bool,
}
