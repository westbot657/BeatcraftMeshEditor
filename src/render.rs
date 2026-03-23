use std::collections::HashMap;

use eframe::glow::{self, HasContext};
use glam::{Mat3, Mat4, Vec3, Vec4};
use indexmap::IndexMap;

use crate::data::MaterialData;
use crate::light_mesh::{LightMesh, Part, Triangle, Vertex};

/// Per-instance data passed to the GPU.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model: Mat4,
    /// rgb + alpha packed into one vec4
    pub color_alpha: [f32; 4],
}

impl InstanceData {
    pub fn new(model: Mat4, alpha: f32, part_color: Option<[f32; 3]>) -> Self {
        let color_alpha = match part_color {
            Some(c) => [c[0], c[1], c[2], alpha],
            None => [0.0, 0.0, 0.0, alpha],
        };
        Self { model, color_alpha }
    }
}

pub struct MeshDrawCall<'a> {
    pub mesh: &'a GpuMesh,
    pub instances: Vec<InstanceData>,
    pub wireframe: bool,
}

pub struct PointDrawCall<'a> {
    pub mesh: &'a GpuMesh,
    pub instances: Vec<InstanceData>,
    pub size: f32,
}

pub struct GpuMesh {
    pub vao: glow::NativeVertexArray,
    pub vbos: [glow::NativeBuffer; 3],
    pub instance_vbo: glow::NativeBuffer,

    pub point_vao: glow::NativeVertexArray,
    pub point_vbos: [glow::NativeBuffer; 3],
    pub point_instance_vbo: glow::NativeBuffer,

    pub vertex_count: usize,
}

