use std::collections::HashMap;
use std::f32;
use std::path::{Path, PathBuf};

use eframe::glow::{self, HasContext};
use glam::{FloatExt, IVec3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};
use indexmap::IndexMap;

use crate::{RefDuper, data};
use crate::data::MaterialData;
use crate::light_mesh::{LightMesh, Part, Triangle, Vertex};

static MISSING_TEXTURE_BYTES: &[u8] = include_bytes!("./assets/textures/missing.png");

pub static LIGHT_COLORS: [Vec4; 8] = [
    Vec4::new(0.55,0.70,1.00, 1.), Vec4::new(1.00,0.25,0.35, 1.),
    Vec4::new(0.15,0.95,0.45, 1.), Vec4::new(1.00,0.90,0.10, 1.),
    Vec4::new(0.20,0.50,1.00, 1.), Vec4::new(0.90,0.20,1.00, 1.),
    Vec4::new(0.10,0.95,0.95, 1.), Vec4::new(1.00,0.55,0.10, 1.)
];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub clipping_plane: Vec4,
    pub model: Mat4,
    pub colors: [Vec4; 8],
}

impl InstanceData {
    pub fn new(clipping_plane: Vec4, model: Mat4, colors: [Vec4; 8]) -> Self {
        Self {
            clipping_plane,
            model,
            colors
        }
    }
}

#[derive(Clone)]
pub struct MeshDrawCall<'a> {
    pub mesh: &'a GpuMesh,
    pub instances: Vec<InstanceData>,
    pub wireframe: bool,
    pub cull: bool,
    pub bloomfog: bool,
    pub solid: bool,
    pub bloom: bool,
    pub mirror: bool,
}

pub struct PointDrawCall<'a> {
    pub mesh: &'a GpuMesh,
    pub instances: Vec<InstanceData>,
    pub size: f32,
}

pub struct HandleDrawCall<'a> {
    pub mesh: &'a GpuMesh,
    pub instances: Vec<InstanceData>,
}

#[derive(Debug)]
pub struct GpuMesh {
    pub vao: glow::NativeVertexArray,
    pub vbos: [glow::NativeBuffer; 3],
    pub instance_vbo: glow::NativeBuffer,

    pub point_vao: glow::NativeVertexArray,
    pub point_vbo: glow::NativeBuffer,
    pub point_instance_vbo: glow::NativeBuffer,

    pub vertex_count: usize,
    pub point_count: usize,
}

impl GpuMesh {
    fn setup_instance_attribs(gl: &glow::Context) -> glow::NativeBuffer {
        unsafe {
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            let stride = std::mem::size_of::<InstanceData>() as i32;
            let mut offset = 0i32;

            for col in 3..=15u32 {
                gl.enable_vertex_attrib_array(col);
                gl.vertex_attrib_pointer_f32(col, 4, glow::FLOAT, false, stride, offset);
                gl.vertex_attrib_divisor(col, 1);
                offset += 16;
            }

            vbo
        }
    }

