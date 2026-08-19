pub mod buffers;
pub mod commands;
pub mod descriptor;
pub mod device;
pub mod framebuffers;
pub mod pipeline;
pub mod render_pass;
pub mod renderer_impl;
pub mod swapchain;
pub mod sync;
pub mod texture;
use commands::MAX_FRAMES_IN_FLIGHT;
use device::QueueFamilyIndices;
use engine_math::{Mat4, Transform2D};
use swapchain::SwapchainData;
use sync::SyncObjects;

use std::collections::BTreeSet;
use std::ffi::CStr;

use ash::vk::{self, Queue};
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

use crate::UvRegion;
use crate::batch::{BatchMode, INDICES_PER_SPRITE, MAX_SPRITES, SpriteBatch};
use crate::colors::BLACK;

#[cfg(debug_assertions)]
const ENABLE_VALIDATION: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION: bool = false;

const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

pub struct VulkanRenderer {
    entry: ash::Entry,
    pub instance: ash::Instance,
    surface_loader: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub queue_families: QueueFamilyIndices,
    pub graphics_queue: Queue,
    pub present_queue: Queue,
    pub swapchain: SwapchainData,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub sync: SyncObjects,
    pub batch_buffers: Vec<buffers::FrameBatchBuffers>,
    pub placeholder_texture: texture::TextureImage,
    pub placeholder_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub current_frame: usize,
    pub current_image_index: u32,
    pub needs_recreation: bool,
    window: Window,

    batch: SpriteBatch,
    segments: Vec<BatchSegment>,
    cached_projection: Mat4,
    clear_color: [f32; 4],
}
pub struct BatchSegment {
    pub mode: BatchMode,
    pub index_start: u32,
    pub index_count: u32,
}

impl VulkanRenderer {
    pub fn new(event_loop: &ActiveEventLoop, title: &str) -> VulkanRenderer {
        let entry = unsafe { ash::Entry::load() }
            .expect("Failed to load Vulkan entry point -- is a Vulkan driver/loader installed");

        let app_name = c"engine-core";
        let engine_name = c"engine-core";

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let display_handle = event_loop
            .display_handle()
            .expect("Failed to get display handle from event loop")
            .as_raw();

        let mut required_extensions = ash_window::enumerate_required_extensions(display_handle)
            .expect("Failed to enumerate required surface extensions")
            .to_vec();

        let mut layers: Vec<*const i8> = Vec::new();

        if ENABLE_VALIDATION {
            let available_layers = unsafe { entry.enumerate_instance_layer_properties() }
                .expect("Failed to enumerate instance layers");

            let has_validation = available_layers.iter().any(|layer| {
                let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                name == VALIDATION_LAYER
            });

            if has_validation {
                layers.push(VALIDATION_LAYER.as_ptr());
                required_extensions.push(ash::ext::debug_utils::NAME.as_ptr());
            } else {
                eprintln!(
                    "Vulkan validation layer requested but not available on this system \
                    (install the Vulkan SDK to get it) -- continuing without it."
                );
            }
        }

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&required_extensions)
            .enabled_layer_names(&layers);

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .expect("Failed to create Vulkan instance");

        let window = event_loop
            .create_window(Window::default_attributes().with_title(title))
            .expect("Failed to create window");

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        let display_handle = window
            .display_handle()
            .expect("Failed to get display handle from window")
            .as_raw();

