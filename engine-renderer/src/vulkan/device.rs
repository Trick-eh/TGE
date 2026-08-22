use std::ffi::CStr;

use ash::vk;

use crate::vulkan::ENABLE_VALIDATION;

#[derive(Clone, Copy, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: u32,
    pub present_family: u32,
}

impl QueueFamilyIndices {
    pub fn is_single_queue(&self) -> bool {
        self.graphics_family == self.present_family
    }
}

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    fn is_adequate(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }
}

pub fn query_swapchain_support(
    surface_loader: &ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> SwapchainSupportDetails {
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
    }
    .expect("Failed to query surface capabilities");

    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface) }
            .expect("Failed to query surface formats");

    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
    }
    .expect("Failed to query surface present modes");

    SwapchainSupportDetails {
        capabilities,
        formats,
        present_modes,
    }
}

fn find_queue_families(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Option<QueueFamilyIndices> {
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut graphics_family = None;
    let mut present_family = None;

    for (index, family) in queue_families.iter().enumerate() {
        let index = index as u32;

        if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics_family.is_none() {
            graphics_family = Some(index);
        }

        let supports_present = unsafe {
            surface_loader.get_physical_device_surface_support(physical_device, index, surface)
        }
        .unwrap_or(false);

        if supports_present && present_family.is_none() {
            present_family = Some(index)
        }

        if graphics_family.is_some() && present_family.is_some() {
            break;
        }
    }

    Some(QueueFamilyIndices {
        graphics_family: graphics_family?,
        present_family: present_family?,
    })
}

fn supports_required_device_extensions(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> bool {
    let available = unsafe { instance.enumerate_device_extension_properties(physical_device) }
        .unwrap_or_default();

    available.iter().any(
        |ext| unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) } == ash::khr::swapchain::NAME,
    )
}

fn is_device_suitable(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Option<QueueFamilyIndices> {
    let indices = find_queue_families(instance, surface_loader, physical_device, surface)?;

    if !supports_required_device_extensions(instance, physical_device) {
        return None;
    }

    let swapchain_support = query_swapchain_support(surface_loader, physical_device, surface);
    if !swapchain_support.is_adequate() {
        return None;
    }

    Some(indices)
}

fn score_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    indices: &QueueFamilyIndices,
) -> i64 {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };

    let mut score: i64 = 0;

    score += match properties.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU => 10_000,
        vk::PhysicalDeviceType::INTEGRATED_GPU => 1_000,
        vk::PhysicalDeviceType::VIRTUAL_GPU => 100,
        vk::PhysicalDeviceType::CPU => 10,
        _ => 0,
    };

    score += properties.limits.max_image_dimension2_d as i64;

    if indices.is_single_queue() {
        score += 500;
    }

    score
}

pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, QueueFamilyIndices) {
    let devices = unsafe { instance.enumerate_physical_devices() }
        .expect("Failed to enumerate physical devices");

    let mut best: Option<(vk::PhysicalDevice, QueueFamilyIndices, i64)> = None;

    for device in devices {
        let Some(indices) = is_device_suitable(instance, surface_loader, device, surface) else {
            continue;
        };

        let score = score_device(instance, device, &indices);

        let properties = unsafe { instance.get_physical_device_properties(device) };
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
        if ENABLE_VALIDATION {
            println!(
                "Vulkan candidate device: {:?} (type: {:?}, score: {})",
                name, properties.device_type, score
            );
        }

        if best.is_none_or(|(_, _, best_score)| score > best_score) {
            best = Some((device, indices, score));
        }
    }

    let (device, indices, _) =
        best.expect("No suitable Vulkan physical device found on this system");

    let properties = unsafe { instance.get_physical_device_properties(device) };
    let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
    if ENABLE_VALIDATION {
        println!(
            "Selected Vulkan device: {:?} (graphics family: {}, present family: {}, single queue: {})",
            name,
            indices.graphics_family,
            indices.present_family,
            indices.is_single_queue()
        );
    }

    (device, indices)
}
