mod font;
pub mod texture;

use crate::{
    FontHandle, Renderer, SpriteSheetHandle, TextureHandle, UvRegion,
    batch::{
        BatchMode, FLOATS_PER_SPRITE, INDICES_PER_SPRITE, MAX_SPRITES, SpriteBatch, VERTEX_SIZE,
        WHITE,
    },
    opengl::{
        font::FontAtlas,
        texture::{SpriteSheet, Texture},
    },
};

use engine_math::{Camera2D, Mat4, Transform2D, Vec2};
use glow::{HasContext, UniformLocation};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    surface::{GlSurface, Surface, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use std::ffi::CString;
use winit::{event_loop::ActiveEventLoop, raw_window_handle::HasRawWindowHandle, window::Window};

pub struct OpenGLRenderer {
    gl: glow::Context,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    shader_program: glow::NativeProgram,
    uniform_model: Option<glow::UniformLocation>,
    uniform_projection: Option<glow::UniformLocation>,
    uniform_texture: Option<UniformLocation>,
    uniform_uv_min: Option<UniformLocation>,
    uniform_uv_max: Option<UniformLocation>,
    uniform_use_texture: Option<glow::UniformLocation>,
    uniform_is_text: Option<glow::UniformLocation>,
    quad: Mesh,
    textures: Vec<Texture>,
    sprite_sheets: Vec<SpriteSheet>,
    fonts: Vec<FontAtlas>,
    batch: SpriteBatch,
    viewport_size: (f32, f32),
    cached_projection: Mat4,
}

pub struct Mesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
}

