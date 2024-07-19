use ash::{
    ext::debug_utils,
    khr::win32_surface,
    vk::{self, SurfaceKHR},
    Entry, Instance,
};
use std::{
    borrow::Cow,
    ffi::{self, c_char},
};

use crate::window::win32::WindowWin32;

pub struct Vulkan {}

impl Vulkan {
    pub fn init(surface: &WindowWin32) -> Result<Self, ()> {
        let layer_names = vec![b"VK_LAYER_KHRONOS_validation\0"];

        let layers_names_raw: Vec<*const c_char> = layer_names
            .into_iter()
            .map(|raw_name| unsafe { ffi::CStr::from_bytes_with_nul_unchecked(raw_name).as_ptr() })
            .collect();

        let extension_names = vec![b"VK_EXT_debug_utils\0  ",b"VK_KHR_win32_surface\0", b"VK_KHR_surface\0      "];

        let extensions_names_raw: Vec<*const c_char> = extension_names
            .into_iter()
            .map(|raw_name| unsafe { ffi::CStr::from_bytes_with_nul_unchecked(raw_name).as_ptr() })
            .collect();

        let entry = Entry::linked();
        log::trace!("vulkan entry created");

        let instance =
            Self::create_instance(&entry, layers_names_raw, extensions_names_raw).unwrap();
        log::trace!("vulkan instance created");

        Self::create_debug_utils_messenger(&entry, &instance);
        log::trace!("vulkan debug utils messenger created");

        let surface = Self::create_surface(surface, &entry, &instance).unwrap();
        log::trace!("vulkan surface created");

        return Ok(Vulkan {});
    }

    fn create_instance(
        entry: &Entry,
        layer_names: Vec<*const c_char>,
        extension_names: Vec<*const c_char>,
    ) -> Result<Instance, vk::Result> {
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

        return unsafe { entry.create_instance(&create_info, None) };
    }

    fn create_surface(
        window: &WindowWin32,
        entry: &Entry,
        instance: &Instance,
    ) -> Result<SurfaceKHR, vk::Result> {
        let win32_surface_create_info = vk::Win32SurfaceCreateInfoKHR::default();
        //win32_surface_create_info.hwnd(window.hwnd as isize);
        let win32_surface_loader = win32_surface::Instance::new(&entry, &instance);
        return unsafe {
            win32_surface_loader.create_win32_surface(&win32_surface_create_info, None)
        };
    }

    fn create_device() {}

    fn create_debug_utils_messenger(entry: &Entry, instance: &Instance) {
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
        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };
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
