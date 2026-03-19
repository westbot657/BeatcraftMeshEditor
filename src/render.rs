use eframe::glow::{self, HasContext};
use glam::{Mat4, Vec3, Vec4};


pub struct GpuMesh {
    pub vao: glow::NativeVertexArray,
    vbos: [glow::NativeBuffer; 3],
    pub ibo: glow::NativeBuffer,
    pub point_vao: glow::NativeVertexArray,
    point_vbos: [glow::NativeBuffer; 3],
    pub vertex_count: usize,
    pub triangle_count: usize,
}

impl GpuMesh {

    fn upload_f32(gl: &glow::Context, loc: u32, data: &[f32], span: i32) -> Option<glow::NativeBuffer> {
        unsafe {
            let vbo = gl.create_buffer().ok()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(data), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(loc);
            gl.vertex_attrib_pointer_f32(loc, span, glow::FLOAT, false, 0, 0);
            Some(vbo)
        }
    }

    fn upload_i32(gl: &glow::Context, loc: u32, data: &[i32], span: i32) -> Option<glow::NativeBuffer> {
        unsafe {
            let vbo = gl.create_buffer().ok()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(data), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(loc);
            gl.vertex_attrib_pointer_i32(loc, span, glow::INT, 0, 0);
            Some(vbo)
        }
    }

    pub fn new(
        gl: &glow::Context,
        positions: &[Vec3],
        normals: &[Vec3],
        channels: &[i32],
        tris: &[[u32; 3]]
    ) -> Option<Self> {
        if positions.is_empty() { return None; }

        let vertex_count = positions.len();
        let triangle_count = tris.len();

        let pos: &[f32] = bytemuck::cast_slice(positions);
        let norm: &[f32] = bytemuck::cast_slice(normals);
        let tris: &[u32] = bytemuck::cast_slice(tris);

        unsafe {
            let vao = gl.create_vertex_array().ok()?;
            gl.bind_vertex_array(Some(vao));
            let vbos = [
                Self::upload_f32(gl, 0, pos, 3)?,
                Self::upload_f32(gl, 1, norm, 3)?,
                Self::upload_i32(gl, 2, channels, 1)?,
            ];
            let ibo = gl.create_buffer().ok()?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(tris), glow::STATIC_DRAW);
            gl.bind_vertex_array(None);

            let point_vao = gl.create_vertex_array().ok()?;
            gl.bind_vertex_array(Some(point_vao));
            let point_vbos = [
                Self::upload_f32(gl, 0, pos, 3)?,
                Self::upload_f32(gl, 1, norm, 3)?,
                Self::upload_i32(gl, 2, channels, 1)?,
            ];
            gl.bind_vertex_array(None);

            Some(Self { vao, vbos, ibo, point_vao, point_vbos, vertex_count, triangle_count })
        }

    }

    pub fn draw_tris(&self, gl: &glow::Context, wireframe: bool) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ibo));
            gl.draw_elements(glow::TRIANGLES, (self.triangle_count * 3) as i32, glow::UNSIGNED_INT, 0);
            if wireframe {
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                gl.draw_elements(glow::TRIANGLES, (self.triangle_count * 3) as i32, glow::UNSIGNED_INT, 0);
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
            }
            gl.bind_vertex_array(None);
        }
    }

    pub fn draw_points(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.point_vao));
            gl.draw_arrays(glow::POINTS, 0, self.vertex_count as i32);
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
            gl.delete_buffer(self.ibo);
            gl.delete_vertex_array(self.vao);
            gl.delete_vertex_array(self.point_vao);
        }
    }

}

pub struct Renderer {
    mesh: glow::NativeProgram,
    point: glow::NativeProgram,
    flat: glow::NativeProgram,
    grid_vao: glow::NativeVertexArray,
    grid_n: i32,
    axis_vao: glow::NativeVertexArray,
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

