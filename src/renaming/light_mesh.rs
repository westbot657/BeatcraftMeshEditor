use std::mem;

use crate::RefDuper;
use crate::data::{NormalId, UvId, VertexId};
use crate::editor::DataSwap;
use crate::light_mesh::{LightMesh, Part};
use anyhow::{Result, anyhow};
use indexmap::IndexMap;
use indexmap::map::MutableKeys;

fn rehash<T>(map: IndexMap<String, T>) -> IndexMap<String, T> {
    map.into_iter().collect()
}

impl LightMesh {
    pub fn rename_vertex(&mut self, part: &str, swap: &DataSwap<VertexId>) -> Result<()> {
        if let Some(part) = self.parts.get_mut(part) {
            if !part.contains_vertex(&swap.from) {
                return Err(anyhow!("vertex is not present"));
            }

            println!("pre-rename: {part:#?}");

            let update_vertices = |part: &mut Part| {
                match &swap.from {
                    VertexId::Index(_) => { /* Handled by to_update system */ }
                    VertexId::Named(n) => {
                        for tri in part.triangles.0.iter_mut() {
                            for vert in tri.vertices.iter_mut() {
                                if matches!(vert.vertex, VertexId::Named(ref n2) if *n == *n2) {
                                    vert.vertex = swap.to.clone();
                                }
                            }
                        }
                        for comp in part.vertices.compute.values_mut() {
                            for vert in comp.points.iter_mut() {
                                if matches!(vert, VertexId::Named(n2) if *n == *n2) {
                                    *vert = swap.to.clone();
                                }
                            }
                        }
                        for comp in part.normals.compute.values_mut() {
                            for vert in comp.points.iter_mut() {
                                if matches!(vert, VertexId::Named(n2) if *n == *n2) {
                                    *vert = swap.to.clone();
                                }
                            }
                        }
                    }
                }
            };

            match swap {
                DataSwap {
                    from: VertexId::Named(from),
                    to: VertexId::Named(to),
                } => {
                    for (name, _) in part.vertices.named.iter_mut2() {
                        if *name == *from {
                            *name = to.clone();
                        }
                    }
                    for (name, _) in part.vertices.compute.iter_mut2() {
                        if *name == *from {
                            *name = to.clone();
                        }
                    }
                    update_vertices(part);
                }
                DataSwap { from, to } => {
                    let mut rd = RefDuper;
                    let mut to_update = Vec::new();
                    let vertex = match from {
                        VertexId::Index(i) => {
                            let vertex = part.vertices.indexed.remove(*i);
                            to_update = unsafe { part.get_detached_vertex_refs(&mut rd, *i) };
                            part.cascade_ids(*i, (-1, 0, 0));
                            vertex
                        }
                        VertexId::Named(n) => {
                            if let Some(vert) = part.vertices.named.shift_remove(n.as_str()) {
                                vert
                            } else if let Some(vert) = part.vertices.compute.get(n.as_str()) {
                                let res = vert.compute(part)?;
                                part.vertices.compute.shift_remove(n.as_str());
                                res
                            } else {
                                unreachable!()
                            }
                        }
                    };
                    update_vertices(part);
                    match to {
                        VertexId::Index(i) => {
                            part.cascade_ids(*i, (1, 0, 0));
                            part.vertices.indexed.insert(*i, vertex);
                        }
                        VertexId::Named(n) => {
                            part.vertices.named.insert(n.clone(), vertex);
                        }
                    }
                    for id in to_update {
                        *id = to.clone();
                    }
                }
            }
            let named = mem::take(&mut part.vertices.named);
            part.vertices.named = rehash(named);
            let compute = mem::take(&mut part.vertices.compute);
            part.vertices.compute = rehash(compute);
        }
        Ok(())
    }