    fn upload_instances(gl: &glow::Context, vbo: glow::NativeBuffer, instances: &[InstanceData]) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(instances),
                glow::DYNAMIC_DRAW,
            );
        }
    }

    pub fn rebuild(
        &mut self,
        gl: &glow::Context,
        position_us: &[Vec4],
        normal_vs: &[Vec4],
        material_data: &[IVec3],
        point_positions: &[Vec3],
    ) {
        self.vertex_count = position_us.len();
        self.point_count = point_positions.len();

        let pos_u: &[u8] = bytemuck::cast_slice(position_us);
        let norm_v: &[u8] = bytemuck::cast_slice(normal_vs);
        let mats: &[u8] = bytemuck::cast_slice(material_data);

        let p_pos: &[u8] = bytemuck::cast_slice(point_positions);

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[0]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, pos_u, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[1]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, norm_v, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[2]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, mats, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.point_vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, p_pos, glow::DYNAMIC_DRAW);
 
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    pub fn new(
        gl: &glow::Context,
        position_us: &[Vec4],
        normal_vs: &[Vec4],
        material_data: &[IVec3],
        point_positions: &[Vec3],
    ) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            let vbos = [
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(0);
                    gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, 0, 0);
                    vbo
                },
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(1);
                    gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 0, 0);
                    vbo
                },
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(2);
                    gl.vertex_attrib_pointer_i32(2, 3, glow::INT, 0, 0);
                    vbo
                },
            ];
            let instance_vbo = Self::setup_instance_attribs(gl);
            gl.bind_vertex_array(None);

            let point_vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(point_vao));
            let point_vbo = {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
                vbo
            };
            let point_instance_vbo = Self::setup_instance_attribs(gl);
            gl.bind_vertex_array(None);

            let mut mesh = Self {
                vao,
                vbos,
                instance_vbo,
                point_vao,
                point_vbo,
                point_instance_vbo,
                vertex_count: 0,
                point_count: 0,
            };
            mesh.rebuild(
                gl,
                position_us,
                normal_vs,
                material_data,
                point_positions,
            );
            mesh
        }
    }

    pub fn draw_tris(
        &self,
        gl: &glow::Context,
        instances: &[InstanceData],
        wireframe: bool,
        renderer: &Renderer,
    ) {
        if instances.is_empty() {
            return;
        }
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            Self::upload_instances(gl, self.instance_vbo, instances);
            let n = instances.len() as i32;
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, self.vertex_count as i32, n);
            if wireframe {
                renderer.set_int(gl, renderer.mesh, "u_render_mode", 2);
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                gl.line_width(0.5);
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, self.vertex_count as i32, n);
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
            }
            gl.bind_vertex_array(None);
        }
    }

    pub fn draw_points(&self, gl: &glow::Context, instances: &[InstanceData]) {
        if instances.is_empty() {
            return;
        }
        unsafe {
            gl.bind_vertex_array(Some(self.point_vao));
            Self::upload_instances(gl, self.point_instance_vbo, instances);
            gl.draw_arrays_instanced(
                glow::POINTS,
                0,
                self.point_count as i32,
                instances.len() as i32,
            );
            gl.bind_vertex_array(None);
        }
    }

    pub fn destroy(self, gl: &glow::Context) {
        unsafe {
            for vbo in self.vbos {
                gl.delete_buffer(vbo);
            }
            gl.delete_buffer(self.point_vbo);
            gl.delete_buffer(self.instance_vbo);
            gl.delete_buffer(self.point_instance_vbo);
            gl.delete_vertex_array(self.vao);
            gl.delete_vertex_array(self.point_vao);
        }
    }

    pub fn set_from_hashmap(
        gl: &glow::Context,
        mesh: &LightMesh,
        mut gpu_meshes: HashMap<String, Self>,
        mesh_textures: &IndexMap<String, String>,
        texture_paths: &HashMap<String, PathBuf>,
        atlas_map: &HashMap<PathBuf, Vec4>,
    ) -> HashMap<String, Self> {
        let mut out = HashMap::new();
        for (name, part) in mesh.parts.iter() {
            let mut gpu_mesh = gpu_meshes
                .remove(name)
                .unwrap_or_else(|| GpuMesh::new(gl, &[], &[], &[], &[]));
            gpu_mesh.set_from_light_mesh_part(
                gl,
                part,
                &mesh.data,
                mesh_textures,
                texture_paths,
                atlas_map,
            );
            out.insert(name.clone(), gpu_mesh);
        }
        for unused in gpu_meshes.into_values() {
            unused.destroy(gl);
        }
        out
    }

    pub fn from_light_mesh(
        gl: &glow::Context,
        mesh: &LightMesh,
        texture_paths: &HashMap<String, PathBuf>,
        atlas_map: &HashMap<PathBuf, Vec4>,
    ) -> HashMap<String, Self> {
        Self::set_from_hashmap(
            gl,
            mesh,
            HashMap::new(),
            &mesh.textures,
            texture_paths,
            atlas_map,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_triangle_data(
        vertice_us: &mut Vec<Vec4>,
        normal_vs: &mut Vec<Vec4>,
        materials: &mut Vec<IVec3>,
        part: &Part,
        data: &IndexMap<String, MaterialData>,
        transform: &Mat4,
        remap_data: &IndexMap<String, String>,
        mesh_textures: &IndexMap<String, String>,
        texture_paths: &HashMap<String, PathBuf>,
        atlas_map: &HashMap<PathBuf, Vec4>,
    ) {
        let mat3 = Mat3::from_mat4(*transform);
        let normal_transform = mat3.inverse().transpose();
        let flip = mat3.determinant() < 0.;

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
        } in part.triangles.0.iter()
        {
            let mut verts = [
                transform.transform_point3(part.resolve_vertex(v0).unwrap()),
                transform.transform_point3(part.resolve_vertex(v1).unwrap()),
                transform.transform_point3(part.resolve_vertex(v2).unwrap()),
            ];
            let mut norms = [
                normal_transform * part.resolve_normal(n0).unwrap(),
                normal_transform * part.resolve_normal(n1).unwrap(),
                normal_transform * part.resolve_normal(n2).unwrap(),
            ];

            let material_key = match material {
                Some(mat) => Some(if let Some(remap) = remap_data.get(mat) {
                    remap.as_str()
                } else {
                    mat.as_str()
                }),
                None => None,
            };

            let tex_path: Option<&PathBuf> = data
                .get(material_key.unwrap_or("default"))
                .and_then(|mat_data| {
                    mesh_textures
                        .get(&mat_data.texture.to_string())
                })
                .and_then(|asset_key| texture_paths.get(asset_key));

            let remap = |uv: Vec2| -> Vec2 {
                if let Some(path) = tex_path
                    && let Some(rect) = atlas_map.get(path)
                {
                    return Vec2::new(rect.x.lerp(rect.z, uv.x), rect.y.lerp(rect.w, uv.y));
                }
                uv
            };

            let mut uvs2 = [
                remap(part.resolve_uv(u0)),
                remap(part.resolve_uv(u1)),
                remap(part.resolve_uv(u2)),
            ];

            if flip {
                verts.swap(0, 2);
                norms.swap(0, 2);
                uvs2.swap(0, 2);
            }

            let (verts, norms) = {
                let [v0, v1, v2] = verts;
                let [u0, u1, u2] = uvs2;
                let [n0, n1, n2] = norms;

                (
                    [v0.extend(u0.x), v1.extend(u1.x), v2.extend(u2.x)],
                    [n0.extend(u0.y), n1.extend(u1.y), n2.extend(u2.y)]
                )
            };

            vertice_us.extend_from_slice(&verts);
            normal_vs.extend_from_slice(&norms);

            let material = match material {
                Some(mat) => Some(if let Some(remap) = remap_data.get(mat) {
                    remap.as_str()
                } else {
                    mat.as_str()
                }),
                None => None,
            };

            if let Some(MaterialData {
                material,
                texture: _,
                color,
            }) = data.get(material.unwrap_or("default"))
                && *material != 0
            {
                let mat = IVec3::new(*color as i32, *material as i32, 0);
                materials.extend_from_slice(&[mat; 3]);
            } else {
                let mat = IVec3::new(8, 0, 0);
                materials.extend_from_slice(&[mat; 3]);
            }
        }
    }

    pub fn add_point_data(points: &mut Vec<Vec3>, part: &Part, tranform: &Mat4) {
        for p in part.vertices.indexed.iter() {
            points.push(tranform.transform_point3(*p));
        }
        for p in part.vertices.named.values() {
            points.push(tranform.transform_point3(*p));
        }
        for p in part.vertices.compute.values() {
            if let Ok(p) = p.compute(part) {
                points.push(tranform.transform_point3(p));
            }
        }
    }

    pub fn set_from_full_light_mesh(
        &mut self,
        gl: &glow::Context,
        light_mesh: &LightMesh,
        texture_paths: &HashMap<String, PathBuf>,
        atlas_map: &HashMap<PathBuf, Vec4>,
    ) {
        let mut vertice_us = Vec::new();
        let mut normal_vs = Vec::new();
        let mut materials = Vec::new();

        for placement in light_mesh.placements.iter() {
            let mat = placement.transform();
            let part = light_mesh.parts.get(&placement.part).unwrap();
            Self::add_triangle_data(
                &mut vertice_us,
                &mut normal_vs,
                &mut materials,
                part,
                &light_mesh.data,
                &mat,
                &placement.remap_data,
                &light_mesh.textures,
                texture_paths,
                atlas_map,
            );
        }

        self.rebuild(gl, &vertice_us, &normal_vs, &materials, &[]);
    }

    pub fn points_from_light_mesh(&mut self, gl: &glow::Context, light_mesh: &LightMesh) {
        let mut vertice_us = Vec::new();
        let mut points = Vec::new();

        let mut materials = Vec::new();
        let mut normal_vs = Vec::new();

        let mut circle_plane = |pos: Vec3, axis: Vec3, a: Vec3, b: Vec3, c: i32| {
            let a = a * 0.5;
            let b = b * 0.5;
            let p0 = (pos + axis + a + b).extend(0.);
            let p1 = (pos + axis + a - b).extend(0.);
            let p2 = (pos + axis - a - b).extend(0.);
            let p3 = (pos + axis - a + b).extend(0.);
            let n0 = Vec4::new(-1., -1., 0., 0.);
            let n1 = Vec4::new(-1., 1., 0., 0.);
            let n2 = Vec4::new(1., 1., 0., 0.);
            let n3 = Vec4::new(1., -1., 0., 0.);
            vertice_us.extend_from_slice(&[p0, p3, p1, p1, p3, p2]);
            materials.extend_from_slice(&[IVec3::new(c, 0, 0); 6]);
            normal_vs.extend_from_slice(&[n0, n3, n1, n1, n3, n2]);
        };

        for placement in light_mesh.placements.iter() {
            let pos = placement.position;
            let transform = Mat4::from_quat(placement.rotation);
            let up = transform.transform_vector3(Vec3::Y);
            let left = transform.transform_vector3(Vec3::X);
            let forward = transform.transform_vector3(Vec3::Z);

            let sx = pos + transform.transform_vector3(Vec3::X * placement.scale.x);
            let sy = pos + transform.transform_vector3(Vec3::Y * placement.scale.y);
            let sz = pos + transform.transform_vector3(Vec3::Z * placement.scale.z);

            circle_plane(pos, up, left, forward, 1);
            circle_plane(pos, left, forward, up, 2);
            circle_plane(pos, forward, up, left, 3);

            points.extend_from_slice(&[pos, sx, sy, sz]);
        }

        self.rebuild(
            gl,
            &vertice_us,
            &normal_vs,
            &materials,
            &points,
        );
    }

    pub fn set_from_light_mesh_part(
        &mut self,
        gl: &glow::Context,
        part: &Part,
        data: &IndexMap<String, MaterialData>,
        mesh_textures: &IndexMap<String, String>,
        texture_paths: &HashMap<String, PathBuf>,
        atlas_map: &HashMap<PathBuf, Vec4>,
    ) {
        let mut vertice_us = Vec::new();
        let mut normal_vs = Vec::new();
        let mut materials = Vec::new();
        let mut points = Vec::new();

        Self::add_triangle_data(
            &mut vertice_us,
            &mut normal_vs,
            &mut materials,
            part,
            data,
            &Mat4::IDENTITY,
            &IndexMap::default(),
            mesh_textures,
            texture_paths,
            atlas_map,
        );
        Self::add_point_data(&mut points, part, &Mat4::IDENTITY);

        self.rebuild(
            gl,
            &vertice_us,
            &normal_vs,
            &materials,
            &points,
        );
    }
}


