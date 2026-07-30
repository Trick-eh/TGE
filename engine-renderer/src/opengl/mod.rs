pub mod texture;

use crate::{
    Renderer, SpriteSheetHandle, TextureHandle, UvRegion,
    opengl::texture::{SpriteSheet, Texture},
};

use engine_math::{Camera2D, Mat4, Transform2D, Vec2};
use glow::{DYNAMIC_DRAW, HasContext, UniformLocation};

const MAX_SPRITES: usize = 1000;
const VERTEX_SIZE: usize = 4;
const FLOATS_PER_SPRITE: usize = 4 * VERTEX_SIZE;
const INDICES_PER_SPRITE: usize = 6;

pub struct SpriteBatch {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub sprite_count: usize,
    pub current_texture: Option<TextureHandle>,
}
impl SpriteBatch {
    pub fn new() -> SpriteBatch {
        SpriteBatch {
            vertices: Vec::with_capacity(MAX_SPRITES * FLOATS_PER_SPRITE),
            indices: Vec::with_capacity(MAX_SPRITES * INDICES_PER_SPRITE),
            sprite_count: 0,
            current_texture: None,
        }
    }
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.sprite_count = 0;
        self.current_texture = None;
    }
    pub fn push_sprite(&mut self, transform: &Transform2D, uv: &UvRegion) {
        let half_w = transform.scale.x / 2.0;
        let half_h = transform.scale.y / 2.0;

        let corners = [
            Vec2::new(-half_w, -half_h),
            Vec2::new(half_w, -half_h),
            Vec2::new(half_w, half_h),
            Vec2::new(-half_w, half_h),
        ];

        let uvs = [
            Vec2::new(uv.min.x, uv.min.y),
            Vec2::new(uv.max.x, uv.min.y),
            Vec2::new(uv.max.x, uv.max.y),
            Vec2::new(uv.min.x, uv.max.y),
        ];

        let cos = transform.rotation_in_radians.cos();
        let sin = transform.rotation_in_radians.sin();

        for (corner, uv) in corners.iter().zip(uvs.iter()) {
            let rotated = Vec2::new(
                corner.x * cos - corner.y * sin,
                corner.x * sin + corner.y * cos,
            );

            let world = rotated + transform.position;

            self.vertices
                .extend_from_slice(&[world.x, world.y, uv.x, uv.y])
        }

        let base = (self.sprite_count * 4) as u32;
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        self.sprite_count += 1;
    }
}

pub struct OpenGLRenderer {
    gl: glow::Context,
    shader_program: glow::NativeProgram,
    uniform_model: Option<glow::UniformLocation>,
    uniform_projection: Option<glow::UniformLocation>,
    uniform_texture: Option<UniformLocation>,
    uniform_uv_min: Option<UniformLocation>,
    uniform_uv_max: Option<UniformLocation>,
    quad: Mesh,
    textures: Vec<Texture>,
    sprite_sheets: Vec<SpriteSheet>,
    viewport_size: (f32, f32),
    batch: SpriteBatch,
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
    }

    fn draw_sprite(
        &mut self,
        transform: &Transform2D,
        spritesheet: SpriteSheetHandle,
        tile_index: u32,
    ) {
        let sheet = self.sprite_sheets.get(spritesheet.0 as usize).unwrap();

        let uv_region = sheet.uv_for_tile(tile_index);
        let texture_index = sheet.texture.0;
        let texture = self.sprite_sheets[texture_index as usize].texture;

        if self.batch.sprite_count > 0 && self.batch.current_texture != Some(texture) {
            self.flush_batch();
            self.batch.current_texture = Some(texture);
        }
        if self.batch.sprite_count >= MAX_SPRITES {
            self.flush_batch();
            self.batch.current_texture = Some(texture);
        }

        self.batch.push_sprite(transform, &uv_region);

        // self.draw_quad(
        //     transform,
        //     camera,
        //     &uv_region,
        //     texture_index as usize,
        //     self.viewport_size.0,
        //     self.viewport_size.1,
        // );
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
}