            // Grid
            let mut grid_pts: Vec<f32> = vec![];
            let (size, step) = (120i32, 10i32);
            let mut i = -size;
            while i <= size {
                let fi = i as f32;
                let fs = size as f32;
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

            // Axis lines
            let ax: f32 = 120.0;
            let axis_pts: [f32; 12] = [0.0,0.0,0.0, ax,0.0,0.0, 0.0,0.0,0.0, 0.0,0.0,ax];
            let axis_vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(axis_vao));
            let avbo = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(avbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&axis_pts), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);

            Ok(Self { mesh, point, flat, grid_vao, grid_n, axis_vao })
        }
    }

    pub fn draw_grid(&self, gl: &glow::Context, mvp: &Mat4) {
        unsafe {
            use glow::HasContext;
            gl.use_program(Some(self.flat));
            self.set_mat4(gl, self.flat, "uMVP", mvp);
            self.set_vec4(gl, self.flat, "uColor", Vec4::new(0.27, 0.27, 0.34, 0.5));
            gl.bind_vertex_array(Some(self.grid_vao));
            gl.draw_arrays(glow::LINES, 0, self.grid_n);
            // Axes
            gl.line_width(2.0);
            self.set_vec4(gl, self.flat, "uColor", Vec4::new(0.85, 0.20, 0.20, 0.9)); // +X red
            gl.bind_vertex_array(Some(self.axis_vao));
            gl.draw_arrays(glow::LINES, 0, 2);
            self.set_vec4(gl, self.flat, "uColor", Vec4::new(0.20, 0.45, 0.90, 0.9)); // +Z blue
            gl.draw_arrays(glow::LINES, 2, 2);
            gl.line_width(1.0);
            gl.bind_vertex_array(None);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_mesh(
        &self,
        gl: &glow::Context,
        mesh: &GpuMesh,
        mvp: &Mat4,
        model: &Mat4,
        alpha: f32,
        part_color: Option<[f32; 3]>,
        wireframe: bool
    ) {
        unsafe {
            use glow::HasContext;
            gl.use_program(Some(self.mesh));
            self.set_mat4(gl, self.mesh, "uMVP", mvp);
            self.set_mat4(gl, self.mesh, "uNormalMat", model);
            self.set_float(gl, self.mesh, "uAlpha", alpha);
            if let Some(pc) = part_color {
                self.set_vec3(gl, self.mesh, "uPartColor", Vec3::from_array(pc));
                self.set_int(gl, self.mesh, "uUsePartColor", 1);
            } else {
                self.set_int(gl, self.mesh, "uUsePartColor", 0);
            }
            self.set_int(gl, self.mesh, "uWire", 0);
            mesh.draw_tris(gl, wireframe);
        }
    }

    pub fn draw_points(&self, gl: &glow::Context, mesh: &GpuMesh,
                               mvp: &Mat4, color: [f32; 3], size: f32) {
        unsafe {
            use glow::HasContext;
            gl.use_program(Some(self.point));
            self.set_mat4(gl, self.point, "uMVP", mvp);
            self.set_float(gl, self.point, "uPointSize", size);
            self.set_vec3(gl, self.point, "uColor", Vec3::from_array(color));
            mesh.draw_points(gl);
        }
    }

    fn set_mat4(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, m: &Mat4) {
        use glow::HasContext;
        unsafe {
            let loc = gl.get_uniform_location(prog, name);
            if let Some(l) = loc {
                gl.uniform_matrix_4_f32_slice(Some(&l), false, &m.to_cols_array());
            }
        }
    }

    fn set_vec4(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: Vec4) {
        use glow::HasContext;
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_4_f32(Some(&l), v.x, v.y, v.z, v.w);
            }
        }
    }
    fn set_vec3(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: Vec3) {
        use glow::HasContext;
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_3_f32(Some(&l), v.x, v.y, v.z);
            }
        }
    }
    fn set_float(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: f32) {
        use glow::HasContext;
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_1_f32(Some(&l), v);
            }
        }
    }
    fn set_int(&self, gl: &glow::Context, prog: glow::NativeProgram, name: &str, v: i32) {
        use glow::HasContext;
        unsafe {
            if let Some(l) = gl.get_uniform_location(prog, name) {
                gl.uniform_1_i32(Some(&l), v);
            }
        }
    }

}



