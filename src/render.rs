use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::glow::{self, HasContext};
use glam::{FloatExt, Mat3, Mat4, Vec2, Vec3, Vec4};
use indexmap::IndexMap;

use crate::data::MaterialData;
use crate::light_mesh::{LightMesh, Part, Triangle, Vertex};

static MISSING_TEXTURE_BYTES: &[u8] = include_bytes!("./assets/textures/missing.png");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model: Mat4,
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

pub struct HandleDrawCall<'a> {
    pub mesh: &'a GpuMesh,
    pub instances: Vec<InstanceData>,
}

pub struct GpuMesh {
    pub vao: glow::NativeVertexArray,
    pub vbos: [glow::NativeBuffer; 4],
    pub instance_vbo: glow::NativeBuffer,

    pub point_vao: glow::NativeVertexArray,
    pub point_vbos: [glow::NativeBuffer; 3],
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

            for col in 0..4u32 {
                gl.enable_vertex_attrib_array(4 + col);
                gl.vertex_attrib_pointer_f32(4 + col, 4, glow::FLOAT, false, stride, offset);
                gl.vertex_attrib_divisor(4 + col, 1);
                offset += 16;
            }

            gl.enable_vertex_attrib_array(8);
            gl.vertex_attrib_pointer_f32(8, 4, glow::FLOAT, false, stride, offset);
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

    #[allow(clippy::too_many_arguments)]
    pub fn rebuild(
        &mut self,
        gl: &glow::Context,
        positions: &[Vec3],
        uvs: &[Vec2],
        normals: &[Vec3],
        channels: &[i32],
        point_positions: &[Vec3],
        point_normals: &[Vec3],
        point_channels: &[i32],
    ) {
        self.vertex_count = positions.len();
        self.point_count = point_positions.len();

        let pos: &[u8] = bytemuck::cast_slice(positions);
        let uvs: &[u8] = bytemuck::cast_slice(uvs);
        let norm: &[u8] = bytemuck::cast_slice(normals);
        let chan: &[u8] = bytemuck::cast_slice(channels);

        let p_pos: &[u8] = bytemuck::cast_slice(point_positions);
        let p_norm: &[u8] = bytemuck::cast_slice(point_normals);
        let p_chan: &[u8] = bytemuck::cast_slice(point_channels);

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[0]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, pos, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[1]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, uvs, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[2]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, norm, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbos[3]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, chan, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.point_vbos[0]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, p_pos, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.point_vbos[1]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, p_norm, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.point_vbos[2]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, p_chan, glow::DYNAMIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gl: &glow::Context,
        positions: &[Vec3], uvs: &[Vec2], normals: &[Vec3], channels: &[i32],
        point_positions: &[Vec3], point_normals: &[Vec3], point_channels: &[i32],
    ) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            let vbos = [
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(0);
                    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
                    vbo
                },
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(1);
                    gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
                    vbo
                },
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(2);
                    gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, 0, 0);
                    vbo
                },
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(3);
                    gl.vertex_attrib_pointer_i32(3, 1, glow::INT, 0, 0);
                    vbo
                },
            ];
            let instance_vbo = Self::setup_instance_attribs(gl);
            gl.bind_vertex_array(None);

            let point_vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(point_vao));
            let point_vbos = [
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(0);
                    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
                    vbo
                },
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(1);
                    gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0);
                    vbo
                },
                {
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.enable_vertex_attrib_array(2);
                    gl.vertex_attrib_pointer_i32(2, 1, glow::INT, 0, 0);
                    vbo
                },
            ];
            let point_instance_vbo = Self::setup_instance_attribs(gl);
            gl.bind_vertex_array(None);

            let mut mesh = Self {
                vao,
                vbos,
                instance_vbo,
                point_vao,
                point_vbos,
                point_instance_vbo,
                vertex_count: 0,
                point_count: 0,
            };
            mesh.rebuild(gl, positions, uvs, normals, channels, point_positions, point_normals, point_channels);
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
            for vbo in self.point_vbos {
                gl.delete_buffer(vbo);
            }
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
    ) -> HashMap<String, Self> {
        let mut out = HashMap::new();
        for (name, part) in mesh.parts.iter() {
            let mut gpu_mesh = gpu_meshes
                .remove(name)
                .unwrap_or_else(|| GpuMesh::new(gl, &[], &[], &[], &[], &[], &[], &[]));
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

    #[allow(clippy::too_many_arguments)]
    pub fn add_triangle_data(
        vertices: &mut Vec<Vec3>,
        uvs: &mut Vec<Vec2>,
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
            let mut uvs2 = [
                part.resolve_uv(u0),
                part.resolve_uv(u1),
                part.resolve_uv(u2),
            ];

            if flip {
                verts.swap(0, 2);
                norms.swap(0, 2);
                uvs2.swap(0, 2);
            }

            vertices.extend_from_slice(&verts);
            uvs.extend_from_slice(&uvs2);
            normals.extend_from_slice(&norms);

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
                channels.extend_from_slice(&[*color as i32; 3]);
            } else {
                channels.extend_from_slice(&[8; 3]);
            }
        }
    }

    pub fn add_point_data(
        points: &mut Vec<Vec3>,
        part: &Part,
        tranform: &Mat4,
    ) {
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

    pub fn set_from_full_light_mesh(&mut self, gl: &glow::Context, light_mesh: &LightMesh) {
        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        let mut normals = Vec::new();
        let mut channels = Vec::new();

        for placement in light_mesh.placements.iter() {
            let mat = placement.transform();
            let part = light_mesh.parts.get(&placement.part).unwrap();
            Self::add_triangle_data(
                &mut vertices,
                &mut uvs,
                &mut normals,
                &mut channels,
                part,
                &light_mesh.data,
                &mat,
                &placement.remap_data,
            );
        }

        self.rebuild(gl, &vertices, &uvs, &normals, &channels, &[], &[], &[]);
    }

    pub fn points_from_light_mesh(&mut self, gl: &glow::Context, light_mesh: &LightMesh) {
        let mut vertices = Vec::new();
        let mut points = Vec::new();

        let mut channels = Vec::new();
        let mut p_channels = Vec::new();
        let mut normals = Vec::new();

        let mut circle_plane = |pos: Vec3, axis: Vec3, a: Vec3, b: Vec3, c: i32| {
            let a = a * 0.5;
            let b = b * 0.5;
            let p0 = pos + axis + a + b;
            let p1 = pos + axis + a - b;
            let p2 = pos + axis - a - b;
            let p3 = pos + axis - a + b;
            let n0 = Vec3::new(-1., -1., 0.);
            let n1 = Vec3::new(-1., 1., 0.);
            let n2 = Vec3::new(1., 1., 0.);
            let n3 = Vec3::new(1., -1., 0.);
            vertices.extend_from_slice(&[p0, p3, p1, p1, p3, p2]);
            channels.extend_from_slice(&[c; 6]);
            normals.extend_from_slice(&[n0, n3, n1, n1, n3, n2]);
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
            p_channels.extend_from_slice(&[0,0,0, 1,1,1, 2,2,2, 3,3,3]);

        }

        let p_normals = vec![Vec3::Y; points.len()];
        self.rebuild(gl, &vertices, &[], &normals, &channels, &points, &p_normals, &p_channels);

    }

    pub fn set_from_light_mesh_part(
        &mut self,
        gl: &glow::Context,
        part: &Part,
        data: &IndexMap<String, MaterialData>,
    ) {
        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        let mut normals = Vec::new();
        let mut channels = Vec::new();
        let mut points = Vec::new();

        Self::add_triangle_data(
            &mut vertices,
            &mut uvs,
            &mut normals,
            &mut channels,
            part,
            data,
            &Mat4::IDENTITY,
            &IndexMap::default(),
        );
        Self::add_point_data(
            &mut points,
            part,
            &Mat4::IDENTITY
        );

        let p_normals = vec![Vec3::ZERO; points.len()];
        let p_channels = vec![0; 3 * points.len()];

        self.rebuild(gl, &vertices, &uvs, &normals, &channels, &points, &p_normals, &p_channels);
    }
}

