use ash::vk::{self, DescriptorPool};

pub fn create_descriptor_pool(device: &ash::Device, max_sets: u32) -> vk::DescriptorPool {
    let pool_size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(max_sets);

    let pool_sizes = [pool_size];
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&pool_sizes)
        .max_sets(max_sets);

    unsafe { device.create_descriptor_pool(&pool_info, None) }
        .expect("Failed to creeate descriptor pool")
}

pub fn create_descriptor_set(
    device: &ash::Device,
    pool: DescriptorPool,
    layout: vk::DescriptorSetLayout,
    image_view: vk::ImageView,
    sampler: vk::Sampler,
) -> vk::DescriptorSet {
    let layouts = [layout];
    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);

    let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info) }
        .expect("Failed to allocate descriptor set")[0];

    let image_info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(image_view)
        .sampler(sampler);

    let image_infos = [image_info];
    let write = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&image_infos);

    unsafe { device.update_descriptor_sets(&[write], &[]) };

    descriptor_set
}