impl Renderer for OpenGLRenderer {
    fn begin_frame(&mut self) {
        unsafe {
            self.gl.clear_color(0.1, 0.1, 0.2, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }
    fn present(&mut self) {
        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
    }
    fn end_frame(&mut self) {
        if self.batch.sprite_count > 0 {
            self.flush_batch();
        }
        unsafe {
            self.gl.bind_vertex_array(None);
        }
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.viewport_size = (width as f32, height as f32);
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
        }
        self.gl_surface.resize(
            &self.gl_context,
            std::num::NonZeroU32::new(width.max(1)).unwrap(),
            std::num::NonZeroU32::new(height.max(1)).unwrap(),
        );
    }

    fn draw_sprite(
        &mut self,
        transform: &Transform2D,
        spritesheet: SpriteSheetHandle,
        tile_index: u32,
    ) {
        let sheet = self.sprite_sheets.get(spritesheet.0 as usize).unwrap();
        let uv_region = sheet.uv_for_tile(tile_index);
        let target_mode = BatchMode::Textured(sheet.texture);

        if self.batch.sprite_count > 0 && self.batch.mode != target_mode {
            self.flush_batch();
        }
        self.batch.mode = target_mode;

        if self.batch.sprite_count >= MAX_SPRITES {
            self.flush_batch();
            self.batch.mode = target_mode;
        }

        self.batch.push_sprite(transform, &uv_region, WHITE);
    }
    fn draw_colored_rect(&mut self, transform: &Transform2D, r: f32, g: f32, b: f32, a: f32) {
        let target_mode = BatchMode::Untextured;

        if self.batch.sprite_count > 0 && self.batch.mode != target_mode {
            self.flush_batch();
        }
        self.batch.mode = target_mode;

        if self.batch.sprite_count >= MAX_SPRITES {
            self.flush_batch();
            self.batch.mode;
        }

        let uv = UvRegion {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        };
        self.batch.push_sprite(transform, &uv, [r, g, b, a]);

        unsafe {
            self.gl.uniform_1_i32(self.uniform_use_texture.as_ref(), 1);
        }
    }

    fn load_texture(&mut self, bytes: &[u8]) -> crate::TextureHandle {
        let texture = {
            let img = image::load_from_memory(bytes).unwrap().into_rgba8();
            Texture::from_rgba8(&self.gl, img.width(), img.height(), img.as_raw())
        };

        let index = self.textures.len();
        self.textures.push(texture);

        TextureHandle(index as u32)
    }

    fn create_sprite_sheet(
        &mut self,
        texture: crate::TextureHandle,
        tile_width: u32,
        tile_height: u32,
    ) -> crate::SpriteSheetHandle {
        let texture_index = texture.0 as usize;
        let texture_width = self.textures.get(texture_index).unwrap().width;
        let texture_height = self.textures.get(texture_index).unwrap().height;
        let sprite_sheet = SpriteSheet {
            texture,
            tile_width,
            tile_height,
            texture_width,
            texture_height,
        };
        let index = self.sprite_sheets.len();
        self.sprite_sheets.push(sprite_sheet);

        SpriteSheetHandle(index as u32)
    }

    fn uv_for_tile(&self, sheet: crate::SpriteSheetHandle, index: u32) -> UvRegion {
        let sprite_sheet = self.sprite_sheets.get(sheet.0 as usize).unwrap();

        sprite_sheet.uv_for_tile(index)
    }

    fn set_camera(&mut self, camera: &Camera2D) {
        let (w, h) = self.viewport_size;
        self.cached_projection = camera.projection_matrix(w, h);
    }

    fn clear_assets(&mut self) {
        for texture in &self.textures {
            unsafe {
                self.gl.delete_texture(texture.handle);
            }
        }
        for font in &self.fonts {
            unsafe {
                self.gl.delete_texture(font.texture.handle);
            }
        }
        self.textures.clear();
        self.sprite_sheets.clear();
        self.fonts.clear();
    }

    fn load_font(&mut self, bytes: &[u8], size: f32) -> crate::FontHandle {
        let atlas = FontAtlas::build(&self.gl, bytes, size);
        let index = self.fonts.len();
        self.fonts.push(atlas);

        FontHandle(index as u32)
    }

    fn draw_text(&mut self, text: &str, font: FontHandle, position: Vec2, color: [f32; 4]) {
        if self.batch.sprite_count > 0 {
            self.flush_batch();
        }

        let atlas = match self.fonts.get(font.0 as usize) {
            Some(a) => a,
            None => {
                eprintln!("draw_text: invalid FontHandle");
                return;
            }
        };

        unsafe {
            &self.gl.use_program(Some(self.shader_program));
            &self.gl.uniform_1_i32(self.uniform_is_text.as_ref(), 1);
            &self.gl.uniform_1_i32(self.uniform_use_texture.as_ref(), 1);
            &self.gl.active_texture(glow::TEXTURE0);
            &self
                .gl
                .bind_texture(glow::TEXTURE_2D, Some(atlas.texture.handle));
        }

        let mut cursor_x = position.x;

        for c in text.chars() {
            let Some(glyph) = atlas.glyphs.get(&c) else {
                continue;
            };

            if glyph.width > 0.0 && glyph.height > 0.0 {
                let x = cursor_x + glyph.offset_x + glyph.width / 2.0;
                let y = position.y + glyph.offset_y + glyph.height / 2.0;

                let transform = Transform2D {
                    position: Vec2::new(x, y),
                    rotation_in_radians: 0.0,
                    scale: Vec2::new(glyph.width, glyph.height),
                };
                let uv = UvRegion {
                    min: glyph.uv_min,
                    max: glyph.uv_max,
                };

                self.batch.push_sprite(&transform, &uv, color);
            }

            cursor_x += glyph.advance;
        }

        self.flush_batch_with_bound_texture();

        unsafe {
            &self.gl.uniform_1_i32(self.uniform_is_text.as_ref(), 0);
            &self.gl.uniform_1_i32(self.uniform_use_texture.as_ref(), 0);
            self.gl.bind_texture(glow::TEXTURE_2D, None);
        }
        self.batch.mode = BatchMode::Empty;
    }

    fn measure_text(&self, text: &str, font: FontHandle) -> Vec2 {
        match self.fonts.get(font.0 as usize) {
            Some(atlas) => atlas.measure_text(text),
            None => Vec2::ZERO,
        }
    }
}

impl OpenGLRenderer {
    pub fn new(
        gl: glow::Context,
        gl_context: PossiblyCurrentContext,
        gl_surface: Surface<WindowSurface>,
    ) -> OpenGLRenderer {
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        let vertex_shader_source = include_str!("shaders/quad.vert");
        let fragment_shader_source = include_str!("shaders/quad.frag");
        let vertex_shader =
            unsafe { compile_shader(&gl, glow::VERTEX_SHADER, vertex_shader_source) };

        let fragment_shader =
            unsafe { compile_shader(&gl, glow::FRAGMENT_SHADER, fragment_shader_source) };

        let shader_program = unsafe { gl.create_program().unwrap() };
        unsafe {
            gl.attach_shader(shader_program, vertex_shader);
            gl.attach_shader(shader_program, fragment_shader);
            gl.link_program(shader_program);
            if !gl.get_program_link_status(shader_program) {
                panic!(
                    "Program link error: {}",
                    gl.get_program_info_log(shader_program)
                );
            }
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader)
        };

        let (vao, vbo, ebo) = unsafe {
            (
                gl.create_vertex_array().unwrap(),
                gl.create_buffer().unwrap(),
                gl.create_buffer().unwrap(),
            )
        };
        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (MAX_SPRITES * FLOATS_PER_SPRITE * size_of::<f32>()) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                (MAX_SPRITES * INDICES_PER_SPRITE * size_of::<u32>()) as i32,
                glow::DYNAMIC_DRAW,
            );

