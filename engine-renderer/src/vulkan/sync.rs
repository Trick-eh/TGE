use ash::vk;

use crate::vulkan::commands::MAX_FRAMES_IN_FLIGHT;

pub struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
}

pub fn create_sync_objects(device: &ash::Device, swapchain_image_count: usize) -> SyncObjects {
    let semaphore_info = vk::SemaphoreCreateInfo::default();
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

    let image_available_semaphores = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| {
            unsafe { device.create_semaphore(&semaphore_info, None) }
                .expect("Failed to create image-available semaphore")
        })
        .collect();

    let render_finished_semaphores = (0..swapchain_image_count)
        .map(|_| {
            unsafe { device.create_semaphore(&semaphore_info, None) }
                .expect("Failed to create render-finished semaphore")
        })
        .collect();

    let in_flight_fences = (00..MAX_FRAMES_IN_FLIGHT)
        .map(|_| unsafe { device.create_fence(&fence_info, None) }.expect("Failed to create fence"))
        .collect();

    SyncObjects {
        image_available_semaphores,
        render_finished_semaphores,
        in_flight_fences,
    }
}