impl data::SpectrogramData {
    pub fn generate(&self, gl: &glow::Context, vm: &mut GpuMesh) {
        // NOTE: update to a match block when more styles get added.

        let v = [
            self.rotation * Vec3::new(-0.5, 0., -0.5),
            self.rotation * Vec3::new(-0.5, 0.,  0.5),
            self.rotation * Vec3::new( 0.5, 0.,  0.5),
            self.rotation * Vec3::new( 0.5, 0., -0.5),

            self.rotation * Vec3::new(-0.5, self.base_height, -0.5),
            self.rotation * Vec3::new(-0.5, self.base_height,  0.5),
            self.rotation * Vec3::new( 0.5, self.base_height,  0.5),
            self.rotation * Vec3::new( 0.5, self.base_height, -0.5),
        ];

        let tris = [
            v[7], v[5], v[6], v[7], v[4], v[5],
            v[1], v[2], v[6], v[1], v[6], v[5],
            v[3], v[0], v[4], v[3], v[4], v[7],
            v[0], v[1], v[5], v[0], v[5], v[4],
            v[2], v[3], v[7], v[2], v[7], v[6],
        ];

        let normals = [
            Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y,
            Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z,
            Vec3::NEG_Z,Vec3::NEG_Z,Vec3::NEG_Z,Vec3::NEG_Z,Vec3::NEG_Z,Vec3::NEG_Z,
            Vec3::NEG_X,Vec3::NEG_X,Vec3::NEG_X,Vec3::NEG_X,Vec3::NEG_X,Vec3::NEG_X,
            Vec3::X, Vec3::X, Vec3::X, Vec3::X, Vec3::X, Vec3::X,
        ].map(|n| self.rotation * n);


        let offset = self.rotation * self.offset;

        let mut geo = Vec::new();
        let mut norms = Vec::new();

        for i in 0..self.count {
            let tower: Vec<Vec3> = tris.iter().map(|v| v + self.position + offset * (i as f32)).collect();
            geo.extend_from_slice(&tower);
            norms.extend_from_slice(&normals);
        }

        if let Some(plane) = self.mirror {
            let mn = plane.xyz().normalize();
            let d = plane.w;

            let mirror = |point: Vec3| -> Vec3 {
                point - 2. * (mn.dot(point) + d) * mn
            };

            let mut mirror_tris = [
                v[6], v[5], v[7], v[5], v[4], v[7],
                v[2], v[1], v[6], v[6], v[1], v[5],
                v[0], v[3], v[4], v[4], v[3], v[7],
                v[1], v[0], v[5], v[5], v[0], v[4],
                v[3], v[2], v[7], v[7], v[2], v[6],
            ];
            let mirror_normals: Vec<Vec3> = normals.iter().map(|v| v - 2. * mn.dot(*v) * mn).collect();
            mirror_tris.iter_mut().for_each(|v| *v = mirror(*v));

            let offset = offset - 2. * mn.dot(offset) * mn;
            let pos = mirror(self.position);

            for i in 0..self.count {
                let tower: Vec<Vec3> = mirror_tris.iter().map(|v| v + pos + offset * (i as f32)).collect();
                geo.extend_from_slice(&tower);
                norms.extend_from_slice(&mirror_normals);
            }

        }

        let mats = vec![IVec3::new(0, 0, 1 << 31); geo.len()];

        let mut i = 0;
        let mut next_uv = || -> Vec2 {
            let res = match i {
                0 => Vec2::ZERO,
                1 => Vec2::new(0.5, 0.),
                _ => Vec2::new(0., 0.5)
            };
            i = (i + 1) % 3;
            res
        };
        let (geo, norms) = geo
            .into_iter()
            .zip(norms)
            .map(|(pos, norm)| {
                let uv = next_uv();
                (pos.extend(uv.x), norm.extend(uv.y))
            })
            .collect::<(Vec<Vec4>, Vec<Vec4>)>();

        vm.rebuild(gl, &geo, &norms, &mats, &[]);
    }
}

pub struct Renderer {
    pub mesh: glow::NativeProgram,
    pub mirror: glow::NativeProgram,
    pub point: glow::NativeProgram,
    pub flat: glow::NativeProgram,
    pub handles: glow::NativeProgram,
    pub handle_points: glow::NativeProgram,
    pub grid_vao: glow::NativeVertexArray,
    pub gvbo: glow::NativeBuffer,
    pub grid_n: i32,
    pub axis_vao: glow::NativeVertexArray,
    pub avbo: glow::NativeBuffer,
    pub blue_noise: glow::NativeTexture,
    pub missing_texture: glow::NativeTexture,
    /// maps a texture id ('beatcraft:textures/...') to a real path.
    pub texture_paths: HashMap<String, PathBuf>,
    /// the single atlas texture
    pub atlas: Option<glow::NativeTexture>,
    /// maps a path to its UV rect (x0, y0, x1, y1) within the atlas
    pub atlas_map: HashMap<PathBuf, Vec4>,
    pub bloomfog: BloomfogRenderer,
}

