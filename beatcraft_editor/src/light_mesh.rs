use core::f32;
use std::collections::{HashMap, HashSet, hash_map};
use std::fs;
use std::hash::Hash;
use std::path::Path;

use anyhow::{Result, anyhow};
use glam::{FloatExt, Mat4, Quat, Vec2, Vec3};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;

use crate::RefDuper;
use crate::data::{
    BillboardData, ComputeNormalData, ComputeVertexData, LightMeshData, MaterialData, NormalId, PartData, PlacementData, ShaderSettingsData, StateSet, TriangleData, TriangleEntry, UvId, VertRefData, VertexId
};
use crate::easing::Easing;
use crate::editor::DataSwap;
use crate::renaming::light_mesh::rehash;
use crate::render::BillboardDesc;

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
    pub(crate) fn compute(&self, part: &Part) -> anyhow::Result<Vec3> {
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
    pub(crate) fn compute(&self, part: &Part) -> anyhow::Result<Vec3> {
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

    pub fn resolve_uv(&self, id: &UvId) -> Vec2 {
        match id {
            UvId::Index(i) => self.uvs.indexed.get(*i).copied().unwrap_or(Vec2::ZERO),
            UvId::Named(n) => self.uvs.named.get(n).copied().unwrap_or(Vec2::ZERO),
        }
    }

    pub fn resolve_uv_mut(&mut self, id: &UvId) -> Option<&mut Vec2> {
        match id {
            UvId::Index(i) => self.uvs.indexed.get_mut(*i),
            UvId::Named(n) => self.uvs.named.get_mut(n),
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
            if let Some(mat) = tri.material.as_mut()
                && *mat == swap.from
            {
                *mat = swap.to.clone();
            }
        }
    }

    /// if any deltas are negative, than ids > threshold are shifted down
    /// otherwise ids <= threshold are shifted up.
    pub fn cascade_ids(&mut self, threshold: usize, deltas: (i64, i64, i64)) {
        let negative = deltas.0 < 0 || deltas.1 < 0 || deltas.2 < 0;

        let shift = |id: &mut usize, delta: i64| {
            if negative {
                if *id > threshold {
                    *id = (*id as i64 + delta) as usize;
                }
            } else if *id <= threshold {
                *id += delta as usize
            }
        };

        for tri in self.triangles.0.iter_mut() {
            for vert in tri.vertices.iter_mut() {
                if let VertexId::Index(i) = &mut vert.vertex {
                    shift(i, deltas.0)
                }
                if let UvId::Index(i) = &mut vert.uv {
                    shift(i, deltas.1)
                }
                if let NormalId::Index(i) = &mut vert.normal {
                    shift(i, deltas.2)
                }
            }
        }
        for comp in self.vertices.compute.values_mut() {
            for id in comp.points.iter_mut() {
                if let VertexId::Index(i) = id {
                    shift(i, deltas.0)
                }
            }
        }
        for comp in self.normals.compute.values_mut() {
            for id in comp.points.iter_mut() {
                if let VertexId::Index(i) = id {
                    shift(i, deltas.0)
                }
            }
        }
    }

    pub fn contains_vertex(&self, id: &VertexId) -> bool {
        match id {
            VertexId::Index(i) => *i < self.vertices.indexed.len(),
            VertexId::Named(n) => {
                self.vertices.named.contains_key(n) || self.vertices.compute.contains_key(n)
            }
        }
    }

    pub fn contains_uv(&self, id: &UvId) -> bool {
        match id {
            UvId::Index(i) => *i < self.uvs.indexed.len(),
            UvId::Named(n) => self.uvs.named.contains_key(n),
        }
    }

    pub fn contains_normal(&self, id: &NormalId) -> bool {
        match id {
            NormalId::Index(i) => *i < self.normals.indexed.len(),
            NormalId::Named(n) => {
                self.normals.named.contains_key(n) || self.normals.compute.contains_key(n)
            }
        }
    }

    pub(crate) unsafe fn get_detached_vertex_refs<'a>(
        &mut self,
        lifeline: &'a mut RefDuper,
        target: usize,
    ) -> Vec<&'a mut VertexId> {
        let mut refs = Vec::new();

        for comp in self.vertices.compute.values_mut() {
            for id in comp.points.iter_mut() {
                if matches!(id, VertexId::Index(i) if *i == target) {
                    refs.push(unsafe { lifeline.detach_mut_ref(id) });
                }
            }
        }
        for comp in self.normals.compute.values_mut() {
            for id in comp.points.iter_mut() {
                if matches!(id, VertexId::Index(i) if *i == target) {
                    refs.push(unsafe { lifeline.detach_mut_ref(id) });
                }
            }
        }
        for tri in self.triangles.0.iter_mut() {
            for vert in tri.vertices.iter_mut() {
                if matches!(vert.vertex, VertexId::Index(i) if i == target) {
                    refs.push(unsafe { lifeline.detach_mut_ref(&mut vert.vertex) });
                }
            }
        }
        refs
    }

    pub(crate) unsafe fn get_detached_uv_refs<'a>(
        &mut self,
        lifeline: &'a mut RefDuper,
        target: usize,
    ) -> Vec<&'a mut UvId> {
        let mut refs = Vec::new();

        for tri in self.triangles.0.iter_mut() {
            for vert in tri.vertices.iter_mut() {
                if matches!(vert.uv, UvId::Index(i) if i == target) {
                    refs.push(unsafe { lifeline.detach_mut_ref(&mut vert.uv) });
                }
            }
        }
        refs
    }

    pub(crate) unsafe fn get_detached_normal_refs<'a>(
        &mut self,
        lifeline: &'a mut RefDuper,
        target: usize,
    ) -> Vec<&'a mut NormalId> {
        let mut refs = Vec::new();

        for tri in self.triangles.0.iter_mut() {
            for vert in tri.vertices.iter_mut() {
                if matches!(vert.normal, NormalId::Index(i) if i == target) {
                    refs.push(unsafe { lifeline.detach_mut_ref(&mut vert.normal) });
                }
            }
        }
        refs
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

    /// Iterates over triangles where all 3 vertices of the triangle are in `ids`
    pub fn filter_triangles(
        &mut self,
        ids: &[impl AsRef<VertexId>],
    ) -> impl Iterator<Item = &mut Triangle> {
        self.triangles.0.iter_mut().filter(|t| {
            t.vertices.iter().all(|v| {
                let ids: Vec<_> = ids.iter().map(AsRef::as_ref).collect();
                ids.contains(&&v.vertex)
            })
        })
    }

    /// Filters the input `ids` to only contain ids that are part of triangles.
    pub fn filter_triangle_vertices<'a>(
        &mut self,
        ids: &'a [VertexId],
    ) -> impl Iterator<Item = &'a VertexId> {
        let set: HashSet<&VertexId> = self
            .triangles
            .0
            .iter()
            .filter_map(|t| {
                if t.vertices.iter().all(|v| ids.contains(&v.vertex)) {
                    Some(ids.iter().filter(|id| {
                        **id == t.vertices[0].vertex
                            || **id == t.vertices[1].vertex
                            || **id == t.vertices[2].vertex
                    }))
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        set.into_iter()
    }

    /// Iterates over Vec3 positions for indexed/named vertices in `ids`
    pub fn filter_non_compute_vertices(
        &mut self,
        ids: &[&VertexId],
    ) -> impl Iterator<Item = &mut Vec3> {
        self.vertices
            .indexed
            .iter_mut()
            .enumerate()
            .filter_map(|(n, v)| {
                if ids.contains(&&VertexId::Index(n)) {
                    Some(v)
                } else {
                    None
                }
            })
            .chain(self.vertices.named.iter_mut().filter_map(|(n, v)| {
                if ids.contains(&&VertexId::Named(n.clone())) {
                    Some(v)
                } else {
                    None
                }
            }))
    }

    pub fn get_valid_vertex_ids(&self) -> impl Iterator<Item = VertexId> {
        (0..self.vertices.indexed.len())
            .map(VertexId::Index)
            .chain(
                self.vertices
                    .named
                    .keys()
                    .map(|n| VertexId::Named(n.clone())),
            )
            .chain(
                self.vertices
                    .compute
                    .keys()
                    .map(|c| VertexId::Named(c.clone())),
            )
    }

    pub fn get_valid_uv_ids(&self) -> impl Iterator<Item = UvId> {
        (0..self.uvs.indexed.len())
            .map(UvId::Index)
            .chain(self.uvs.named.keys().map(|n| UvId::Named(n.clone())))
    }

    pub fn get_valid_normal_ids(&self) -> impl Iterator<Item = NormalId> {
        (0..self.normals.indexed.len())
            .map(NormalId::Index)
            .chain(
                self.normals
                    .named
                    .keys()
                    .map(|n| NormalId::Named(n.clone())),
            )
            .chain(
                self.normals
                    .compute
                    .keys()
                    .map(|c| NormalId::Named(c.clone())),
            )
    }

    /// Deletes all listed vertices, and deletes all triangles that contain
    /// a deleted vertex, iteratively deletes compute vertices that reference deleted vertices
    pub fn delete_vertices<L, V>(&mut self, ids: L)
    where
        L: AsRef<[V]>,
        V: AsRef<VertexId>,
    {
        let ids = ids.as_ref();
        let mut ids: Vec<VertexId> = ids.iter().map(|i| i.as_ref().clone()).collect();
        let sentinel_name = unsafe { String::from_utf8_unchecked(vec![0]) };
        let sentinel = VertexId::Named(sentinel_name.clone());
        loop {
            match || -> Result<Vec<VertexId>> {
                let mut re_check = Vec::new();
                ids.sort_by(|a, b| match (a, b) {
                    (VertexId::Index(a), VertexId::Index(b)) => b.cmp(a),
                    (VertexId::Index(_), VertexId::Named(_)) => std::cmp::Ordering::Less,
                    (VertexId::Named(_), VertexId::Index(_)) => std::cmp::Ordering::Greater,
                    (VertexId::Named(a), VertexId::Named(b)) => b.cmp(a),
                });
                for id in ids {
                    self.rename_vertex(&DataSwap {
                        from: id,
                        to: sentinel.clone(),
                    })?;
                }
                let _ = self.vertices.named.shift_remove(&sentinel_name);
                let _ = self.vertices.compute.shift_remove(&sentinel_name);
                self.triangles
                    .0
                    .retain(|tri| tri.vertices.iter().all(|v| v.vertex != sentinel));
                for (name, vert) in self.vertices.compute.iter() {
                    if vert.points.contains(&sentinel) {
                        re_check.push(VertexId::Named(name.clone()));
                    }
                }
                self.normals
                    .compute
                    .retain(|_, c| c.points.iter().all(|v| *v != sentinel));
                Ok(re_check)
            }() {
                Err(_) => {
                    // cleanup
                    todo!("Add delete vertices sentinel cleanup")
                }
                Ok(next) => {
                    if next.is_empty() {
                        break;
                    }
                    if next.len() == 1
                        && let Some([a]) = next.as_array::<1>()
                        && *a == sentinel
                    {
                        break;
                    }
                    ids = next;
                }
            }
        }
    }

    /// Deletes all listed uvs, and sets references to Index(0)
    pub fn delete_uvs<L, U>(&mut self, ids: L)
    where
        L: AsRef<[U]>,
        U: AsRef<UvId>,
    {
        if || -> Result<()> {
            let sentinel_name = unsafe { String::from_utf8_unchecked(vec![0]) };
            let sentinel = UvId::Named(sentinel_name.clone());
            let ids = ids.as_ref();
            for id in ids {
                let id = id.as_ref().clone();
                self.rename_uv(&DataSwap {
                    from: id,
                    to: sentinel.clone(),
                })?;
            }
            let _ = self.uvs.named.shift_remove(&sentinel_name);
            self.triangles
                .0
                .retain(|tri| tri.vertices.iter().all(|v| v.uv != sentinel));
            Ok(())
        }()
        .is_err()
        {
            todo!("Add delete uvs sentinel cleanup")
        }
    }

    /// Deletes all listed normals, and sets references to Index(0)
    pub fn delete_normals<L, N>(&mut self, ids: L)
    where
        L: AsRef<[N]>,
        N: AsRef<NormalId>,
    {
        if || -> Result<()> {
            let sentinel_name = unsafe { String::from_utf8_unchecked(vec![0]) };
            let sentinel = NormalId::Named(sentinel_name.clone());
            let ids = ids.as_ref();
            for id in ids {
                let id = id.as_ref().clone();
                self.rename_normal(&DataSwap {
                    from: id,
                    to: sentinel.clone(),
                })?;
            }
            let _ = self.normals.named.shift_remove(&sentinel_name);
            let _ = self.normals.compute.shift_remove(&sentinel_name);
            self.triangles
                .0
                .retain(|tri| tri.vertices.iter().all(|v| v.normal != sentinel));
            Ok(())
        }()
        .is_err()
        {
            // cleanup
            todo!("Add delete vertices sentinel cleanup")
        }
    }

    pub fn delete_triangles_with_all_vertices<L, V>(&mut self, vertices: L)
    where
        L: AsRef<[V]>,
        V: AsRef<VertexId>,
    {
        let verts: Vec<_> = vertices.as_ref().iter().map(AsRef::as_ref).collect();
        self.triangles
            .0
            .retain(|tri| !tri.vertices.iter().all(|v| verts.contains(&&v.vertex)))
    }

    /// If the given vertices make up any triangles, the triangles are deleted,
    /// otherwise a triangle strip is created
    pub fn toggle_triangles(&mut self, vertices: &[VertexId], eye: Vec3) {
        if vertices.len() < 3 {
            return;
        }
        let tri_verts: Vec<_> = self.filter_triangle_vertices(vertices).collect();
        if !vertices.iter().all(|v| tri_verts.contains(&v)) {
            for i in 0..vertices.len() - 2 {
                let [a, b, c] = &vertices[i..i + 3] else {
                    unreachable!()
                };
                let va = self.resolve_vertex(a).unwrap();
                let vb = self.resolve_vertex(b).unwrap();
                let vc = self.resolve_vertex(c).unwrap();
                let mut a = a.clone();
                let mut c = c.clone();
                let ab = vb - va;
                let ac = vc - va;
                let normal = ab.cross(ac);
                if normal.dot(va - eye) > 0. {
                    std::mem::swap(&mut a, &mut c);
                }
                let tri = Triangle {
                    vertices: [
                        Vertex {
                            vertex: a.clone(),
                            uv: UvId::Index(0),
                            normal: NormalId::Index(0),
                        },
                        Vertex {
                            vertex: b.clone(),
                            uv: UvId::Index(0),
                            normal: NormalId::Index(0),
                        },
                        Vertex {
                            vertex: c.clone(),
                            uv: UvId::Index(0),
                            normal: NormalId::Index(0),
                        },
                    ],
                    material: None,
                };
                self.triangles.0.push(tri);
            }
        } else {
            self.delete_triangles_with_all_vertices(tri_verts);
        }
    }

    pub fn flip_triangles(&mut self, vertices: &[VertexId]) {
        for tri in self.filter_triangles(vertices) {
            let [a, _, c] = &mut tri.vertices;
            std::mem::swap(a, c);
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
    pub billboard: Option<BillboardData>,
    pub shader_settings: Option<ShaderSettingsData>,
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
            billboard: value.billboard,
            shader_settings: value.shader_settings,
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
            billboard: value.billboard,
            shader_settings: value.shader_settings,
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
        let data = std::mem::take(&mut self.data);
        self.data = rehash(data);
        for part in self.parts.values_mut() {
            part.rename_data(swap);
        }
    }

    pub fn resolve_data(&self, id: Option<&String>) -> Option<&MaterialData> {
        self.data.get(id.unwrap_or(&"default".to_string()))
    }

    pub fn rename_part(&mut self, swap: &DataSwap<String>) {
        for (name, _) in self.parts.iter_mut2() {
            if *name == swap.from {
                *name = swap.to.clone();
            }
        }
        let parts = std::mem::take(&mut self.parts);
        self.parts = rehash(parts);
        for placement in self.placements.iter_mut() {
            if placement.part == swap.from {
                placement.part = swap.to.clone();
            }
        }
        for name in self.part_names.iter_mut() {
            if *name == swap.from {
                *name = swap.to.clone();
            }
        }
    }

    pub fn rebuild(&mut self) {
        let part_names = self.parts.keys().cloned().collect();
        self.part_names = part_names;
    }

    pub fn snapshot_mesh_meta(&self, id: String) -> LightMeshMetaSnapshot {
        LightMeshMetaSnapshot {
            id,
            credits: self.credits.clone(),
            textures: self.textures.clone(),
            data: self.data.clone(),
            cull: self.cull,
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
pub struct LightMeshSnapshot {
    pub id: String,
    pub mesh: Box<LightMesh>,
}

#[derive(Clone, Debug)]
pub struct LightMeshPartSnapshot {
    pub id: String,
    pub name: String,
    pub part: Box<Part>,
}

#[derive(Clone, Debug)]
pub struct LightMeshPlacementSnapshot {
    pub view_id: String,
    pub placements: Vec<Placement>,
}

#[derive(Clone, Debug)]
pub struct LightMeshMetaSnapshot {
    pub id: String,
    pub credits: Vec<String>,
    pub textures: IndexMap<String, String>,
    pub data: IndexMap<String, MaterialData>,
    pub cull: bool,
}
