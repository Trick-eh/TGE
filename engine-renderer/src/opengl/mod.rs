mod font;
mod renderer_impl;
pub mod texture;

use crate::sprite_sheet::SpriteSheet;
use crate::{
    Renderer,
    batch::{
        BatchMode, FLOATS_PER_SPRITE, INDICES_PER_SPRITE, MAX_SPRITES, SpriteBatch, VERTEX_SIZE,
    },
    colors::BLACK,
    opengl::{font::FontAtlas, texture::Texture},
};

use engine_math::Mat4;
use glow::{HasContext, UniformLocation};
use glutin::surface::{GlSurface, SwapInterval};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{Surface, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::ffi::CString;
use winit::{event_loop::ActiveEventLoop, window::Window};

pub struct OpenGLRenderer {
    gl: glow::Context,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    shader_program: glow::NativeProgram,
    _uniform_model: Option<glow::UniformLocation>,
    uniform_projection: Option<glow::UniformLocation>,
    _uniform_texture: Option<UniformLocation>,
    _uniform_uv_min: Option<UniformLocation>,
    _uniform_uv_max: Option<UniformLocation>,
    uniform_use_texture: Option<glow::UniformLocation>,
    uniform_is_text: Option<glow::UniformLocation>,
    quad: Mesh,
    textures: Vec<Texture>,
    sprite_sheets: Vec<SpriteSheet>,
    fonts: Vec<FontAtlas>,
    batch: SpriteBatch,
    viewport_size: (f32, f32),
    cached_projection: Mat4,
    window: Window,
    clear_color: [f32; 4],
}

pub struct Mesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    _ebo: glow::Buffer,
    _index_count: i32,
}

impl OpenGLRenderer {
    pub fn new(
        gl: glow::Context,
        gl_context: PossiblyCurrentContext,
        gl_surface: Surface<WindowSurface>,
        window: Window,
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
            _ebo: ebo,
            _index_count: 6,
        };

        OpenGLRenderer {
            gl,
            gl_context,
            gl_surface,
            shader_program,
            _uniform_model: uniform_model,
            uniform_projection,
            _uniform_texture: uniform_texture,
            _uniform_uv_min: uniform_uv_min,
            _uniform_uv_max: uniform_uv_max,
            uniform_use_texture,
            uniform_is_text,
            quad,
            viewport_size: (0.0, 0.0),
            textures: Vec::new(),
            sprite_sheets: Vec::new(),
            fonts: Vec::new(),
            batch: SpriteBatch::new(),
            cached_projection: Mat4::IDENTITY,
            window,
            clear_color: BLACK,
        }
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
                    gl.uniform_1_i32(self.uniform_is_text.as_ref(), 0);
                    gl.active_texture(glow::TEXTURE0);
                    gl.bind_texture(
                        glow::TEXTURE_2D,
                        Some(self.textures[texture_handle.0 as usize].handle),
                    );
                }
                BatchMode::Text(font_handle) => {
                    gl.uniform_1_i32(self.uniform_use_texture.as_ref(), 1);
                    gl.uniform_1_i32(self.uniform_is_text.as_ref(), 1);
                    gl.active_texture(glow::TEXTURE0);
                    gl.bind_texture(
                        glow::TEXTURE_2D,
                        Some(self.fonts[font_handle.0 as usize].texture.handle),
                    )
                }
                BatchMode::Untextured => {
                    gl.uniform_1_i32(self.uniform_use_texture.as_ref(), 0);
                    gl.uniform_1_i32(self.uniform_is_text.as_ref(), 0);
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
    vsync: bool,
) -> Box<dyn crate::Renderer> {
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
        ContextAttributesBuilder::new().build(Some(window.window_handle().unwrap().as_raw()));
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

    let swap_interval = if vsync {
        SwapInterval::Wait(std::num::NonZero::new(1).unwrap())
    } else {
        SwapInterval::DontWait
    };
    if let Err(e) = gl_surface.set_swap_interval(&gl_context, swap_interval) {
        eprintln!("Failed to set swap interval (vsync={vsync}): {e:?}");
    }

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            gl_config
                .display()
                .get_proc_address(CString::new(s).unwrap().as_c_str()) as *const _
        })
    };

    let size = window.inner_size();
    let mut renderer = OpenGLRenderer::new(gl, gl_context, gl_surface, window);
    renderer.resize(size.width, size.height);

    Box::new(renderer)
}