impl OpenGLRenderer {
    pub fn new(gl: glow::Context) -> OpenGLRenderer {
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

        let vertices: &[f32] = &[
            -0.5, -0.5, 0.0, 0.0, 0.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.0, 1.0, 1.0, -0.5,
            0.5, 0.0, 0.0, 1.0,
        ];
        let indices: &[u32] = &[0, 1, 2, 0, 2, 3];

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
            // gl.buffer_data_u8_slice(
            //     glow::ARRAY_BUFFER,
            //     bytemuck::cast_slice(vertices),
            //     glow::STATIC_DRAW,
            // );
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (MAX_SPRITES * FLOATS_PER_SPRITE * size_of::<f32>()) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            // gl.buffer_data_u8_slice(
            //     glow::ELEMENT_ARRAY_BUFFER,
            //     bytemuck::cast_slice(indices),
            //     glow::STATIC_DRAW,
            // );
            gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                (MAX_SPRITES * INDICES_PER_SPRITE * size_of::<u32>()) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.vertex_attrib_pointer_f32(
                0,
                2,
                glow::FLOAT,
                false,
                (4 * size_of::<f32>()) as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                (4 * size_of::<f32>()) as i32,
                (2 * size_of::<f32>()) as i32,
            );
            gl.enable_vertex_attrib_array(1);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        };

        let (uniform_model, uniform_projection, uniform_texture, uniform_uv_min, uniform_uv_max) = unsafe {
            (
                gl.get_uniform_location(shader_program, "model"),
                gl.get_uniform_location(shader_program, "projection"),
                gl.get_uniform_location(shader_program, "tex"),
                gl.get_uniform_location(shader_program, "uv_min"),
                gl.get_uniform_location(shader_program, "uv_max"),
            )
        };

        unsafe {
            gl.use_program(Some(shader_program));
            gl.uniform_1_i32(uniform_texture.as_ref(), 0);
        }

        let quad = Mesh {
            vao,
            vbo,
            ebo,
            index_count: 6,
        };

        OpenGLRenderer {
            gl,
            shader_program,
            uniform_model,
            uniform_projection,
            uniform_texture,
            uniform_uv_min,
            uniform_uv_max,
            quad,
            viewport_size: (0.0, 0.0),
            textures: Vec::new(),
            sprite_sheets: Vec::new(),
            batch: SpriteBatch::new(),
            cached_projection: Mat4::IDENTITY,
        }
    }
    fn draw_quad(
        &mut self,
        transform: &Transform2D,
        camera: &Camera2D,
        uv_region: &UvRegion,
        texture_index: usize,
        width: f32,
        height: f32,
    ) {
        let gl = &self.gl;
        unsafe {
            gl.use_program(Some(self.shader_program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(
                glow::TEXTURE_2D,
                Some(self.textures.get(texture_index).unwrap().handle),
            );
            gl.bind_vertex_array(Some(self.quad.vao))
        };
        let model = transform.to_matrix();
        let projection = camera.projection_matrix(width, height);
        unsafe {
            gl.uniform_matrix_4_f32_slice(
                self.uniform_model.as_ref(),
                false,
                &model.to_cols_array(),
            );
            gl.uniform_matrix_4_f32_slice(
                self.uniform_projection.as_ref(),
                false,
                &projection.to_cols_array(),
            );
            gl.uniform_2_f32_slice(self.uniform_uv_min.as_ref(), &uv_region.min.to_array());
            gl.uniform_2_f32_slice(self.uniform_uv_max.as_ref(), &uv_region.max.to_array());
        };
        unsafe {
            gl.draw_elements(
                glow::TRIANGLES,
                self.quad.index_count,
                glow::UNSIGNED_INT,
                0,
            )
        };
    }

    fn flush_batch(&mut self) {
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

            if let Some(texture_handle) = self.batch.current_texture {
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(
                    glow::TEXTURE_2D,
                    Some(self.textures[texture_handle.0 as usize].handle),
                );
            }

            let projection = &self.cached_projection;
            gl.uniform_matrix_4_f32_slice(
                self.uniform_projection.as_ref(),
                false,
                &projection.to_cols_array(),
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