            let stride = (VERTEX_SIZE * size_of::<f32>()) as i32;

            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                stride,
                (2 * size_of::<f32>()) as i32,
            );
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                2,
                4,
                glow::FLOAT,
                false,
                stride,
                (4 * size_of::<f32>()) as i32,
            );
            gl.enable_vertex_attrib_array(2);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        };

        let (
            uniform_model,
            uniform_projection,
            uniform_texture,
            uniform_uv_min,
            uniform_uv_max,
            uniform_use_texture,
            uniform_is_text,
        ) = unsafe {
            (
                gl.get_uniform_location(shader_program, "model"),
                gl.get_uniform_location(shader_program, "projection"),
                gl.get_uniform_location(shader_program, "tex"),
                gl.get_uniform_location(shader_program, "uv_min"),
                gl.get_uniform_location(shader_program, "uv_max"),
                gl.get_uniform_location(shader_program, "use_texture"),
                gl.get_uniform_location(shader_program, "is_text"),
            )
        };

        unsafe {
            gl.use_program(Some(shader_program));
            gl.uniform_1_i32(uniform_texture.as_ref(), 0);
            gl.uniform_1_i32(uniform_use_texture.as_ref(), 1);
            gl.uniform_1_i32(uniform_is_text.as_ref(), 0);
        }

        let quad = Mesh {
            vao,
            vbo,
            ebo,
            index_count: 6,
        };

        OpenGLRenderer {
            gl,
            gl_context,
            gl_surface,
            shader_program,
            uniform_model,
            uniform_projection,
            uniform_texture,
            uniform_uv_min,
            uniform_uv_max,
            uniform_use_texture,
            uniform_is_text,
            quad,
            viewport_size: (0.0, 0.0),
            textures: Vec::new(),
            sprite_sheets: Vec::new(),
            fonts: Vec::new(),
            batch: SpriteBatch::new(),
            cached_projection: Mat4::IDENTITY,
        }
    }

    fn flush_batch_with_bound_texture(&mut self) {
        if self.batch.sprite_count == 0 {
            return;
        }

        let gl = &self.gl;
        unsafe {
            gl.bind_vertex_array(Some(self.quad.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.quad.vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(&self.batch.vertices),
            );
            gl.buffer_sub_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(&self.batch.indices),
            );
            // no texture binding — caller already bound it
            gl.uniform_matrix_4_f32_slice(
                self.uniform_projection.as_ref(),
                false,
                &self.cached_projection.to_cols_array(),
            );
            gl.draw_elements(
                glow::TRIANGLES,
                (self.batch.sprite_count * 6) as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
        self.batch.clear();
    }

    fn flush_batch(&mut self) {
        if self.batch.sprite_count == 0 {
            return;
        }

        let gl = &self.gl;
        unsafe {
            gl.use_program(Some(self.shader_program));
            gl.bind_vertex_array(Some(self.quad.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.quad.vbo));

            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(&self.batch.vertices),
            );
            gl.buffer_sub_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(&self.batch.indices),
            );

            match self.batch.mode {
                BatchMode::Textured(texture_handle) => {
                    gl.uniform_1_i32(self.uniform_use_texture.as_ref(), 1);
                    gl.active_texture(glow::TEXTURE0);
                    gl.bind_texture(
                        glow::TEXTURE_2D,
                        Some(self.textures[texture_handle.0 as usize].handle),
                    );
                }
                BatchMode::Untextured => {
                    gl.uniform_1_i32(self.uniform_use_texture.as_ref(), 0);
                }
                BatchMode::Empty => {}
            }

            gl.uniform_matrix_4_f32_slice(
                self.uniform_projection.as_ref(),
                false,
                &self.cached_projection.to_cols_array(),
            );
            gl.draw_elements(
                glow::TRIANGLES,
                (self.batch.sprite_count * 6) as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
        self.batch.clear();
    }
}

unsafe fn compile_shader(gl: &glow::Context, shader_type: u32, source: &str) -> glow::Shader {
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("Shader compile error: {}", gl.get_shader_info_log(shader));
        }
        shader
    }
}

pub fn create_renderer(
    event_loop: &ActiveEventLoop,
    title: &str,
) -> (Window, Box<dyn crate::Renderer>) {
    let template_builder = ConfigTemplateBuilder::new();
    let display_builder = DisplayBuilder::new()
        .with_window_attributes(Some(Window::default_attributes().with_title(title)));

    let (window, gl_config) = display_builder
        .build(event_loop, template_builder, |configs| {
            configs
                .reduce(|a, b| {
                    if a.num_samples() > b.num_samples() {
                        a
                    } else {
                        b
                    }
                })
                .unwrap()
        })
        .unwrap();
    let window = window.unwrap();
    let context_attrs =
        ContextAttributesBuilder::new().build(Some(window.raw_window_handle().unwrap()));
    let not_current_ctx = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attrs)
            .unwrap()
    };

    let surface_attrs = window.build_surface_attributes(Default::default()).unwrap();
    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &surface_attrs)
            .unwrap()
    };

    let gl_context = not_current_ctx.make_current(&gl_surface).unwrap();

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            gl_config
                .display()
                .get_proc_address(CString::new(s).unwrap().as_c_str()) as *const _
        })
    };

    let size = window.inner_size();
    let mut renderer = OpenGLRenderer::new(gl, gl_context, gl_surface);
    renderer.resize(size.width, size.height);

    (window, Box::new(renderer))
}