    pub fn rename_uv(&mut self, part: &str, swap: &DataSwap<UvId>) -> Result<()> {
        if let Some(part) = self.parts.get_mut(part) {
            if !part.contains_uv(&swap.from) {
                return Err(anyhow!("uv is not present"));
            }

            let update_uvs = |part: &mut Part| {
                match &swap.from {
                    UvId::Index(_) => { /* Handled by to_update system */ }
                    UvId::Named(n) => {
                        for tri in part.triangles.0.iter_mut() {
                            for vert in tri.vertices.iter_mut() {
                                if matches!(vert.uv, UvId::Named(ref n2) if *n == *n2) {
                                    vert.uv = swap.to.clone();
                                }
                            }
                        }
                    }
                }
            };

            match swap {
                DataSwap {
                    from: UvId::Named(from),
                    to: UvId::Named(to),
                } => {
                    for (name, _) in part.uvs.named.iter_mut2() {
                        if *name == *from {
                            *name = to.clone();
                        }
                    }
                    update_uvs(part);
                }
                DataSwap { from, to } => {
                    let mut rd = RefDuper;
                    let mut to_update = Vec::new();
                    let uv = match from {
                        UvId::Index(i) => {
                            let uv = part.uvs.indexed.remove(*i);
                            to_update = unsafe { part.get_detached_uv_refs(&mut rd, *i) };
                            part.cascade_ids(*i, (0, -1, 0));
                            uv
                        }
                        UvId::Named(n) => {
                            if let Some(uv) = part.uvs.named.shift_remove(n.as_str()) {
                                uv
                            } else {
                                unreachable!()
                            }
                        }
                    };
                    update_uvs(part);
                    match to {
                        UvId::Index(i) => {
                            part.cascade_ids(*i, (0, 1, 0));
                            part.uvs.indexed.insert(*i, uv);
                        }
                        UvId::Named(n) => {
                            part.uvs.named.insert(n.clone(), uv);
                        }
                    }
                    for id in to_update {
                        *id = to.clone();
                    }
                }
            }

            let named = mem::take(&mut part.uvs.named);
            part.uvs.named = rehash(named);
        }
        Ok(())
    }

    pub fn rename_normal(&mut self, part: &str, swap: &DataSwap<NormalId>) -> Result<()> {
        if let Some(part) = self.parts.get_mut(part) {
            if !part.contains_normal(&swap.from) {
                return Err(anyhow!("normal is not present"));
            }

            let update_normals = |part: &mut Part| {
                match &swap.from {
                    NormalId::Index(_) => { /* Handled by to_update system */ }
                    NormalId::Named(n) => {
                        for tri in part.triangles.0.iter_mut() {
                            for vert in tri.vertices.iter_mut() {
                                if matches!(vert.normal, NormalId::Named(ref n2) if *n == *n2) {
                                    vert.normal = swap.to.clone();
                                }
                            }
                        }
                    }
                }
            };

            match swap {
                DataSwap {
                    from: NormalId::Named(from),
                    to: NormalId::Named(to),
                } => {
                    for (name, _) in part.normals.named.iter_mut2() {
                        if *name == *from {
                            *name = to.clone();
                        }
                    }
                    update_normals(part);
                }
                DataSwap { from, to } => {
                    let mut rd = RefDuper;
                    let mut to_update = Vec::new();
                    let normal = match from {
                        NormalId::Index(i) => {
                            let normal = part.normals.indexed.remove(*i);
                            to_update = unsafe { part.get_detached_normal_refs(&mut rd, *i) };
                            part.cascade_ids(*i, (0, 0, -1));
                            normal
                        }
                        NormalId::Named(n) => {
                            if let Some(norm) = part.normals.named.shift_remove(n.as_str()) {
                                norm
                            } else {
                                unreachable!()
                            }
                        }
                    };
                    update_normals(part);
                    match to {
                        NormalId::Index(i) => {
                            part.cascade_ids(*i, (0, 0, 1));
                            part.normals.indexed.insert(*i, normal);
                        }
                        NormalId::Named(n) => {
                            part.normals.named.insert(n.clone(), normal);
                        }
                    }
                    for id in to_update {
                        *id = to.clone();
                    }
                }
            }
            let named = mem::take(&mut part.normals.named);
            part.normals.named = rehash(named);
            let compute = mem::take(&mut part.normals.compute);
            part.normals.compute = rehash(compute);
        }
        Ok(())
    }
}