impl GpuMesh {
    fn setup_instance_attribs(gl: &glow::Context) -> glow::NativeBuffer {
        unsafe {
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            let stride = std::mem::size_of::<InstanceData>() as i32;
            let mut offset = 0i32;

            // mat4 columns at locations 3-6 (4 × vec4)
            for col in 0..4u32 {
                gl.enable_vertex_attrib_array(3 + col);
                gl.vertex_attrib_pointer_f32(3 + col, 4, glow::FLOAT, false, stride, offset);
                gl.vertex_attrib_divisor(3 + col, 1);
                offset += 16; // sizeof(vec4)
            }

            // color_alpha: vec4 at location 7
            gl.enable_vertex_attrib_array(7);
            gl.vertex_attrib_pointer_f32(7, 4, glow::FLOAT, false, stride, offset);
            gl.vertex_attrib_divisor(7, 1);
            offset += 16;

            // use_part_color: float at location 8
            gl.enable_vertex_attrib_array(8);
            gl.vertex_attrib_pointer_f32(8, 1, glow::FLOAT, false, stride, offset);
            gl.vertex_attrib_divisor(8, 1);

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
        positions: &[Vec3],
        normals: &[Vec3],
        channels: &[i32],
    ) {
        self.vertex_count = positions.len();

        let pos: &[u8]  = bytemuck::cast_slice(positions);
        let norm: &[u8] = bytemuck::cast_slice(normals);
        let chan: &[u8] = bytemuck::cast_slice(channels);

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[0]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, pos, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[1]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, norm, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[2]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, chan, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.point_vbos[0]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, pos, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.point_vbos[1]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, norm, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.point_vbos[2]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, chan, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    pub fn new(
        gl: &glow::Context,
        positions: &[Vec3],
        normals: &[Vec3],
        channels: &[i32],
    ) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            let vbos = [
                { let vbo = gl.create_buffer().unwrap(); gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)); gl.enable_vertex_attrib_array(0); gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0); vbo },
                { let vbo = gl.create_buffer().unwrap(); gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)); gl.enable_vertex_attrib_array(1); gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0); vbo },
                { let vbo = gl.create_buffer().unwrap(); gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)); gl.enable_vertex_attrib_array(2); gl.vertex_attrib_pointer_i32(2, 1, glow::INT, 0, 0); vbo },
            ];
            let instance_vbo = Self::setup_instance_attribs(gl);
            gl.bind_vertex_array(None);

            let point_vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(point_vao));
            let point_vbos = [
                { let vbo = gl.create_buffer().unwrap(); gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)); gl.enable_vertex_attrib_array(0); gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0); vbo },
                { let vbo = gl.create_buffer().unwrap(); gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)); gl.enable_vertex_attrib_array(1); gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0); vbo },
                { let vbo = gl.create_buffer().unwrap(); gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)); gl.enable_vertex_attrib_array(2); gl.vertex_attrib_pointer_i32(2, 1, glow::INT, 0, 0); vbo },
            ];
            let point_instance_vbo = Self::setup_instance_attribs(gl);
            gl.bind_vertex_array(None);

            let mut mesh = Self {
                vao, vbos, instance_vbo,
                point_vao, point_vbos, point_instance_vbo,
                vertex_count: 0,
            };
            mesh.rebuild(gl, positions, normals, channels);
            mesh
        }
    }

    pub fn draw_tris(&self, gl: &glow::Context, instances: &[InstanceData], wireframe: bool, renderer: &Renderer) {
        if instances.is_empty() { return; }
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            Self::upload_instances(gl, self.instance_vbo, instances);
            let n = instances.len() as i32;
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, self.vertex_count as i32, n);
            if wireframe {
                renderer.set_int(gl, renderer.mesh, "uWire", 1);
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                gl.line_width(0.5);
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, self.vertex_count as i32, n);
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                renderer.set_int(gl, renderer.mesh, "uWire", 0);
            }
            gl.bind_vertex_array(None);
        }
    }

    pub fn draw_points(&self, gl: &glow::Context, instances: &[InstanceData]) {
        if instances.is_empty() { return; }
        unsafe {
            gl.bind_vertex_array(Some(self.point_vao));
            Self::upload_instances(gl, self.point_instance_vbo, instances);
            gl.draw_arrays_instanced(glow::POINTS, 0, self.vertex_count as i32, instances.len() as i32);
            gl.bind_vertex_array(None);
        }
    }

    pub fn destroy(self, gl: &glow::Context) {
        unsafe {
            for vbo in self.vbos { gl.delete_buffer(vbo); }
            for vbo in self.point_vbos { gl.delete_buffer(vbo); }
            gl.delete_buffer(self.instance_vbo);
            gl.delete_buffer(self.point_instance_vbo);
            gl.delete_vertex_array(self.vao);
            gl.delete_vertex_array(self.point_vao);
        }
    }

    pub fn set_from_hashmap(gl: &glow::Context, mesh: &LightMesh, mut gpu_meshes: HashMap<String, Self>) -> HashMap<String, Self> {
        let mut out = HashMap::new();
        for (name, part) in mesh.parts.iter() {
            let mut gpu_mesh = gpu_meshes.remove(name).unwrap_or_else(|| GpuMesh::new(gl, &[], &[], &[]));
            gpu_mesh.set_from_light_mesh_part(gl, part, &mesh.data);
            out.insert(name.clone(), gpu_mesh);
        }
        for unused in gpu_meshes.into_values() {
            unused.destroy(gl);
        }
        out
    }

    pub fn from_light_mesh(gl: &glow::Context, mesh: &LightMesh) -> HashMap<String, Self> {
        Self::set_from_hashmap(gl, mesh, HashMap::new())
    }

    pub fn add_triangle_data(
        vertices: &mut Vec<Vec3>,
        normals: &mut Vec<Vec3>,
        channels: &mut Vec<i32>,
        part: &Part,
        data: &IndexMap<String, MaterialData>,
        transform: &Mat4,
        remap_data: &IndexMap<String, String>,
    ) {
        let mat3 = Mat3::from_mat4(*transform);
        let normal_transform = mat3.inverse().transpose();
        let flip = mat3.determinant() < 0.;

        for Triangle {
            vertices: [
                Vertex { vertex: v0, uv:_, normal: n0 },
                Vertex { vertex: v1, uv:_, normal: n1 },
                Vertex { vertex: v2, uv:_, normal: n2 }
            ],
            material
        } in part.triangles.0.iter() {
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

            if flip {
                verts.swap(0, 2);
                norms.swap(0, 2);
            }

            vertices.extend_from_slice(&verts);

            normals.extend_from_slice(&norms);

            let material = match material {
                Some(mat) => {
                    Some(if let Some(remap) = remap_data.get(mat) {
                        remap.as_str()
                    } else {
                        mat.as_str()
                    })
                }
                None => None
            };

            if let Some(MaterialData {
                material, texture:_, color
            }) = data.get(material.unwrap_or("default")) && *material != 0 {
                channels.extend_from_slice(&[*color as i32; 3]);
            } else {
                channels.extend_from_slice(&[8; 3]);
            }
        }

    }

    pub fn set_from_full_light_mesh(&mut self, gl: &glow::Context, light_mesh: &LightMesh) {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut channels = Vec::new();

        for placement in light_mesh.placements.iter() {
            let mat = placement.transform();
            let part = light_mesh.parts.get(&placement.part).unwrap();
            Self::add_triangle_data(&mut vertices, &mut normals, &mut channels, part, &light_mesh.data, &mat, &placement.remap_data);
        }

        self.rebuild(gl, &vertices, &normals, &channels);
    }

    pub fn set_from_light_mesh_part(&mut self, gl: &glow::Context, part: &Part, data: &IndexMap<String, MaterialData>) {

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut channels = Vec::new();

        Self::add_triangle_data(&mut vertices, &mut normals, &mut channels, part, data, &Mat4::IDENTITY, &IndexMap::default());

        self.rebuild(gl, &vertices, &normals, &channels);
    }
}