impl Renderer {
    fn compile_shader(
        gl: &glow::Context,
        vs: &str,
        fs: &str,
    ) -> Result<glow::NativeProgram, String> {
        unsafe {
            let v = gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(|e| e.to_string())?;
            gl.shader_source(v, vs);
            gl.compile_shader(v);
            if !gl.get_shader_compile_status(v) {
                return Err(gl.get_shader_info_log(v));
            }
            let f = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(|e| e.to_string())?;
            gl.shader_source(f, fs);
            gl.compile_shader(f);
            if !gl.get_shader_compile_status(f) {
                return Err(gl.get_shader_info_log(f));
            }
            let p = gl.create_program().map_err(|e| e.to_string())?;
            gl.attach_shader(p, v);
            gl.attach_shader(p, f);
            gl.link_program(p);
            if !gl.get_program_link_status(p) {
                return Err(gl.get_program_info_log(p));
            }
            gl.delete_shader(v);
            gl.delete_shader(f);
            Ok(p)
        }
    }

    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let mesh = Self::compile_shader(
                gl,
                include_str!("./assets/shaders/mesh.vert"),
                include_str!("./assets/shaders/mesh.frag"),
            )?;
            let mirror = Self::compile_shader(
                gl,
                include_str!("./assets/shaders/mirror.vert"),
                include_str!("./assets/shaders/mirror.frag")
            )?;
            let point = Self::compile_shader(
                gl,
                include_str!("./assets/shaders/point.vert"),
                include_str!("./assets/shaders/point.frag"),
            )?;
            let flat = Self::compile_shader(
                gl,
                include_str!("./assets/shaders/flat.vert"),
                include_str!("./assets/shaders/flat.frag"),
            )?;

            let handles = Self::compile_shader(
                gl,
                include_str!("./assets/shaders/handles.vert"),
                include_str!("./assets/shaders/handles.frag"),
            )?;

            let handle_points = Self::compile_shader(
                gl,
                include_str!("./assets/shaders/handle_points.vert"),
                include_str!("./assets/shaders/handle_points.frag"),
            )?;

            let mut grid_pts: Vec<f32> = vec![];
            let (size, step) = (300i32, 10i32);
            let mut i = -size;
            while i <= size {
                let fi = i as f32;
                let fs = size as f32;
                grid_pts.extend_from_slice(&[fi, 0.0, -fs, fi, 0.0, fs, -fs, 0.0, fi, fs, 0.0, fi]);
                i += step;
            }
            let grid_vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(grid_vao));
            let gvbo = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(gvbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&grid_pts),
                glow::STATIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);
            let grid_n = (grid_pts.len() / 3) as i32;

            let ax: f32 = size as f32;
            let axis_pts: [f32; 12] = [0.0, 0.0, 0.0, ax, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, ax];
            let axis_vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(axis_vao));
            let avbo = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(avbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&axis_pts),
                glow::STATIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);

            gl.enable(glow::PROGRAM_POINT_SIZE);

            let blue_noise = {
                let bn =
                    image::load_from_memory(include_bytes!("assets/textures/noise/blue_noise.png"))
                        .unwrap()
                        .to_rgba8();

                let (w, h) = bn.dimensions();
                let pixels = bn.into_raw();

                let tex = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_2D, Some(tex));

                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::LINEAR as i32,
                );

                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    w as i32,
                    h as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(pixels.as_slice())),
                );

                tex
            };

            let missing_texture = {
                let img = image::load_from_memory(MISSING_TEXTURE_BYTES)
                    .unwrap()
                    .to_rgba8();
                let (w, h) = img.dimensions();
                let tex = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::NEAREST as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::NEAREST as i32,
                );
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    w as i32,
                    h as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(img.as_raw().as_slice())),
                );
                tex
            };

            Ok(Self {
                mesh,
                mirror,
                point,
                flat,
                handles,
                handle_points,
                grid_vao,
                gvbo,
                grid_n,
                axis_vao,
                avbo,
                blue_noise,
                missing_texture,
                texture_paths: HashMap::new(),
                atlas: None,
                atlas_map: HashMap::new(),
                bloomfog: BloomfogRenderer::new(gl)?,
            })
        }
    }

    pub fn rebuild_atlases(&mut self, gl: &glow::Context) {
        const ATLAS_SIZE: u32 = 1024;

        unsafe {
            if let Some(tex) = self.atlas.take() {
                gl.delete_texture(tex);
            }
        }
        self.atlas_map.clear();

        let mut atlas_image = image::RgbaImage::new(ATLAS_SIZE, ATLAS_SIZE);
        let mut shelf_x: u32 = 0;
        let mut shelf_y: u32 = 0;
        let mut shelf_h: u32 = 0;

        let mut unique_paths: Vec<PathBuf> = self.texture_paths.values().cloned().collect();
        unique_paths.sort();
        unique_paths.dedup();

        for path in &unique_paths {
            let img = image::open(path)
                .unwrap_or_else(|_| panic!("failed to open texture {:?}", path))
                .to_rgba8();

            let (w, h) = img.dimensions();
            assert!(
                w <= ATLAS_SIZE && h <= ATLAS_SIZE,
                "texture {:?} ({}x{}) exceeds atlas size {}x{}",
                path,
                w,
                h,
                ATLAS_SIZE,
                ATLAS_SIZE
            );

            if shelf_x + w > ATLAS_SIZE {
                shelf_y += shelf_h;
                shelf_x = 0;
                shelf_h = 0;
            }

            assert!(
                shelf_y + h <= ATLAS_SIZE,
                "atlas is full, could not fit texture {:?}",
                path
            );

            image::imageops::replace(&mut atlas_image, &img, shelf_x as i64, shelf_y as i64);

            let u0 = shelf_x as f32 / ATLAS_SIZE as f32;
            let v0 = shelf_y as f32 / ATLAS_SIZE as f32;
            let u1 = (shelf_x + w) as f32 / ATLAS_SIZE as f32;
            let v1 = (shelf_y + h) as f32 / ATLAS_SIZE as f32;
            self.atlas_map
                .insert(path.clone(), Vec4::new(u0, v0, u1, v1));

            shelf_x += w;
            shelf_h = shelf_h.max(h);
        }

        unsafe {
            let tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                ATLAS_SIZE as i32,
                ATLAS_SIZE as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(atlas_image.as_raw().as_slice())),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            self.atlas = Some(tex);
        }
    }

    pub fn remap_uv(&self, texture: &Path, uv: Vec2) -> Vec2 {
        if let Some(rect) = self.atlas_map.get(texture) {
            Vec2::new(rect.x.lerp(rect.z, uv.x), rect.y.lerp(rect.w, uv.y))
        } else {
            uv
        }
    }

    pub fn draw_grid(
        &self,
        gl: &glow::Context,
        vp: &Mat4
    ) {
        unsafe {
            let flat = self.flat;
            gl.line_width(1.);
            gl.use_program(Some(flat));
            if let Some(l) = gl.get_uniform_location(flat, "uMVP") {
                gl.uniform_matrix_4_f32_slice(
                    Some(&l),
                    false,
                    &vp.to_cols_array(),
                );
            }
            if let Some(l) = gl.get_uniform_location(flat, "uColor") {
                gl.uniform_4_f32(Some(&l), 0.27, 0.27, 0.34, 0.5);
            }
            gl.bind_vertex_array(Some(self.grid_vao));
            gl.draw_arrays(glow::LINES, 0, self.grid_n);
            gl.line_width(2.);
            if let Some(l) = gl.get_uniform_location(flat, "uColor") {
                gl.uniform_4_f32(Some(&l), 0.85, 0.2, 0.2, 0.9);
            }
            gl.bind_vertex_array(Some(self.axis_vao));
            gl.draw_arrays(glow::LINES, 0, 2);
            if let Some(l) =
                gl.get_uniform_location(self.flat, "uColor")
            {
                gl.uniform_4_f32(Some(&l), 0.2, 0.45, 0.9, 0.9);
            }
            gl.draw_arrays(glow::LINES, 2, 2);
            gl.line_width(1.);
            gl.bind_vertex_array(None);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_meshes_fancy(
        &mut self,
        gl: &glow::Context,
        view: &Mat4,
        proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
        window: (i32, i32),
        draw_grid: bool,
        mirror_mesh: Option<&GpuMesh>,
        wireframe: bool,
        fog_heights: [f32; 2],
        draw_mirror: bool,
    ) {
        let rd = RefDuper;
        let self2 = unsafe { rd.detach_mut_ref(self) };
        self.bloomfog.draw_meshes(
            self2, gl, view, proj, calls, window, draw_grid,
            None, mirror_mesh, wireframe, fog_heights, draw_mirror,
        );
    }

    pub fn draw_meshes(
        &mut self,
        gl: &glow::Context,
        view: &Mat4, proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
        mirror_mesh: Option<&GpuMesh>,
        wireframe: bool,
        draw_mirror: bool,
    ) {
        unsafe {
            self.draw_meshes_internal(gl, view, proj, calls);
            let rd = RefDuper;
            let self2 = rd.detach_mut_ref(self);
            if draw_mirror && let Some(mirror_mesh) = mirror_mesh {
                self.bloomfog.draw_mirror(
                    self2, gl, view, proj, calls, mirror_mesh,
                    1, wireframe, [-50., -30.]
                );
            }
        }
    }

    fn draw_meshes_internal(
        &mut self,
        gl: &glow::Context,
        view: &Mat4, proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
    ) {
        unsafe {
            gl.use_program(Some(self.mesh));
            self.set_int(gl, self.mesh, "passType", 0);
            self.set_mat4(gl, self.mesh, "u_view", view);
            self.set_mat4(gl, self.mesh, "u_projection", proj);
            let tex = self.atlas.or(Some(self.missing_texture));
            self.set_sampler(gl, self.mesh, "u_texture", tex, 0);
            self.set_sampler(gl, self.mesh, "u_noise", Some(self.blue_noise), 1);
            for call in calls {
                self.set_int(gl, self.mesh, "u_render_mode", 1);
                call.mesh
                    .draw_tris(gl, &call.instances, call.wireframe, self);
            }
        }
    }

    pub fn draw_points_batch(&self, gl: &glow::Context, vp: &Mat4, calls: &[PointDrawCall<'_>]) {
        unsafe {
            gl.use_program(Some(self.point));
            self.set_mat4(gl, self.point, "uVP", vp);
            for call in calls {
                self.set_float(gl, self.point, "uPointSize", call.size);
                call.mesh.draw_points(gl, &call.instances);
            }
        }
    }

    pub fn draw_handles(&self, gl: &glow::Context, vp: &Mat4, calls: &[HandleDrawCall<'_>]) {
        unsafe {
            gl.use_program(Some(self.handles));
            self.set_mat4(gl, self.handles, "uVP", vp);
            for call in calls {
                call.mesh.draw_tris(gl, &call.instances, false, self);
            }
            gl.use_program(Some(self.handle_points));
            self.set_mat4(gl, self.handle_points, "uVP", vp);
            self.set_float(gl, self.handle_points, "uPointSize", 3.);
            for call in calls {
                call.mesh.draw_points(gl, &call.instances);
            }
        }
    }

    fn set_sampler(
        &self,
        gl: &glow::Context,
        prog: glow::NativeProgram,
        name: &str,
        texture: Option<glow::NativeTexture>,
        slot: u32,
    ) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, texture);
            gl.uniform_1_i32(gl.get_uniform_location(prog, name).as_ref(), slot as i32);
        }
    }

    fn set_mat4(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, m: &Mat4) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_matrix_4_f32_slice(Some(&l), false, &m.to_cols_array());
            }
        }
    }
    // fn set_vec4(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: Vec4) {
    //     unsafe {
    //         if let Some(l) = gl.get_uniform_location(prog, name) {
    //             gl.uniform_4_f32(Some(&l), v.x, v.y, v.z, v.w);
    //         }
    //     }
    // }

    fn set_vec3(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: Vec3) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_3_f32(Some(&l), v.x, v.y, v.z);
            }
        }
    }
    fn set_float(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: f32) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_1_f32(Some(&l), v);
            }
        }
    }
    fn set_vec2(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: Vec2) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_2_f32(Some(&l), v.x, v.y);
            }
        }
    }
    fn set_int(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: i32) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_1_i32(Some(&l), v);
            }
        }
    }
}

