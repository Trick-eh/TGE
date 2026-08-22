use std::collections::HashMap;

use ash::vk;

use crate::{font_atlas::GlyphInfo, vulkan::buffers::create_buffer};

pub struct TextureImage {
    pub image: vk::Image,
    memory: vk::DeviceMemory,
    pub width: u32,
    pub height: u32,
}

fn run_one_shot_commands(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    record: impl FnOnce(vk::CommandBuffer),
) {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }
        .expect("Failed to allocate one-shot command buffer")[0];

    let begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe {
        device
            .begin_command_buffer(command_buffer, &begin_info)
            .expect("Failed to begin one-shot command buffer");
    }

    record(command_buffer);

    unsafe {
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end one-shot command buffer");
    }

    let command_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);

    unsafe {
        device
            .queue_submit(queue, &[submit_info], vk::Fence::null())
            .expect("Failed to submit one-shot command buffer");
        device
            .queue_wait_idle(queue)
            .expect("Failed to wait for one-shot command buffer");
        device.free_command_buffers(command_pool, &command_buffers);
    }
}

fn transition_image_layout(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    run_one_shot_commands(device, command_pool, queue, |cmd| {
        let (src_access, dst_access, src_stage, dst_stage) = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => panic!("Unsupported layout transition: {old_layout:?} -> {new_layout:?}"),
        };

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(src_access)
            .dst_access_mask(dst_access);

        unsafe {
            device.cmd_pipeline_barrier(
                cmd,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }
    });
}

fn copy_buffer_to_image(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) {
    run_one_shot_commands(device, command_pool, queue, |cmd| {
        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });

        unsafe {
            device.cmd_copy_buffer_to_image(
                cmd,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }
    });
}

pub fn create_texture_image(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    pixels: &[u8],
    width: u32,
    height: u32,
    format: vk::Format,
) -> TextureImage {
    let size = pixels.len() as vk::DeviceSize;

    let (staging_buffer, staging_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let mapped = device
            .map_memory(staging_memory, 0, size, vk::MemoryMapFlags::empty())
            .expect("Failed to map staging buffer");
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), mapped as *mut u8, pixels.len());
        device.unmap_memory(staging_memory);
    }

    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);

    let image = unsafe { device.create_image(&image_info, None) }.expect("Failed to create image");

    let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let memory_type_index = (0..mem_properties.memory_type_count)
        .find(|&i| {
            (mem_requirements.memory_type_bits & (1 << i)) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
        })
        .expect("Failed to find suitable memory type for image");

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type_index);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .expect("Failed to allocate image memory");
    unsafe { device.bind_image_memory(image, memory, 0) }.expect("Failed to bind image memory");

    transition_image_layout(
        device,
        command_pool,
        queue,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    copy_buffer_to_image(
        device,
        command_pool,
        queue,
        staging_buffer,
        image,
        width,
        height,
    );
    transition_image_layout(
        device,
        command_pool,
        queue,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_memory, None);
    }

    TextureImage {
        image,
        memory,
        width,
        height,
    }
}

pub fn destroy_texture_image(device: &ash::Device, texture: &TextureImage) {
    unsafe {
        device.destroy_image(texture.image, None);
        device.free_memory(texture.memory, None);
    }
}

pub fn create_image_view(
    device: &ash::Device,
    image: vk::Image,
    format: vk::Format,
) -> vk::ImageView {
    let create_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .components(vk::ComponentMapping::default())
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });

    unsafe { device.create_image_view(&create_info, None) }
        .expect("Failed to create texture image view")
}

pub fn create_sampler(device: &ash::Device) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(false)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST);

    unsafe { device.create_sampler(&sampler_info, None) }.expect("Failed to create sampler")
}

pub fn create_linear_sampler(device: &ash::Device) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(false)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

    unsafe { device.create_sampler(&sampler_info, None) }.expect("Failed to create sampler")
}

pub struct LoadedTexture {
    pub image: TextureImage,
    pub view: vk::ImageView,
    pub descriptor_set: vk::DescriptorSet,
}

pub struct LoadedFont {
    pub image: TextureImage,
    pub view: vk::ImageView,
    pub descriptor_set: vk::DescriptorSet,
    pub glyphs: HashMap<char, GlyphInfo>,
}
