use ash::{
    ext::debug_utils,
    khr::{surface, swapchain, win32_surface},
    vk::{
        self, ColorSpaceKHR, ComponentMapping, CompositeAlphaFlagsKHR, DebugUtilsMessengerEXT,
        DeviceQueueCreateInfo, Format, Image, ImageAspectFlags, ImageSubresourceRange,
        ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, PFN_vkCreateImageView,
        PhysicalDevice, PresentModeKHR, Queue, QueueFlags, SharingMode, SurfaceKHR,
        SwapchainCreateInfoKHR, SwapchainKHR,
    },
    Device, Entry, Instance,
};
use std::{
    borrow::Cow,
    collections::HashSet,
    ffi::{self, c_char},
};

use crate::Handle;
use crate::{error::Result, Error};

pub struct Vulkan {}

impl Vulkan {
    pub fn init(handle: Handle) -> Result<Self> {
        let layer_names = vec![b"VK_LAYER_KHRONOS_validation\0"]
            .into_iter()
            .map(|raw_name| unsafe { ffi::CStr::from_bytes_with_nul_unchecked(raw_name).as_ptr() })
            .collect();

        let extension_names = vec![
            b"VK_EXT_debug_utils\0  ",
            b"VK_KHR_win32_surface\0",
            b"VK_KHR_surface\0      ",
        ]
        .into_iter()
        .map(|raw_name| unsafe { ffi::CStr::from_bytes_with_nul_unchecked(raw_name).as_ptr() })
        .collect();

        let device_extension_names = vec![b"VK_KHR_swapchain\0"]
            .into_iter()
            .map(|raw_name| unsafe { ffi::CStr::from_bytes_with_nul_unchecked(raw_name).as_ptr() })
            .collect();

        let entry = Entry::linked();
        log::trace!("vulkan entry created");

        let instance = Self::create_instance(&entry, layer_names, extension_names).unwrap();
        log::trace!("vulkan instance created");

        let debug_utils = Self::create_debug_utils_messenger(&entry, &instance).unwrap();
        log::trace!("vulkan debug utils messenger created");

        let surface = Self::create_surface(&handle, &entry, &instance).unwrap();
        log::trace!("vulkan surface created");

        let (
            device,
            physical_device,
            (queue_graphic, queue_graphic_index),
            (queue_present, queue_present_index),
        ) = Self::create_device(&entry, &instance, device_extension_names, &surface).unwrap();
        log::trace!("vulkan device created");

        let (swapchain, images) = Self::create_swapchain(
            &entry,
            &instance,
            &surface,
            &physical_device,
            &device,
            (queue_graphic_index, queue_present_index),
        )
        .unwrap();
        log::trace!("vulkan swapchain created");

        return Ok(Vulkan {});
    }