struct RenderTarget {
    fbo: glow::NativeFramebuffer,
    color: glow::NativeTexture,
    depth: glow::NativeTexture,
    size: (i32, i32),
}

impl RenderTarget {
    pub fn new(gl: &glow::Context, width: i32, height: i32) -> Self {
        unsafe {
            let fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            let color = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(color));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0,
                glow::RGBA as i32,
                width, height, 0,
                glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D, Some(color), 0,
            );

            let depth = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(depth));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0,
                glow::DEPTH_COMPONENT24 as i32,
                width, height, 0,
                glow::DEPTH_COMPONENT, glow::UNSIGNED_INT,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D, Some(depth), 0,
            );

            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE,
                "framebuffer incomplete"
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.bind_texture(glow::TEXTURE_2D, None);

            Self { fbo, color, depth, size: (width, height) }
        }
    }

    pub fn resize(&mut self, gl: &glow::Context, width: i32, height: i32) {
        self.size = (width, height);
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.color));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0,
                glow::RGBA as i32,
                width, height, 0,
                glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );

            gl.bind_texture(glow::TEXTURE_2D, Some(self.depth));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0,
                glow::DEPTH_COMPONENT24 as i32,
                width, height, 0,
                glow::DEPTH_COMPONENT, glow::UNSIGNED_INT,
                glow::PixelUnpackData::Slice(None),
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.viewport(0, 0, self.size.0, self.size.1);
        }
    }

    // pub fn destroy(self, gl: &glow::Context) {
    //     unsafe {
    //         gl.delete_texture(self.color);
    //         gl.delete_texture(self.depth);
    //         gl.delete_framebuffer(self.fbo);
    //     }
    // }
}