pub struct Renderer {
    pub mesh: glow::NativeProgram,
    pub point: glow::NativeProgram,
    pub flat: glow::NativeProgram,
    pub grid_vao: glow::NativeVertexArray,
    pub grid_n: i32,
    pub axis_vao: glow::NativeVertexArray,
}

impl Renderer {
    fn compile_shader(gl: &glow::Context, vs: &str, fs: &str) -> Result<glow::NativeProgram, String> {
        unsafe {
            let v = gl.create_shader(glow::VERTEX_SHADER).map_err(|e| e.to_string())?;
            gl.shader_source(v, vs); gl.compile_shader(v);
            if !gl.get_shader_compile_status(v) { return Err(gl.get_shader_info_log(v)); }
            let f = gl.create_shader(glow::FRAGMENT_SHADER).map_err(|e| e.to_string())?;
            gl.shader_source(f, fs); gl.compile_shader(f);
            if !gl.get_shader_compile_status(f) { return Err(gl.get_shader_info_log(f)); }
            let p = gl.create_program().map_err(|e| e.to_string())?;
            gl.attach_shader(p, v); gl.attach_shader(p, f); gl.link_program(p);
            if !gl.get_program_link_status(p) { return Err(gl.get_program_info_log(p)); }
            gl.delete_shader(v); gl.delete_shader(f);
            Ok(p)
        }
    }

    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let mesh = Self::compile_shader(gl, include_str!("./assets/shaders/mesh.vert"), include_str!("./assets/shaders/mesh.frag"))?;
            let point = Self::compile_shader(gl, include_str!("./assets/shaders/point.vert"), include_str!("./assets/shaders/point.frag"))?;
            let flat = Self::compile_shader(gl, include_str!("./assets/shaders/flat.vert"), include_str!("./assets/shaders/flat.frag"))?;

            let mut grid_pts: Vec<f32> = vec![];
            let (size, step) = (300i32, 10i32);
            let mut i = -size;
            while i <= size {
                let fi = i as f32; let fs = size as f32;
                grid_pts.extend_from_slice(&[fi,0.0,-fs, fi,0.0,fs, -fs,0.0,fi, fs,0.0,fi]);
                i += step;
            }
            let grid_vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(grid_vao));
            let gvbo = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(gvbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&grid_pts), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);
            let grid_n = (grid_pts.len() / 3) as i32;

            let ax: f32 = size as f32;
            let axis_pts: [f32; 12] = [0.0,0.0,0.0, ax,0.0,0.0, 0.0,0.0,0.0, 0.0,0.0,ax];
            let axis_vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(axis_vao));
            let avbo = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(avbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&axis_pts), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);

            gl.enable(glow::PROGRAM_POINT_SIZE);

            Ok(Self { mesh, point, flat, grid_vao, grid_n, axis_vao })
        }
    }

    pub fn draw_meshes(&self, gl: &glow::Context, vp: &Mat4, calls: &[MeshDrawCall<'_>]) {
        unsafe {
            gl.use_program(Some(self.mesh));
            self.set_mat4(gl, self.mesh, "uVP", vp);
            for call in calls {
                self.set_int(gl, self.mesh, "uWire", 0);
                call.mesh.draw_tris(gl, &call.instances, call.wireframe, self);
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

    fn set_mat4(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, m: &Mat4) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_matrix_4_f32_slice(Some(&l), false, &m.to_cols_array());
            }
        }
    }
    fn set_vec4(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: Vec4) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_4_f32(Some(&l), v.x, v.y, v.z, v.w);
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
    fn set_int(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: i32) {
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_1_i32(Some(&l), v);
            }
        }
    }
}