    fn create_instance(
        entry: &Entry,
        layer_names: Vec<*const c_char>,
        extension_names: Vec<*const c_char>,
    ) -> Result<Instance> {
        let app_name = ffi::CString::new("Alovan App").unwrap();
        let eng_name = ffi::CString::new("Alovan Eng").unwrap();

        let appinfo = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&eng_name)
            .engine_version(vk::make_version(0, 0, 3))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        return match unsafe { entry.create_instance(&create_info, None) } {
            Ok(value) => Ok(value),
            Err(error) => Err(crate::Error::Vulkan(error)),
        };
    }

    fn create_surface(handle: &Handle, entry: &Entry, instance: &Instance) -> Result<SurfaceKHR> {
        match handle {
            Handle::Win32(h) => {
                let win32_surface_create_info = vk::Win32SurfaceCreateInfoKHR::default().hwnd(*h);
                let win32_surface_loader = win32_surface::Instance::new(&entry, &instance);

                return match unsafe {
                    win32_surface_loader.create_win32_surface(&win32_surface_create_info, None)
                } {
                    Ok(value) => Ok(value),
                    Err(error) => Err(crate::Error::Vulkan(error)),
                };
            }
            Handle::Custom(h) => {
                let win32_surface_create_info =
                    vk::Win32SurfaceCreateInfoKHR::default().hwnd(*h as isize);
                let win32_surface_loader = win32_surface::Instance::new(&entry, &instance);

                return match unsafe {
                    win32_surface_loader.create_win32_surface(&win32_surface_create_info, None)
                } {
                    Ok(value) => Ok(value),
                    Err(error) => Err(Error::Vulkan(error)),
                };
            }
            _ => {
                unimplemented!();
            }
        }
    }

    fn create_device(
        entry: &Entry,
        instance: &Instance,
        extension: Vec<*const i8>,
        surface: &SurfaceKHR,
    ) -> Result<(Device, PhysicalDevice, (Queue, u32), (Queue, u32))> {
        let instance_surface = surface::Instance::new(entry, instance);

        let physical_devices = unsafe { instance.enumerate_physical_devices() }.unwrap();

        for physical_device in physical_devices {
            let mut queue_family_id_present = None;
            let mut queue_family_id_graphic = None;

            let queue_family_properties =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

            'q: for (index, queue_family_property) in queue_family_properties.iter().enumerate() {
                if queue_family_property
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
                {
                    queue_family_id_graphic = Some(index as u32);
                    break 'q;
                }
            }
            'q: for (index, _queue_family_property) in queue_family_properties.iter().enumerate() {
                if unsafe {
                    instance_surface
                        .get_physical_device_surface_support(
                            physical_device,
                            index as u32,
                            *surface,
                        )
                        .unwrap()
                } {
                    queue_family_id_present = Some(index as u32);
                    break 'q;
                }
            }

            let (Some(queue_family_id_graphic), Some(queue_family_id_present)) =
                (queue_family_id_graphic, queue_family_id_present)
            else {
                return Err(Error::Other("Queue dont found".to_owned()));
            };

            let mut queue_family_ids = HashSet::new();
            queue_family_ids.insert(queue_family_id_graphic);
            queue_family_ids.insert(queue_family_id_present);

            let queue_create_infos: Vec<DeviceQueueCreateInfo> = queue_family_ids
                .iter()
                .map(|id| {
                    let mut queue_create_info = DeviceQueueCreateInfo::default()
                        .queue_family_index(*id)
                        .queue_priorities(&[1.0f32; 1]);
                    queue_create_info.queue_count = 1;
                    queue_create_info
                })
                .collect();

            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&extension);

            let device =
                match unsafe { instance.create_device(physical_device, &device_create_info, None) }
                {
                    Ok(value) => value,
                    Err(error) => return Err(Error::Vulkan(error)),
                };

            let queue_graphic = unsafe { device.get_device_queue(queue_family_id_graphic, 0) };
            let queue_present = unsafe { device.get_device_queue(queue_family_id_present, 0) };

            return Ok((
                device,
                physical_device,
                (queue_graphic, queue_family_id_graphic),
                (queue_present, queue_family_id_present),
            ));
        }

        Err(Error::Other("Device dont found".to_owned()))
    }

    fn create_swapchain(
        entry: &Entry,
        instance: &Instance,
        surface: &SurfaceKHR,
        physical_device: &PhysicalDevice,
        device: &Device,
        (queue_graphic_index, queue_present_index): (u32, u32),
    ) -> Result<(SwapchainKHR, Vec<ImageView>)> {
        let instance_surface = surface::Instance::new(entry, instance);

        let surface_capability = unsafe {
            instance_surface.get_physical_device_surface_capabilities(*physical_device, *surface)
        }
        .unwrap();
        let surface_formats = unsafe {
            instance_surface.get_physical_device_surface_formats(*physical_device, *surface)
        }
        .unwrap();
        let surface_present_mods = unsafe {
            instance_surface.get_physical_device_surface_present_modes(*physical_device, *surface)
        }
        .unwrap();

        let mut image_format = surface_formats.first().unwrap();
        'q: for surface_format_inloop in surface_formats.iter() {
            if surface_format_inloop.format == Format::B8G8R8A8_UNORM
                && surface_format_inloop.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            {
                image_format = surface_format_inloop;
                break 'q;
            }
        }

        let mut image_present_mode = PresentModeKHR::FIFO;
        'q: for surface_present_mode_inloop in surface_present_mods.iter() {
            if surface_present_mode_inloop == &PresentModeKHR::MAILBOX {
                image_present_mode = PresentModeKHR::MAILBOX;
                break 'q;
            }
        }

        let mut image_count = surface_capability.min_image_count + 1;

        if surface_capability.max_image_count > 0
            && image_count > surface_capability.max_image_count
        {
            image_count = surface_capability.max_image_count;
        }

        //log::trace!("Capability: {:?}", surface_capability);
        //log::trace!("Formats: {:?}", surface_formats);
        //log::trace!("Present Mods: {:?}", surface_present_mods);

        let instance_swapchain = swapchain::Device::new(instance, device);

        let mut swapchain_create_info = SwapchainCreateInfoKHR::default()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(image_format.format)
            .image_color_space(image_format.color_space)
            .image_extent(surface_capability.current_extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capability.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(image_present_mode)
            .clipped(true)
            .old_swapchain(SwapchainKHR::default());

        let queue_indexes = &[queue_graphic_index, queue_present_index];

        if queue_graphic_index != queue_present_index {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(queue_indexes);
        } else {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(SharingMode::EXCLUSIVE)
                .queue_family_indices(&[]);
        }

        let swapchain =
            unsafe { instance_swapchain.create_swapchain(&swapchain_create_info, None) }.unwrap();

        let images = unsafe { instance_swapchain.get_swapchain_images(swapchain) }.unwrap();

        let image_views: Vec<ImageView> = images
            .iter()
            .map(|image| {
                let image_view_create_info = ImageViewCreateInfo::default()
                    .image(*image)
                    .view_type(ImageViewType::TYPE_2D)
                    .format(image_format.format)
                    //.components(ComponentMapping::default())
                    .subresource_range(
                        ImageSubresourceRange::default()
                            .aspect_mask(ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );
                return unsafe { device.create_image_view(&image_view_create_info, None) }.unwrap();
            })
            .collect();

        Ok((swapchain, image_views))
    }

    fn create_debug_utils_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> Result<DebugUtilsMessengerEXT> {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(Self::vulkan_debug_callback));

        let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
        return Ok(unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        });
    }

    unsafe extern "system" fn vulkan_debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
        _user_data: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            Cow::from("")
        } else {
            ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            Cow::from("")
        } else {
            ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        log::warn!(
            "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
        );

        vk::FALSE
    }
}