pub struct BloomfogRenderer {
    framebuffer: RenderTarget,
    extra_buffer: RenderTarget,
    blurred_buffer: RenderTarget,
    bloom_input: RenderTarget,
    bloom_swap: RenderTarget,
    bloom_output: RenderTarget,
    light_depth: RenderTarget,
    pyramid_buffers: [RenderTarget; 7],
    blur_down: glow::NativeProgram,
    blur_up: glow::NativeProgram,
    gaussian_v: glow::NativeProgram,
    gaussian_h: glow::NativeProgram,
    blue_noise: glow::NativeProgram,
    blit: glow::NativeProgram,
    comp: glow::NativeProgram,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    mirror_target: RenderTarget,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PassType {
    DownSample,
    UpSample,
    GaussianV,
    GaussianH,
    BlueNoise,
    Blit,
    Comp,
}

impl BloomfogRenderer {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {

        let blur_vsh = include_str!("assets/shaders/core/bloomfog_blur.vsh");

        let blur_down = Renderer::compile_shader(gl,
            blur_vsh,
            include_str!("assets/shaders/core/bloomfog_downsample.fsh")
        )?;
        let blur_up = Renderer::compile_shader(gl,
            blur_vsh,
            include_str!("assets/shaders/core/bloomfog_upsample.fsh")
        )?;
        let gaussian_v = Renderer::compile_shader(gl,
            blur_vsh,
            include_str!("assets/shaders/core/gaussian_v.fsh")
        )?;
        let gaussian_h = Renderer::compile_shader(gl,
            blur_vsh,
            include_str!("assets/shaders/core/gaussian_h.fsh")
        )?;
        let blue_noise = Renderer::compile_shader(gl,
            blur_vsh,
            include_str!("assets/shaders/core/blue_noise.fsh")
        )?;
        let blit = Renderer::compile_shader(gl,
            include_str!("assets/shaders/core/beatcraft_blit.vsh"),
            include_str!("assets/shaders/core/beatcraft_blit.fsh")
        )?;
        let comp = Renderer::compile_shader(gl,
            blur_vsh,
            include_str!("assets/shaders/core/composite.fsh")
        )?;

        unsafe {
            let vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(vao));
            let vbo = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let stride = (3 + 2 + 4) * 4;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 3 * 4);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, stride, 5 * 4);
            gl.bind_vertex_array(None);

