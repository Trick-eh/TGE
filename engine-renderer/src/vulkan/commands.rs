use ash::vk;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub fn create_command_pool(device: &ash::Device, graphics_family: u32) -> vk::CommandPool {
    let pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(graphics_family);

    unsafe { device.create_command_pool(&pool_info, None) }.expect("Failed to create command pool")
}

pub fn allocate_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
) -> Vec<vk::CommandBuffer> {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

    unsafe { device.allocate_command_buffers(&alloc_info) }
        .expect("Failed to allocate command buffers")
}
