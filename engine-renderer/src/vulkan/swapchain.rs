use ash::vk;
use winit::window::Window;

use crate::vulkan::device::{self, QueueFamilyIndices};

pub struct SwapchainData {
    pub loader: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

fn choose_surface_format(available: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    available
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .copied()
        .unwrap_or(available[0])
}

fn choose_present_mode(available: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    if available.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::FIFO
    }
}

fn choose_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window: &Window) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }

    let size = window.inner_size();
    vk::Extent2D {
        width: size.width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: size.height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}

fn create_image_views(
    device: &ash::Device,
    images: &[vk::Image],
    format: vk::Format,
) -> Vec<vk::ImageView> {
    images
        .iter()
        .map(|&image| {
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
                .expect("Failed to create swapchain image view")
        })
        .collect()
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    queue_families: QueueFamilyIndices,
    window: &Window,
) -> SwapchainData {
    let support = device::query_swapchain_support(surface_loader, physical_device, surface);

    let surface_format = choose_surface_format(&support.formats);
    let present_mode = choose_present_mode(&support.present_modes);
    let extent = choose_swap_extent(&support.capabilities, window);

    let mut image_count = support.capabilities.min_image_count + 1;
    if support.capabilities.max_image_count > 0 {
        image_count = image_count.min(support.capabilities.max_image_count);
    }

    let family_indices = [
        queue_families.graphics_family,
        queue_families.present_family,
    ];

    let mut create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    create_info = if queue_families.is_single_queue() {
        create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    } else {
        create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&family_indices)
    };

    let loader = ash::khr::swapchain::Device::new(instance, device);
    let swapchain =
        unsafe { loader.create_swapchain(&create_info, None) }.expect("Failed to create swapchain");

    let images =
        unsafe { loader.get_swapchain_images(swapchain) }.expect("Failed to get swapchain images");

    let image_views = create_image_views(device, &images, surface_format.format);

    println!(
        "Swapchain created: {} images, format {:?}, present mode {:?}, extent {}x{}",
        images.len(),
        surface_format.format,
        present_mode,
        extent.width,
        extent.height
    );

    SwapchainData {
        loader,
        swapchain,
        images,
        image_views,
        format: surface_format.format,
        extent,
    }
}
