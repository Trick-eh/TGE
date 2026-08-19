use std::ffi::c_void;

use ash::vk;

use crate::batch::{FLOATS_PER_SPRITE, INDICES_PER_SPRITE, MAX_SPRITES};

fn find_memory_type(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    for i in 0..mem_properties.memory_type_count {
        let type_matches = (type_filter & (1 << i)) != 0;
        let properties_match = mem_properties.memory_types[i as usize]
            .property_flags
            .contains(properties);

        if type_matches && properties_match {
            return i;
        }
    }

    panic!("Failed to find a suitable memory type for the requested buffer");
}

pub fn create_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer =
        unsafe { device.create_buffer(&buffer_info, None) }.expect("Failed to create buffer");

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type_index = find_memory_type(
        instance,
        physical_device,
        mem_requirements.memory_type_bits,
        properties,
    );

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type_index);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .expect("Failed to allocate buffer memory");

    unsafe { device.bind_buffer_memory(buffer, memory, 0) }.expect("Failed to bind buffer memory");

    (buffer, memory)
}

pub struct FrameBatchBuffers {
    pub vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,
    pub vertex_mapped: *mut c_void,
    pub index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,
    pub index_mapped: *mut c_void,
}

pub fn create_frame_batch_buffers(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    count: usize,
) -> Vec<FrameBatchBuffers> {
    let vertex_size = (MAX_SPRITES * FLOATS_PER_SPRITE * size_of::<f32>()) as vk::DeviceSize;
    let index_size = (MAX_SPRITES * INDICES_PER_SPRITE * size_of::<u32>()) as vk::DeviceSize;

    (0..count)
        .map(|_| {
            let (vertex_buffer, vertex_memory) = create_buffer(
                instance,
                device,
                physical_device,
                vertex_size,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            let vertex_mapped = unsafe {
                device.map_memory(vertex_memory, 0, vertex_size, vk::MemoryMapFlags::empty())
            }
            .expect("Failed to map vertex buffer memory");

            let (index_buffer, index_memory) = create_buffer(
                instance,
                device,
                physical_device,
                index_size,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            let index_mapped = unsafe {
                device.map_memory(index_memory, 0, index_size, vk::MemoryMapFlags::empty())
            }
            .expect("Failed to map index buffer memory");

            FrameBatchBuffers {
                vertex_buffer,
                vertex_memory,
                vertex_mapped,
                index_buffer,
                index_memory,
                index_mapped,
            }
        })
        .collect()
}

pub fn destroy_frame_batch_buffers(device: &ash::Device, buffers: &[FrameBatchBuffers]) {
    for fb in buffers {
        unsafe {
            device.unmap_memory(fb.vertex_memory);
            device.destroy_buffer(fb.vertex_buffer, None);
            device.free_memory(fb.vertex_memory, None);

            device.unmap_memory(fb.index_memory);
            device.destroy_buffer(fb.index_buffer, None);
            device.free_memory(fb.index_memory, None);
        }
    }
}
