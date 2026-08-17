use eframe::glow::{self, HasContext};
use glam::{FloatExt, Mat4, Vec2};

use crate::DB_RENDER;
use crate::audio::Audio;
use crate::render::Renderer;

const SPECTROGRAM_MIN_ZOOM: f32 = 0.01;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GridStepSpacing {
    Thirds,
    Quarters,
}

impl GridStepSpacing {
    pub fn value(&self) -> f32 {
        match self {
            GridStepSpacing::Thirds => 1. / 3.,
            GridStepSpacing::Quarters => 0.25,
        }
    }
}

pub struct BeatmapRenderer {
    grid_shader: glow::NativeProgram,
    placement_grid_shader: glow::NativeProgram,
    world_plane_shader: glow::NativeProgram,
    grid_vao: glow::VertexArray,

    spectrogram_ui_shader: glow::NativeProgram,
    spectrogram_ui_vao: glow::VertexArray,

    pub beat_spacing: f32,
    pub step_spacing: GridStepSpacing,
    pub beat_offset: (u16, f32),
    pub beats_before: u16,
    pub visible_beat_count: u8,
    grid_width: f32,
    digit_size: Vec2,
    digit_position: Vec2,
    beat_line_px: f32,
    thick_line_px: f32,
    thin_line_px: f32,
    digits_tex: glow::Texture,

    pub placement_z: f32,
    pub placement_r: f32,
    pub hovered_placement_cell: u32,

    spectrogram_ui_zoom: f32,
    spectrogram_ui_offset: f32,
}

impl BeatmapRenderer {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let span = tracing::debug_span!("beatmap-renderer");
            let _guard = span.enter();

            tracing::debug!(target: DB_RENDER, "Compiling beatmap grid shader");
            let grid_shader = Renderer::build_geo_program(
                gl,
                include_str!("../assets/shaders/beatmap_grid/beat_grid.vsh"),
                include_str!("../assets/shaders/beatmap_grid/beat_grid.gsh"),
                include_str!("../assets/shaders/beatmap_grid/beat_grid.fsh"),
            )?;

            tracing::debug!(target: DB_RENDER, "Compiling beatmap placement grid shader");
            let placement_grid_shader = Renderer::build_program(
                gl,
                include_str!("../assets/shaders/beatmap_grid/placement_grid.vsh"),
                include_str!("../assets/shaders/beatmap_grid/placement_grid.fsh"),
            )?;

            tracing::debug!(target: DB_RENDER, "Compiling spectrogram UI shader");
            let spectrogram_ui_shader = Renderer::build_program(
                gl,
                include_str!("../assets/shaders/spectrogram.vsh"),
                include_str!("../assets/shaders/spectrogram.fsh"),
            )?;

            tracing::debug!(target: DB_RENDER, "Compiling world plane shader");
            let world_plane_shader = Renderer::build_program(
                gl,
                include_str!("../assets/shaders/beatmap_grid/world_plane.vsh"),
                include_str!("../assets/shaders/beatmap_grid/world_plane.fsh"),
            )?;

            let grid_vao = gl.create_vertex_array()?;
            let spectrogram_ui_vao = gl.create_vertex_array()?;

