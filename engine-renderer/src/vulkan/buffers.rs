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

fn create_mapped_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
) -> (vk::Buffer, vk::DeviceMemory, *mut c_void) {
    let (buffer, memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        usage,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let mapped = unsafe { device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty()) }
        .expect("Failed to map buffer memory");

    (buffer, memory, mapped)
}

pub struct RetiredBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

pub struct FrameBatchBuffers {
    pub vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,
    pub vertex_mapped: *mut c_void,
    pub vertex_buffer_capacity: usize,

    pub index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,
    pub index_mapped: *mut c_void,
    pub index_buffer_capacity: usize,

    pub pending_retirement: Vec<RetiredBuffer>,
}

pub fn create_frame_batch_buffers(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    count: usize,
) -> Vec<FrameBatchBuffers> {
    let initial_vertex_size =
        (MAX_SPRITES * FLOATS_PER_SPRITE * size_of::<f32>()) as vk::DeviceSize;
    let initial_index_size =
        (MAX_SPRITES * INDICES_PER_SPRITE * size_of::<u32>()) as vk::DeviceSize;

    (0..count)
        .map(|_| {
            let (vertex_buffer, vertex_memory, vertex_mapped) = create_mapped_buffer(
                instance,
                device,
                physical_device,
                initial_vertex_size,
                vk::BufferUsageFlags::VERTEX_BUFFER,
            );

            let (index_buffer, index_memory, index_mapped) = create_mapped_buffer(
                instance,
                device,
                physical_device,
                initial_index_size,
                vk::BufferUsageFlags::INDEX_BUFFER,
            );

            FrameBatchBuffers {
                vertex_buffer,
                vertex_memory,
                vertex_mapped,
                vertex_buffer_capacity: initial_vertex_size as usize,
                index_buffer,
                index_memory,
                index_mapped,
                index_buffer_capacity: initial_index_size as usize,
                pending_retirement: Vec::new(),
            }
        })
        .collect()
}

pub fn reallocate_frame_batch_buffers(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    fb: &mut FrameBatchBuffers,
    new_vertex_capacity: usize,
    new_index_capacity: usize,
) {
    let (new_vertex_buffer, new_vertex_memory, new_vertex_mapped) = create_mapped_buffer(
        instance,
        device,
        physical_device,
        new_vertex_capacity as vk::DeviceSize,
        vk::BufferUsageFlags::VERTEX_BUFFER,
    );
    let (new_index_buffer, new_index_memory, new_index_mapped) = create_mapped_buffer(
        instance,
        device,
        physical_device,
        new_index_capacity as vk::DeviceSize,
        vk::BufferUsageFlags::INDEX_BUFFER,
    );

    fb.pending_retirement.push(RetiredBuffer {
        buffer: fb.vertex_buffer,
        memory: fb.vertex_memory,
    });
    fb.pending_retirement.push(RetiredBuffer {
        buffer: fb.index_buffer,
        memory: fb.index_memory,
    });

    fb.vertex_buffer = new_vertex_buffer;
    fb.vertex_memory = new_vertex_memory;
    fb.vertex_mapped = new_vertex_mapped;
    fb.vertex_buffer_capacity = new_vertex_capacity;

    fb.index_buffer = new_index_buffer;
    fb.index_memory = new_index_memory;
    fb.index_mapped = new_index_mapped;
    fb.index_buffer_capacity = new_index_capacity;

    println!(
        "Batch buffer grown: vertex {} bytes, index {} bytes",
        new_vertex_capacity, new_index_capacity
    );
}

pub fn release_retired_buffers(device: &ash::Device, fb: &mut FrameBatchBuffers) {
    for retired in fb.pending_retirement.drain(..) {
        unsafe {
            device.unmap_memory(retired.memory);
            device.destroy_buffer(retired.buffer, None);
            device.free_memory(retired.memory, None);
        }
    }
}

pub fn destroy_frame_batch_buffers(device: &ash::Device, buffers: &[FrameBatchBuffers]) {
    for fb in buffers {
        unsafe {
            for retired in &fb.pending_retirement {
                device.unmap_memory(retired.memory);
                device.destroy_buffer(retired.buffer, None);
                device.free_memory(retired.memory, None);
            }

            device.unmap_memory(fb.vertex_memory);
            device.destroy_buffer(fb.vertex_buffer, None);
            device.free_memory(fb.vertex_memory, None);

            device.unmap_memory(fb.index_memory);
            device.destroy_buffer(fb.index_buffer, None);
            device.free_memory(fb.index_memory, None);
        }
    }
}