            Ok(Self {
                framebuffer: RenderTarget::new(gl, 1920, 1080),
                extra_buffer: RenderTarget::new(gl, 1920, 1080),
                blurred_buffer: RenderTarget::new(gl, 1920, 1080),
                bloom_input: RenderTarget::new(gl, 1920, 1080),
                bloom_swap: RenderTarget::new(gl, 1920, 1080),
                bloom_output: RenderTarget::new(gl, 1920, 1080),
                light_depth: RenderTarget::new(gl, 1920, 1080),
                pyramid_buffers: [
                    RenderTarget::new(gl, 512, 512),
                    RenderTarget::new(gl, 512, 512),
                    RenderTarget::new(gl, 512, 512),
                    RenderTarget::new(gl, 512, 512),
                    RenderTarget::new(gl, 512, 512),
                    RenderTarget::new(gl, 512, 512),
                    RenderTarget::new(gl, 512, 512),
                ],
                blur_down,
                blur_up,
                gaussian_v,
                gaussian_h,
                blue_noise,
                blit,
                comp,
                vao,
                vbo,
                mirror_target: RenderTarget::new(gl, 1920, 1080),
            })
        }
    }

    pub fn resize(&mut self, gl: &glow::Context, window: (i32, i32)) {
        self.framebuffer.resize(gl, window.0*2, window.1*2);
        self.blurred_buffer.resize(gl, window.0*2, window.1*2);
        self.extra_buffer.resize(gl, window.0*2, window.1*2);
        self.bloom_input.resize(gl, window.0, window.1);
        self.bloom_swap.resize(gl, window.0, window.1);
        self.bloom_output.resize(gl, window.0, window.1);
        self.light_depth.resize(gl, window.0, window.1);
        self.mirror_target.resize(gl, window.0, window.1);

        let mut md = 2.;
        for rt in self.pyramid_buffers.iter_mut() {
            if window.0 as f32 / md > 0. && window.1 as f32 / md > 0. {
                rt.resize(gl, (window.0 as f32 * 2. / md) as i32, (window.1 as f32 * 2. / md) as i32);
            }
            md *= 2.;
        }
    }

    /// This function flips the render calls across Y.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_mirror(
        &mut self,
        renderer: &mut Renderer,
        gl: &glow::Context,
        view: &Mat4,
        proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
        mirror: &GpuMesh,
        render_mode: i32,
        wireframe: bool,
        fog_heights: [f32; 2],
    ) {
        unsafe {

            let mut saved_vp = [0i32; 4];
            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut saved_vp);

            self.mirror_target.bind(gl);
            gl.front_face(glow::CW);
            gl.clear_color(0., 0., 0., 1.);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            let mut mirrored = Vec::new();

            for call in calls {
                let mut c = call.clone();
                for inst in c.instances.iter_mut() {
                    inst.clipping_plane = Vec4::new(0., -1., 0., 0.);
                    inst.model *= Mat4::from_scale(Vec3::new(1., -1., 1.))
                }
                mirrored.push(c);
            }

            let view_f = *view;

            gl.enable(glow::CLIP_DISTANCE0);
            if render_mode == 0 {
                self.render_solid(
                    renderer, gl, &view_f, proj, &mirrored,
                    fog_heights
                );
            } else {
                renderer.draw_meshes_internal(gl, &view_f, proj, &mirrored);
            }

            gl.front_face(glow::CCW);
            gl.disable(glow::CLIP_DISTANCE0);

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(saved_vp[0], saved_vp[1], saved_vp[2], saved_vp[3]);

            gl.use_program(Some(renderer.mirror));
            gl.enable(glow::CULL_FACE);
            renderer.set_sampler(gl, renderer.mirror, "u_texture", Some(self.mirror_target.color), 0);
            renderer.set_sampler(gl, renderer.mirror, "u_noise", Some(renderer.blue_noise), 1);
            renderer.set_sampler(gl, renderer.mirror, "u_bloomfog", Some(self.blurred_buffer.color), 2);
            renderer.set_vec3(gl, renderer.mirror, "u_worldPos", Vec3::ZERO);
            renderer.set_mat4(gl, renderer.mirror, "u_view", view);
            renderer.set_mat4(gl, renderer.mirror, "u_projection", proj);
            renderer.set_int(gl, renderer.mirror, "u_render_mode", render_mode);
            mirror.draw_tris(
                gl,
                &[InstanceData::new(
                    Vec4::ZERO,
                    Mat4::IDENTITY,
                    LIGHT_COLORS
                )],
                wireframe,
                renderer
            );
            gl.disable(glow::CULL_FACE);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_meshes(
        &mut self,
        renderer: &mut Renderer,
        gl: &glow::Context,
        view: &Mat4,
        proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
        window: (i32, i32),
        draw_grid: bool,
        main_target: Option<&RenderTarget>,
        mirror_mesh: Option<&GpuMesh>,
        wireframe: bool,
        fog_heights: [f32; 2],
        draw_mirror: bool,
    ) {
        unsafe {
            // pre-render beatmaps
            // render bloomfog

            let mut saved_vp = [0i32; 4];
            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut saved_vp);

            if window != self.bloom_input.size {
                self.resize(gl, window);
            }
            gl.disable(glow::SCISSOR_TEST);

            self.framebuffer.bind(gl);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.render_bloomfog(renderer, gl, view, proj, calls, fog_heights);
            gl.depth_mask(false);

            self.apply_pyramid_blur(renderer, gl, window);

            gl.bind_framebuffer(glow::FRAMEBUFFER, main_target.map(|t| t.fbo));
            if main_target.is_none() {
                gl.viewport(saved_vp[0], saved_vp[1], saved_vp[2], saved_vp[3]);
            }
            self.apply_effect_pass(renderer, gl, &self.blurred_buffer, None, PassType::Blit, false, true, window, 11., 2.);
            gl.depth_mask(true);

            if draw_grid {
                renderer.draw_grid(gl, &(*proj * *view));
            }
            // render mirrored?
            // render HUD
            // render beatmaps
            //   draw mirrors
            if draw_mirror && let Some(mirror_mesh) = mirror_mesh {
                self.draw_mirror(
                    renderer, gl, view, proj, calls, mirror_mesh,
                    0, wireframe, fog_heights,
                );
            }
            //   draw maps
            //     bloomfogPosCol
            //     envLights
            self.render_solid(renderer, gl, view, proj, calls, fog_heights);
            //     floor1
            //     floorLights
            //     obstacles
            // render debug
            // render particles
            // render sabers
            // render smoke
            // render bloom
            self.render_bloom(
                renderer, gl, view, proj, calls, window,
                saved_vp, main_target, mirror_mesh,
                fog_heights
            );
            gl.enable(glow::SCISSOR_TEST);
            gl.enable(glow::DEPTH_TEST);
        }
    }

    fn render_bloomfog(
        &mut self,
        renderer: &Renderer,
        gl: &glow::Context,
        view: &Mat4,
        proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
        fog_heights: [f32; 2],
    ) {
        unsafe {
            let cam_pos = view.inverse().transform_point3(Vec3::ZERO);
            let cam_rot = Quat::from_mat4(view);

            // TODO: conditionally enable clipping plane
            gl.enable(glow::CLIP_DISTANCE0);

            // TODO: re-set instance divisor?

            gl.use_program(Some(renderer.mesh));
            renderer.set_mat4(gl, renderer.mesh, "u_view", view);
            renderer.set_mat4(gl, renderer.mesh, "u_projection", proj);
            let tex = renderer.atlas.or(Some(renderer.missing_texture));
            renderer.set_sampler(gl, renderer.mesh, "u_texture", tex, 0);
            renderer.set_sampler(gl, renderer.mesh, "u_bloomfog", Some(self.extra_buffer.color), 1);
            renderer.set_int(gl, renderer.mesh, "passType", 2);
            renderer.set_vec2(gl, renderer.mesh, "u_fog", Vec2::new(fog_heights[0], fog_heights[1]));

            let mut world_transform = Mat4::from_translation(cam_pos);
            world_transform *= Mat4::from_quat(cam_rot.conjugate());
            renderer.set_mat4(gl, renderer.mesh, "world_transform", &world_transform);
            for call in calls.iter() {
                if !call.bloomfog {
                    continue;
                }
                if call.cull {
                    gl.enable(glow::CULL_FACE);
                }
                gl.bind_vertex_array(Some(call.mesh.vao));
                renderer.set_int(gl, renderer.mesh, "u_render_mode", 0);
                call.mesh.draw_tris(gl, &call.instances, false, renderer);
                if call.cull {
                    gl.disable(glow::CULL_FACE);
                }
            }

            gl.disable(glow::CLIP_DISTANCE0);
            gl.use_program(None);
        }
    }

    fn render_solid(
        &mut self,
        renderer: &Renderer,
        gl: &glow::Context,
        view: &Mat4,
        proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
        fog_heights: [f32; 2],
    ) {
        unsafe {
            let cam_pos = view.inverse().transform_point3(Vec3::ZERO);
            let cam_rot = Quat::from_mat4(view);

            // TODO: conditionally enable clipping plane
            gl.enable(glow::CLIP_DISTANCE0);
            gl.depth_mask(true);

            // TODO: re-set instance divisor?

            gl.use_program(Some(renderer.mesh));
            renderer.set_mat4(gl, renderer.mesh, "u_view", view);
            renderer.set_mat4(gl, renderer.mesh, "u_projection", proj);
            let tex = renderer.atlas.or(Some(renderer.missing_texture));
            renderer.set_sampler(gl, renderer.mesh, "u_texture", tex, 0);
            renderer.set_sampler(gl, renderer.mesh, "u_bloomfog", Some(self.blurred_buffer.color), 1);
            renderer.set_int(gl, renderer.mesh, "passType", 0);
            renderer.set_vec2(gl, renderer.mesh, "u_fog", Vec2::new(fog_heights[0], fog_heights[1]));

            let mut world_transform = Mat4::from_translation(cam_pos);
            world_transform *= Mat4::from_quat(cam_rot.conjugate());
            renderer.set_mat4(gl, renderer.mesh, "world_transform", &world_transform);
            for call in calls.iter() {
                if !call.solid {
                    continue;
                }
                if call.cull {
                    gl.enable(glow::CULL_FACE);
                }
                gl.bind_vertex_array(Some(call.mesh.vao));
                renderer.set_int(gl, renderer.mesh, "u_render_mode", 0);
                call.mesh.draw_tris(gl, &call.instances, call.wireframe, renderer);
                if call.cull {
                    gl.disable(glow::CULL_FACE);
                }
            }

            renderer.set_int(gl, renderer.mesh, "passType", 3);
            for call in calls.iter() {
                if !call.solid {
                    continue;
                }
                if call.cull {
                    gl.enable(glow::CULL_FACE);
                }
                gl.bind_vertex_array(Some(call.mesh.vao));
                renderer.set_int(gl, renderer.mesh, "u_render_mode", 0);
                call.mesh.draw_tris(gl, &call.instances, call.wireframe, renderer);
                if call.cull {
                    gl.disable(glow::CULL_FACE);
                }
            }

            gl.disable(glow::CLIP_DISTANCE0);
            gl.use_program(None);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_bloom(
        &self,
        renderer: &Renderer,
        gl: &glow::Context,
        view: &Mat4,
        proj: &Mat4,
        calls: &[MeshDrawCall<'_>],
        window: (i32, i32),
        saved_vp: [i32; 4],
        main_target: Option<&RenderTarget>,
        mirror_mesh: Option<&GpuMesh>,
        fog_heights: [f32; 2],
    ) {
        unsafe {
            self.light_depth.bind(gl);
            gl.clear_color(0., 0., 0., 0.);
            gl.clear(glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(renderer.mesh));
            renderer.set_mat4(gl, renderer.mesh, "u_view", view);
            renderer.set_mat4(gl, renderer.mesh, "u_projection", proj);
            let tex = renderer.atlas.or(Some(renderer.missing_texture));
            renderer.set_sampler(gl, renderer.mesh, "u_texture", tex, 0);
            renderer.set_sampler(gl, renderer.mesh, "u_bloomfog", Some(self.blurred_buffer.color), 1);
            renderer.set_vec2(gl, renderer.mesh, "u_fog", Vec2::new(fog_heights[0], fog_heights[1]));
            renderer.set_sampler(gl, renderer.mesh, "u_depth", Some(self.light_depth.depth), 2);
            renderer.set_int(gl, renderer.mesh, "u_render_mode", 0);
            renderer.set_int(gl, renderer.mesh, "passType", 0);
            for call in calls {
                if !call.solid {
                    continue;
                }
                if call.cull {
                    gl.enable(glow::CULL_FACE);
                }
                call.mesh.draw_tris(gl, &call.instances, false, renderer);
                if call.cull {
                    gl.disable(glow::CULL_FACE);
                }
            }
            if let Some(mirror_mesh) = mirror_mesh {
                mirror_mesh.draw_tris(gl, &[InstanceData::new(Vec4::ZERO, Mat4::IDENTITY, LIGHT_COLORS)], false, renderer);
            }
            renderer.set_int(gl, renderer.mesh, "passType", 1);
            for call in calls {
                if !call.solid {
                    continue;
                }
                if call.cull {
                    gl.enable(glow::CULL_FACE);
                }
                call.mesh.draw_tris(gl, &call.instances, false, renderer);
                if call.cull {
                    gl.disable(glow::CULL_FACE);
                }
            }


            self.bloom_input.bind(gl);
            gl.clear_color(0., 0., 0., 0.);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            renderer.set_int(gl, renderer.mesh, "passType", 1);
            //gl.blend_func(glow::ONE, glow::ONE);
            gl.enable(glow::BLEND);
            gl.disable(glow::DEPTH_TEST);
            for call in calls {
                if !call.bloom {
                    continue;
                }
                if call.cull {
                    gl.enable(glow::CULL_FACE);
                }
                call.mesh.draw_tris(gl, &call.instances, false, renderer);
                if call.cull {
                    gl.disable(glow::CULL_FACE);
                }
            }

            self.apply_effect_pass(renderer, gl, &self.bloom_input, Some(&self.bloom_swap), PassType::GaussianH, true, true, window, 3.25, 1.);
            self.apply_effect_pass(renderer, gl, &self.bloom_swap, Some(&self.bloom_output), PassType::GaussianV, true, true, window, 3.25, 1.);

            gl.bind_framebuffer(glow::FRAMEBUFFER, main_target.map(|v| v.fbo));
            if main_target.is_none() {
                gl.viewport(saved_vp[0], saved_vp[1], saved_vp[2], saved_vp[3]);
            }
            self.apply_effect_pass(renderer, gl, &self.bloom_output, None, PassType::Blit, false, false, window, 11., 1.);
        }
    }

    fn apply_pyramid_blur(&self, renderer: &Renderer, gl: &glow::Context, window: (i32, i32)) {
        let quad_size = 1.;

        // self.apply_effect_pass(renderer, gl, &self.framebuffer, Some(&self.extra_buffer), PassType::GaussianV, true, true, window, 11., quad_size);
        // self.apply_effect_pass(renderer, gl, &self.extra_buffer, Some(&self.blurred_buffer), PassType::GaussianH, true, true, window, 11., quad_size);

        let mut current = &self.framebuffer;
        for l in 0..7 {
            self.apply_effect_pass(renderer, gl, current, Some(&self.pyramid_buffers[l]), PassType::DownSample, true, true, window, 11., quad_size);
            current = &self.pyramid_buffers[l];
        }

        self.apply_effect_pass(renderer, gl, current, Some(&self.extra_buffer), PassType::UpSample, true, true, window, 11., quad_size);
        self.apply_effect_pass(renderer, gl, &self.extra_buffer, Some(&self.framebuffer), PassType::GaussianV, true, true, window, 11., quad_size);
        self.apply_effect_pass(renderer, gl, &self.framebuffer, Some(&self.extra_buffer), PassType::GaussianH, true, true, window, 11., quad_size);
        self.apply_effect_pass(renderer, gl, &self.extra_buffer, Some(&self.blurred_buffer), PassType::BlueNoise, false, true, window, 11., quad_size);

    }

    #[allow(clippy::too_many_arguments)]
    fn apply_effect_pass(
        &self,
        renderer: &Renderer,
        gl: &glow::Context,
        input: &RenderTarget,
        output: Option<&RenderTarget>,
        pass: PassType,
        linear: bool,
        clear_output: bool,
        window: (i32, i32),
        radius: f32,
        quad_size: f32,
    ) {
        unsafe {
            if let Some(output) = output {
                output.bind(gl);
                if clear_output {
                    gl.clear_color(0., 0., 0., 0.);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }
            }

            let shader = match pass {
                PassType::DownSample => self.blur_down,
                PassType::UpSample => self.blur_up,
                PassType::GaussianV => self.gaussian_v,
                PassType::GaussianH => self.gaussian_h,
                PassType::BlueNoise => self.blue_noise,
                PassType::Blit => self.blit,
                PassType::Comp => self.comp,
            };

            gl.use_program(Some(shader));
            renderer.set_sampler(gl, shader, "Sampler0", Some(input.color), 0);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, if linear { glow::LINEAR } else { glow::NEAREST } as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, if linear { glow::LINEAR } else { glow::NEAREST } as i32);
            if pass == PassType::BlueNoise {
                renderer.set_sampler(gl, shader, "Sampler1", Some(renderer.blue_noise), 1);
                renderer.set_vec2(gl, shader, "texelSize", Vec2::new(512. / window.0 as f32, 512. / window.1 as f32));
            }
            else if pass == PassType::Comp {
                renderer.set_sampler(gl, shader, "Sampler1", None, 1);
            }
            else {
                renderer.set_vec2(gl, shader, "texelSize", Vec2::new(radius / window.0 as f32, radius / window.1 as f32));
            }

            let rg = 2./255.;

            let data: [f32; 54] = [
                -quad_size, -quad_size, 0.,  0., 0., rg, rg, 0., 1.,
                 quad_size, -quad_size, 0.,  1., 0., rg, rg, 0., 1.,
                 quad_size,  quad_size, 0.,  1., 1., rg, rg, 0., 1.,

                -quad_size, -quad_size, 0.,  0., 0., rg, rg, 0., 1.,
                 quad_size,  quad_size, 0.,  1., 1., rg, rg, 0., 1.,
                -quad_size,  quad_size, 0.,  0., 1., rg, rg, 0., 1.,
            ];
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&data),
                glow::DYNAMIC_DRAW
            );
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            gl.bind_vertex_array(None);
        }
    }

}

