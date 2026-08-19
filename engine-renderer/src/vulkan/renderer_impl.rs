use ash::vk;
use engine_math::{Camera2D, Transform2D, Vec2};

use crate::batch::BatchMode;
use crate::vulkan::VulkanRenderer;
use crate::vulkan::commands::MAX_FRAMES_IN_FLIGHT;
use crate::{FontHandle, Renderer, SpriteSheetHandle, TextureHandle, UvRegion};

impl Renderer for VulkanRenderer {
    fn begin_frame(&mut self) {
        let fence = self.sync.in_flight_fences[self.current_frame];

        unsafe {
            self.logical_device
                .wait_for_fences(&[fence], true, u64::MAX)
                .expect("Failed to wait for in-flight fence");
        }

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

        if self.batch.sprite_count > 0 {
            let fb = &self.batch_buffers[self.current_frame];

            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.batch.vertices.as_ptr(),
                    fb.vertex_mapped as *mut f32,
                    self.batch.vertices.len(),
                );
                std::ptr::copy_nonoverlapping(
                    self.batch.indices.as_ptr(),
                    fb.index_mapped as *mut u32,
                    self.batch.indices.len(),
                );
            }

            unsafe {
                self.logical_device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &[fb.vertex_buffer],
                    &[0],
                );
                self.logical_device.cmd_bind_index_buffer(
                    command_buffer,
                    fb.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );
            }

            let projection_cols = self.cached_projection.to_cols_array();

            for segment in &self.segments {
                let use_texture = match segment.mode {
                    BatchMode::Untextured => 0.0f32,
                    BatchMode::Textured(_) => 1.0f32,
                    BatchMode::Empty => unreachable!("segments are never created with Empty mode"),
                };

                let mut push_data = [0u8; 72];
                for (i, f) in projection_cols.iter().enumerate() {
                    push_data[i * 4..i * 4 + 4].copy_from_slice(&f.to_ne_bytes());
                }
                push_data[64..68].copy_from_slice(&use_texture.to_ne_bytes());
                push_data[68..72].copy_from_slice(&0.0f32.to_ne_bytes()); // text rendering not
                // wired yet

                unsafe {
                    let descriptor_sets = [self.descriptor_set];
                    self.logical_device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout,
                        0,
                        &descriptor_sets,
                        &[],
                    );

                    self.logical_device.cmd_push_constants(
                        command_buffer,
                        self.pipeline_layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        &push_data,
                    );

                    self.logical_device.cmd_draw_indexed(
                        command_buffer,
                        segment.index_count,
                        1,
                        segment.index_start,
                        0,
                        0,
                    );
                }
            }

            self.batch.clear();
            self.segments.clear();
        }

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
        unimplemented!("Vulkan texture upload not implemented yet");
    }

    fn create_sprite_sheet(
        &mut self,
        texture: TextureHandle,
        tile_width: u32,
        tile_height: u32,
    ) -> SpriteSheetHandle {
        unimplemented!("Vulkan sprite sheets not implemented yet");
    }

    fn draw_sprite(
        &mut self,
        transform: &Transform2D,
        spritesheet: SpriteSheetHandle,
        tile_index: u32,
    ) {
        unimplemented!("Vulkan sprite drawing not implemented yet -- no pipeline/batch exists yet");
    }

    fn draw_colored_rect(&mut self, transform: &Transform2D, r: f32, g: f32, b: f32, a: f32) {
        let uv = UvRegion {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        };
        self.push_into_batch(BatchMode::Untextured, transform, &uv, [r, g, b, a]);
    }

    fn load_font(&mut self, bytes: &[u8], size: f32) -> FontHandle {
        unimplemented!("Vulkan font loading not implemented yet");
    }

    fn draw_text(&mut self, text: &str, font: FontHandle, position: Vec2, color: [f32; 4]) {
        unimplemented!("Vulkan text drawing not implemented yet");
    }

    fn measure_text(&self, text: &str, font: FontHandle) -> Vec2 {
        unimplemented!("Vulkan text measurement not implemented yet");
    }

    fn uv_for_tile(&self, sheet: SpriteSheetHandle, index: u32) -> UvRegion {
        unimplemented!("Vulkan UV lookup not implemented yet");
    }

    fn clear_assets(&mut self) {
        unimplemented!("Vulkan asset clearing not implemented yet");
    }

    fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }
}