            tracing::debug!(target: DB_RENDER, "Uploading digit texture to GPU");
            let digits_tex = {
                let dg = image::load_from_memory(include_bytes!("../assets/textures/digits.png"))
                    .unwrap()
                    .to_rgba8();

                let (w, h) = dg.dimensions();
                let pixels = dg.into_raw();

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
                    w as i32,
                    h as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(pixels.as_slice())),
                );

                tex
            };

            Ok(Self {
                grid_shader,
                placement_grid_shader,
                world_plane_shader,
                grid_vao,

                spectrogram_ui_shader,
                spectrogram_ui_vao,

                beat_spacing: 8.,
                step_spacing: GridStepSpacing::Quarters,
                beat_offset: (0, 0.),
                beats_before: 4,
                visible_beat_count: 20,
                grid_width: 1.5,
                digit_size: Vec2::splat(0.5),
                digit_position: Vec2::new(0., 0.),
                beat_line_px: 3.,
                thick_line_px: 2.,
                thin_line_px: 1.,
                digits_tex,

                placement_z: 0.,
                placement_r: 0.,
                hovered_placement_cell: 12,

                spectrogram_ui_zoom: 1.,
                spectrogram_ui_offset: 0.,
            })
        }
    }

    pub fn render_grid(&self, renderer: &Renderer, gl: &glow::Context, view: &Mat4, proj: &Mat4) {
        unsafe {
            let vp = *proj * *view;
            gl.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ZERO,
                glow::ONE,
            );
            gl.bind_vertex_array(Some(self.grid_vao));

            // Lanes
            let grid = self.world_plane_shader;
            gl.use_program(Some(grid));
            gl.depth_mask(false);
            renderer.set_mat4(gl, grid, "u_view", view);
            renderer.set_mat4(gl, grid, "u_proj", proj);
            renderer.set_float(gl, grid, "u_rotation", self.placement_r);
            renderer.set_float(gl, grid, "u_z", self.placement_z);
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.depth_mask(true);

            // Timeline markers
            let grid = self.grid_shader;
            gl.use_program(Some(grid));
            renderer.set_mat4(gl, grid, "u_view_proj", &vp);
            renderer.set_float(gl, grid, "u_beat_spacing", self.beat_spacing);
            renderer.set_float(gl, grid, "u_step_spacing", self.step_spacing.value());
            renderer.set_int(gl, grid, "u_beat_i", self.beat_offset.0 as i32);
            renderer.set_float(gl, grid, "u_beat_f", self.beat_offset.1);
            renderer.set_int(gl, grid, "u_beats_before", self.beats_before as i32);
            renderer.set_float(gl, grid, "u_track_width", self.grid_width);
            renderer.set_vec2(gl, grid, "u_digit_offset", self.digit_position);
            renderer.set_vec2(gl, grid, "u_digit_size", self.digit_size);
            renderer.set_float(gl, grid, "u_beat_line_px", self.beat_line_px);
            renderer.set_float(gl, grid, "u_thick_line_px", self.thick_line_px);
            renderer.set_float(gl, grid, "u_thin_line_px", self.thin_line_px);
            renderer.set_sampler(gl, grid, "u_digit_tex", Some(self.digits_tex), 0);
            gl.draw_arrays(glow::POINTS, 0, self.visible_beat_count as i32);

            // placement grid
            let grid = self.placement_grid_shader;
            gl.use_program(Some(grid));
            renderer.set_mat4(gl, grid, "u_view_proj", &vp);
            renderer.set_float(gl, grid, "u_z", self.placement_z);
            renderer.set_float(gl, grid, "u_rotation", self.placement_r);
            renderer.set_uint(gl, grid, "u_hovered_cell", self.hovered_placement_cell);

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            gl.bind_vertex_array(None);
            gl.blend_func_separate(
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE_MINUS_SRC_COLOR,
                glow::ONE,
            );
        }
    }

    pub fn scroll(&mut self, step: f32) {
        let (b, o) = &mut self.beat_offset;

        let total = (*b as f64 + *o as f64 + step as f64).max(0.0);

        let mut new_b = total.floor();
        let mut new_f = (total - new_b) as f32;

        if new_f >= 1.0 {
            new_b += 1.0;
            new_f = 0.0;
        }

        *b = new_b.min(u16::MAX as f64) as u16;
        *o = new_f;
    }

    pub fn spectrogram_center(&mut self, cursor: f32) {
        let zoom = self.spectrogram_ui_zoom;
        self.spectrogram_ui_offset = (cursor - zoom * 0.5).clamp(0.0, (1.0 - zoom).max(0.0));
    }

    pub fn spectrogram_zoom(&mut self, scroll: f32, cursor: f32) {
        if scroll == 0.0 {
            return;
        }
        let factor = if scroll > 0. { 0.88 } else { 1.12 };
        self.spectrogram_ui_zoom =
            (self.spectrogram_ui_zoom * factor).clamp(SPECTROGRAM_MIN_ZOOM, 1.0);
        self.spectrogram_center(cursor);
    }

    pub fn seek(&mut self, beat: f32) {
        self.beat_offset = (beat.trunc() as u16, beat.fract());
    }

    pub fn beat(&self) -> f32 {
        self.beat_offset.0 as f32 + self.beat_offset.1
    }

    pub fn spectrogram_range(&self) -> (f32, f32) {
        (
            self.spectrogram_ui_offset,
            self.spectrogram_ui_offset + self.spectrogram_ui_zoom,
        )
    }

    pub fn render_spectrogram_ui(
        &self,
        renderer: &mut Renderer,
        gl: &glow::Context,
        audio: &Audio,
        second: f32,
        length_seconds: f32,
    ) {
        unsafe {
            let cursor = second / length_seconds;
            renderer.beatmap.spectrogram_center(cursor);
            let program = self.spectrogram_ui_shader;
            gl.use_program(Some(program));
            gl.bind_vertex_array(Some(self.spectrogram_ui_vao));

            let Some(tex) = audio.get_spectrogram_tex(gl) else {
                return;
            };
            let start = self.spectrogram_ui_offset;
            let end =
                start + (0f32.lerp(length_seconds, self.spectrogram_ui_zoom) / length_seconds);
            let coverage = audio.spectrogram_synced_coverage();

            renderer.set_float(gl, program, "u_start", start);
            renderer.set_float(gl, program, "u_end", end);
            renderer.set_float(gl, program, "u_cursor", cursor);
            renderer.set_float(gl, program, "u_coverage", coverage);

            renderer.set_sampler(gl, program, "u_texture", Some(tex), 0);
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}
