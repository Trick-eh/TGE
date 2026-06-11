mod texture;
use std::path::Path;

use crate::{Renderer, opengl::texture::Texture};

use engine_math::{Camera2D, Transform2D};
use glow::{HasContext, UniformLocation};

pub struct OpenGLRenderer {
    gl: glow::Context,
    shader_program: glow::NativeProgram,
    uniform_model: Option<glow::UniformLocation>,
    uniform_projection: Option<glow::UniformLocation>,
    uniform_texture: Option<UniformLocation>,
    quad: Mesh,
    texture: Texture,
    viewport_size: (f32, f32),
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
        // self.draw_quad();
    }
    fn end_frame(&mut self) {
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

    fn draw_sprite(&mut self, transform: &Transform2D, camera: &Camera2D) {
        self.draw_quad(
            transform,
            camera,
            self.viewport_size.0,
            self.viewport_size.1,
        );
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
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STATIC_DRAW,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                (5 * size_of::<f32>()) as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                (5 * size_of::<f32>()) as i32,
                (3 * size_of::<f32>()) as i32,
            );
            gl.enable_vertex_attrib_array(1);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        };

        let (uniform_model, uniform_projection, uniform_texture) = unsafe {
            (
                gl.get_uniform_location(shader_program, "model"),
                gl.get_uniform_location(shader_program, "projection"),
                gl.get_uniform_location(shader_program, "tex"),
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

        let texture = Texture::new(
            &gl,
            Path::new("engine-renderer/src/opengl/textures/test.png"),
        );

        OpenGLRenderer {
            gl,
            shader_program,
            uniform_model,
            uniform_projection,
            uniform_texture,
            quad,
            viewport_size: (0.0, 0.0),
            texture,
        }
    }
    fn draw_quad(&mut self, transform: &Transform2D, camera: &Camera2D, width: f32, height: f32) {
        let gl = &self.gl;
        unsafe { gl.use_program(Some(self.shader_program)) };
        unsafe { gl.active_texture(glow::TEXTURE0) };
        unsafe { gl.bind_texture(glow::TEXTURE_2D, Some(self.texture.handle)) };
        unsafe { gl.bind_vertex_array(Some(self.quad.vao)) };
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