pub struct Renderer {
    pub mesh: glow::NativeProgram,
    pub point: glow::NativeProgram,
    pub flat: glow::NativeProgram,
    pub handles: glow::NativeProgram,
    pub handle_points: glow::NativeProgram,
    pub grid_vao: glow::NativeVertexArray,
    pub grid_n: i32,
    pub axis_vao: glow::NativeVertexArray,
    pub blue_noise: glow::NativeTexture,
    pub missing_texture: glow::NativeTexture,
    /// maps a texture id ('beatcraft:textures/...') to a real path.
    pub texture_paths: HashMap<String, PathBuf>,
    /// the single atlas texture
    pub atlas: Option<glow::NativeTexture>,
    /// maps a path to its UV rect (x0, y0, x1, y1) within the atlas
    pub atlas_map: HashMap<PathBuf, Vec4>,
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
                include_str!("./assets/shaders/handles.frag")
            )?;

            let handle_points = Self::compile_shader(
                gl,
                include_str!("./assets/shaders/handle_points.vert"),
                include_str!("./assets/shaders/handle_points.frag")
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
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
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
                mesh, point, flat, handles, handle_points,
                grid_vao, grid_n, axis_vao,
                blue_noise,
                missing_texture,
                texture_paths: HashMap::new(),
                atlas: None,
                atlas_map: HashMap::new(),
            })
        }
    }

    pub fn rebuild_atlases(&mut self, gl: &glow::Context) {
        const ATLAS_SIZE: u32 = 1024;

        // Destroy old atlas if present
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

        // Collect unique paths (multiple texture IDs may share the same path)
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
                path, w, h, ATLAS_SIZE, ATLAS_SIZE
            );

            // Advance to next shelf if needed
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

            // Half-texel inset so bilinear sampling never bleeds into a neighbour
            let px = 0.5 / ATLAS_SIZE as f32;
            let u0 = shelf_x as f32 / ATLAS_SIZE as f32 + px;
            let v0 = shelf_y as f32 / ATLAS_SIZE as f32 + px;
            let u1 = (shelf_x + w) as f32 / ATLAS_SIZE as f32 - px;
            let v1 = (shelf_y + h) as f32 / ATLAS_SIZE as f32 - px;
            self.atlas_map.insert(path.clone(), Vec4::new(u0, v0, u1, v1));

            shelf_x += w;
            shelf_h = shelf_h.max(h);
        }

        // Upload to GPU
        unsafe {
            let tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
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
            Vec2::new(
                rect.x.lerp(rect.z, uv.x),
                rect.y.lerp(rect.w, uv.y),
            )
        } else {
            uv
        }
    }

    pub fn draw_meshes(&self, gl: &glow::Context, vp: &Mat4, calls: &[MeshDrawCall<'_>]) {
        unsafe {
            gl.use_program(Some(self.mesh));
            self.set_mat4(gl, self.mesh, "uVP", vp);
            self.set_sampler(gl, self.mesh, "uTexture", self.atlas, 0);
            self.set_sampler(gl, self.mesh, "uNoise", Some(self.blue_noise), 1);
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
    //
    // fn set_vec3(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: Vec3) {
    //     unsafe {
    //         if let Some(l) = gl.get_uniform_location(prog, name) {
    //             gl.uniform_3_f32(Some(&l), v.x, v.y, v.z);
    //         }
    //     }
    // }
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