        let window_handle = window
            .window_handle()
            .expect("Failed to get window handle")
            .as_raw();

        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
        }
        .expect("Failed to create Vulkan surface");

        let (physical_device, queue_families) =
            device::pick_physical_device(&instance, &surface_loader, surface);

        let unique_families: BTreeSet<u32> = [
            queue_families.graphics_family,
            queue_families.present_family,
        ]
        .into_iter()
        .collect();

        let queue_priorities = [1.0f32];
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_families
            .iter()
            .map(|&family| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(family)
                    .queue_priorities(&queue_priorities)
            })
            .collect();

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&device_features);

        let logical_device =
            unsafe { instance.create_device(physical_device, &device_create_info, None) }
                .expect("Failed to create logical device");

        let graphics_queue =
            unsafe { logical_device.get_device_queue(queue_families.graphics_family, 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(queue_families.present_family, 0) };

        let swapchain = swapchain::create_swapchain(
            &instance,
            &logical_device,
            &surface_loader,
            surface,
            physical_device,
            queue_families,
            &window,
        );

        let render_pass = render_pass::create_render_pass(&logical_device, swapchain.format);
        println!("Render pass created");

        let framebuffers = framebuffers::create_framebuffers(
            &logical_device,
            render_pass,
            &swapchain.image_views,
            swapchain.extent,
        );
        println!("Framebuffers created: {}", framebuffers.len());

        let descriptor_set_layout = pipeline::create_descriptor_set_layout(&logical_device);
        let pipeline_layout =
            pipeline::create_pipeline_layout(&logical_device, descriptor_set_layout);
        let pipeline =
            pipeline::create_graphics_pipeline(&logical_device, render_pass, pipeline_layout);
        println!("Pipeline created");

        let command_pool =
            commands::create_command_pool(&logical_device, queue_families.graphics_family);
        let command_buffers = commands::allocate_command_buffers(&logical_device, command_pool);
        println!(
            "Command pool created, {} command buffers allocated",
            command_buffers.len()
        );
        let sync = sync::create_sync_objects(&logical_device, swapchain.images.len());
        println!(
            "Sync objects created: {} image-available, {} render-finished, {} fences",
            sync.image_available_semaphores.len(),
            sync.render_finished_semaphores.len(),
            sync.in_flight_fences.len(),
        );

        let batch_buffers = buffers::create_frame_batch_buffers(
            &instance,
            &logical_device,
            physical_device,
            MAX_FRAMES_IN_FLIGHT,
        );
        println!("Batch buffers created: {} sets", batch_buffers.len());

        let placeholder_pixels: [u8; 4] = [255, 255, 255, 255];
        let placeholder_texture = texture::create_texture_image(
            &instance,
            &logical_device,
            physical_device,
            command_pool,
            graphics_queue,
            &placeholder_pixels,
            1,
            1,
        );
        let placeholder_view = texture::create_image_view(
            &logical_device,
            placeholder_texture.image,
            vk::Format::R8G8B8A8_SRGB,
        );
        let sampler = texture::create_sampler(&logical_device);

        let descriptor_pool = descriptor::create_descriptor_pool(&logical_device, 16);
        let descriptor_set = descriptor::create_descriptor_set(
            &logical_device,
            descriptor_pool,
            descriptor_set_layout,
            placeholder_view,
            sampler,
        );
        println!("Placerholder texture + descriptor set created");

        VulkanRenderer {
            entry,
            instance,
            surface_loader,
            surface,
            physical_device,
            logical_device,
            queue_families,
            graphics_queue,
            present_queue,
            swapchain,
            render_pass,
            framebuffers,
            descriptor_set_layout,
            pipeline_layout,
            pipeline,
            command_pool,
            command_buffers,
            sync,
            batch_buffers,
            placeholder_texture,
            placeholder_view,
            sampler,
            descriptor_pool,
            descriptor_set,
            current_frame: 0,
            current_image_index: 0,
            needs_recreation: false,
            window,
            batch: SpriteBatch::new(),
            segments: Vec::new(),
            cached_projection: Mat4::IDENTITY,
            clear_color: BLACK,
        }
    }

    fn recreate_swapchain(&mut self) {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("Failed to wait for GPU idle before swapchain recreation");
        }

        unsafe {
            for &framebuffer in &self.framebuffers {
                self.logical_device.destroy_framebuffer(framebuffer, None);
            }
            for &view in &self.swapchain.image_views {
                self.logical_device.destroy_image_view(view, None);
            }
            self.swapchain
                .loader
                .destroy_swapchain(self.swapchain.swapchain, None);
        }

        self.swapchain = swapchain::create_swapchain(
            &self.instance,
            &self.logical_device,
            &self.surface_loader,
            self.surface,
            self.physical_device,
            self.queue_families,
            &self.window,
        );

        self.framebuffers = framebuffers::create_framebuffers(
            &self.logical_device,
            self.render_pass,
            &self.swapchain.image_views,
            self.swapchain.extent,
        );

        println!(
            "Swapchain recreated: {} images, extent {}x{}",
            self.swapchain.images.len(),
            self.swapchain.extent.width,
            self.swapchain.extent.height,
        );
    }

    fn push_into_batch(
        &mut self,
        mode: BatchMode,
        transform: &Transform2D,
        uv: &UvRegion,
        color: [f32; 4],
    ) {
        assert!(
            self.batch.sprite_count < crate::batch::MAX_SPRITES,
            "Exceeded MAX_SPRITES ({}) sprites in a single frame. \
            dynamic buffer growth / multi-submission-per-frame is not implemented yet",
            MAX_SPRITES
        );

        let needs_new_segment = match self.segments.last() {
            Some(seg) => seg.mode != mode,
            None => true,
        };
        if needs_new_segment {
            self.segments.push(BatchSegment {
                mode,
                index_start: self.batch.indices.len() as u32,
                index_count: 0,
            });
        }

        self.batch.push_sprite(transform, uv, color);

        if let Some(seg) = self.segments.last_mut() {
            seg.index_count += INDICES_PER_SPRITE as u32;
        }
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("Failed to wait for GPU idle before teardown -- device may have crashed/reset, or the system is out of memory");

            buffers::destroy_frame_batch_buffers(&self.logical_device, &self.batch_buffers);

            self.logical_device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.logical_device.destroy_sampler(self.sampler, None);
            self.logical_device
                .destroy_image_view(self.placeholder_view, None);
            texture::destroy_texture_image(&self.logical_device, &self.placeholder_texture);

            for &img_semaphore in &self.sync.image_available_semaphores {
                self.logical_device.destroy_semaphore(img_semaphore, None);
            }
            for &render_semaphore in &self.sync.render_finished_semaphores {
                self.logical_device
                    .destroy_semaphore(render_semaphore, None);
            }
            for &fence in &self.sync.in_flight_fences {
                self.logical_device.destroy_fence(fence, None);
            }

            self.logical_device
                .destroy_command_pool(self.command_pool, None);

            self.logical_device.destroy_pipeline(self.pipeline, None);
            self.logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.logical_device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            for &framebuffer in &self.framebuffers {
                self.logical_device.destroy_framebuffer(framebuffer, None);
            }
            self.logical_device
                .destroy_render_pass(self.render_pass, None);
            for &view in &self.swapchain.image_views {
                self.logical_device.destroy_image_view(view, None);
            }
            self.swapchain
                .loader
                .destroy_swapchain(self.swapchain.swapchain, None);
            self.logical_device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
