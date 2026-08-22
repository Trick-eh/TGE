use ash::vk;
use engine_math::{Camera2D, Transform2D, Vec2};

use crate::{
    FontHandle, Renderer, SpriteSheetHandle, TextureHandle, UvRegion,
    batch::{BatchMode, MAX_SPRITES},
    colors::WHITE,
    font_atlas::{measure_text, pack_atlas},
    sprite_sheet::SpriteSheet,
    vulkan::{
        VulkanRenderer, buffers,
        commands::MAX_FRAMES_IN_FLIGHT,
        descriptor::{self, create_descriptor_set},
        texture::{self, LoadedFont, create_image_view},
    },
};

impl Renderer for VulkanRenderer {
    fn begin_frame(&mut self) {
        let fence = self.sync.in_flight_fences[self.current_frame];

        unsafe {
            self.logical_device
                .wait_for_fences(&[fence], true, u64::MAX)
                .expect("Failed to wait for in-flight fence");
        }

        buffers::release_retired_buffers(
            &self.logical_device,
            &mut self.batch_buffers[self.current_frame],
        );

        let image_index = loop {
            let result = unsafe {
                self.swapchain.loader.acquire_next_image(
                    self.swapchain.swapchain,
                    u64::MAX,
                    self.sync.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
            };

            match result {
                Ok((index, suboptimal)) => {
                    if suboptimal {
                        self.needs_recreation = true;
                    }
                    break index;
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain();
                }
                Err(e) => panic!("Failed to acquire swapchain image: {e:?}"),
            }
        };
        self.current_image_index = image_index;

        self.current_frame_vertex_offset = 0;
        self.current_frame_index_offset = 0;

        unsafe {
            self.logical_device
                .reset_fences(&[fence])
                .expect("Failed to reset in-flight fence");
        }

        let command_buffer = self.command_buffers[self.current_frame];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.logical_device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin recording command buffer");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: self.clear_color,
            },
        }];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
        }

        unsafe {
            self.logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            let viewport = vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(self.swapchain.extent.width as f32)
                .height(self.swapchain.extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);
            self.logical_device
                .cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            };
            self.logical_device
                .cmd_set_scissor(command_buffer, 0, &[scissor]);
        }
    }

    fn end_frame(&mut self) {
        let command_buffer = self.command_buffers[self.current_frame];

        self.flush_batch();

        unsafe {
            self.logical_device.cmd_end_render_pass(command_buffer);
            self.logical_device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer recording");
        }
    }

    fn present(&mut self) {
        let command_buffer = self.command_buffers[self.current_frame];
        let image_index = self.current_image_index;

        let wait_semaphores = [self.sync.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync.render_finished_semaphores[image_index as usize]];
        let command_buffers = [command_buffer];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.logical_device
                .queue_submit(
                    self.graphics_queue,
                    &[submit_info],
                    self.sync.in_flight_fences[self.current_frame],
                )
                .expect("Failed to submit draw command buffer");
        }

        let swapchains = [self.swapchain.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let present_result = unsafe {
            self.swapchain
                .loader
                .queue_present(self.present_queue, &present_info)
        };

        match present_result {
            Ok(suboptimal) => {
                if suboptimal {
                    self.needs_recreation = true;
                }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.needs_recreation = true;
            }
            Err(e) => panic!("Failed to present swapchain image: {e:?}"),
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        if self.needs_recreation {
            self.needs_recreation = false;
            self.recreate_swapchain();
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.recreate_swapchain();
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn set_camera(&mut self, camera: &Camera2D) {
        let (w, h) = (
            self.swapchain.extent.width as f32,
            self.swapchain.extent.height as f32,
        );
        self.cached_projection = camera.projection_matrix(w, h);
    }

    fn load_texture(&mut self, bytes: &[u8]) -> TextureHandle {
        let img = image::load_from_memory(bytes)
            .expect("Failed to decode texture")
            .into_rgba8();
        let (width, height) = img.dimensions();

        let image_data = texture::create_texture_image(
            &self.instance,
            &self.logical_device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            img.as_raw(),
            width,
            height,
            vk::Format::R8G8B8A8_UNORM,
        );
        let view = texture::create_image_view(
            &self.logical_device,
            image_data.image,
            vk::Format::R8G8B8A8_UNORM,
        );
        let descriptor_set = descriptor::create_descriptor_set(
            &self.logical_device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            view,
            self.sampler,
        );

        let index = self.textures.len();
        self.textures.push(texture::LoadedTexture {
            image: image_data,
            view,
            descriptor_set,
        });

        TextureHandle(index as u32)
    }

    fn create_sprite_sheet(
        &mut self,
        texture: TextureHandle,
        tile_width: u32,
        tile_height: u32,
    ) -> SpriteSheetHandle {
        let loaded = &self.textures[texture.0 as usize];
        let sheet = SpriteSheet {
            texture,
            tile_width,
            tile_height,
            texture_width: loaded.image.width,
            texture_height: loaded.image.height,
        };
        let index = self.sprite_sheets.len();
        self.sprite_sheets.push(sheet);

        SpriteSheetHandle(index as u32)
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
            self.batch.mode = target_mode;
        }
        let uv = UvRegion {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        };
        self.batch.push_sprite(transform, &uv, [r, g, b, a]);
    }

    fn load_font(&mut self, bytes: &[u8], size: f32) -> FontHandle {
        let packed = pack_atlas(bytes, size);

        let image_data = texture::create_texture_image(
            &self.instance,
            &self.logical_device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            &packed.pixels,
            packed.atlas_size,
            packed.atlas_size,
            vk::Format::R8_UNORM,
        );
        let view = create_image_view(&self.logical_device, image_data.image, vk::Format::R8_UNORM);
        let descriptor_set = create_descriptor_set(
            &self.logical_device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            view,
            self.linear_sampler,
        );

        let index = self.fonts.len();
        self.fonts.push(LoadedFont {
            image: image_data,
            view,
            descriptor_set,
            glyphs: packed.glyphs,
        });

        FontHandle(index as u32)
    }

    fn draw_text(&mut self, text: &str, font: FontHandle, position: Vec2, color: [f32; 4]) {
        let target_mode = BatchMode::Text(font);

        if self.batch.sprite_count > 0 && self.batch.mode != target_mode {
            self.flush_batch();
        }
        self.batch.mode = target_mode;

        let mut cursor_x = position.x;
        for c in text.chars() {
            let Some(glyph) = self.fonts[font.0 as usize].glyphs.get(&c).copied() else {
                continue;
            };

            if glyph.width > 0.0 && glyph.height > 0.0 {
                if self.batch.sprite_count >= MAX_SPRITES {
                    self.flush_batch();
                    self.batch.mode = target_mode;
                }

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
    }

    fn measure_text(&self, text: &str, font: FontHandle) -> Vec2 {
        match &self.fonts.get(font.0 as usize) {
            Some(atlas) => measure_text(&atlas.glyphs, text),
            None => Vec2::ZERO,
        }
    }

    fn uv_for_tile(&self, sheet: SpriteSheetHandle, index: u32) -> UvRegion {
        let sprite_sheet = self.sprite_sheets.get(sheet.0 as usize).unwrap();

        sprite_sheet.uv_for_tile(index)
    }

    fn clear_assets(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("Failed to wait for GPU idle before clearing assets");
        }

        for texture in &self.textures {
            unsafe {
                self.logical_device.destroy_image_view(texture.view, None);
            }
            texture::destroy_texture_image(&self.logical_device, &texture.image);
        }
        for font in &self.fonts {
            unsafe {
                self.logical_device.destroy_image_view(font.view, None);
            }
            texture::destroy_texture_image(&self.logical_device, &font.image);
        }

        unsafe {
            self.logical_device
                .reset_descriptor_pool(self.descriptor_pool, vk::DescriptorPoolResetFlags::empty())
                .expect("Failed to reset descriptor pool");
        }
        self.descriptor_set = descriptor::create_descriptor_set(
            &self.logical_device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            self.placeholder_view,
            self.sampler,
        );

        self.textures.clear();
        self.sprite_sheets.clear();
        self.fonts.clear();
    }

    fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }
}
