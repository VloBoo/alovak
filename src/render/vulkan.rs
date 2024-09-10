use ash::{
    ext::debug_utils,
    khr::{surface, win32_surface},
    vk::{self, DebugUtilsMessengerEXT, DeviceQueueCreateInfo, QueueFlags, SurfaceKHR},
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

        let device =
            Self::create_device(&entry, &instance, device_extension_names, &surface).unwrap();
        log::trace!("vulkan device created");

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
                    Err(error) => Err(crate::Error::Vulkan(error)),
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
    ) -> Result<Device> {
        // physical_devices
        let physical_devices = unsafe { instance.enumerate_physical_devices() }.unwrap();

        for physical_device in physical_devices {
            let mut queue_id_present = None;
            let mut queue_id_graphic = None;
            let physical_device_property =
                unsafe { instance.get_physical_device_properties(physical_device) };

            let a = physical_device_property.device_name.map(|v| v as u8);
            if let Some(pos) = a.iter().position(|&x| x == 0) {
                let slice = &a[..pos];
                let result = String::from_utf8_lossy(slice);
                log::trace!("physical device name: {:?}", result);
            }

            let queue_family_properties =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

            'q: for (index, queue_family_property) in queue_family_properties.iter().enumerate() {
                if queue_family_property
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
                {
                    queue_id_graphic = Some(index as u32);
                    break 'q;
                }
            }
            'q: for (index, _queue_family_property) in queue_family_properties.iter().enumerate() {
                if unsafe {
                    surface::Instance::new(entry, instance)
                        .get_physical_device_surface_support(
                            physical_device,
                            index as u32,
                            *surface,
                        )
                        .unwrap()
                } {
                    queue_id_present = Some(index as u32);
                    break 'q;
                }
            }
            log::trace!("{:?}, {:?}", queue_id_graphic, queue_id_present);

            let mut queue_ids = HashSet::new();

            match queue_id_graphic {
                Some(value) => queue_ids.insert(value),
                None => return Err(Error::Other("Queue dont found".to_owned())),
            };
            match queue_id_present {
                Some(value) => queue_ids.insert(value),
                None => return Err(Error::Other("Queue dont found".to_owned())),
            };

            let queue_create_infos: Vec<DeviceQueueCreateInfo> = queue_ids
                .iter()
                .map(|id| {
                    let mut queue_create_info =
                        DeviceQueueCreateInfo::default().queue_family_index(*id).queue_priorities(&[1.0f32; 1]);
                    queue_create_info.queue_count = 1;
                    queue_create_info
                })
                .collect();

            let device_create_info =
                vk::DeviceCreateInfo::default().queue_create_infos(&queue_create_infos);

            return match unsafe {
                instance.create_device(physical_device, &device_create_info, None)
            } {
                Ok(value) => Ok(value),
                Err(error) => Err(Error::Vulkan(error)),
            };
        }

        Err(Error::Other("Device dont found".to_owned()))
    }

    fn create_debug_utils_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> Result<DebugUtilsMessengerEXT> {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
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
