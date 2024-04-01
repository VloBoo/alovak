pub use ash::vk::PhysicalDevice;
pub use ash::{vk, Device, Entry, Instance};

pub struct Render {}

impl Render {
    pub fn new() -> Result<Self, String> {
        let entry = Entry::linked();
        let instance: Instance;
        let surface: vk::SurfaceKHR;
        let mut queue_family_graphic_index: u32;
        let queue_family_present_index: u32;
        let physical_device: PhysicalDevice;

        // creating instance

        let layers: Vec<*const i8> = vec![b"VK_LAYER_KHRONOS_validation\0"]
            .iter()
            .map(|&s| s.as_ptr() as *const i8)
            .collect();
        let extension: Vec<*const i8> = vec![b"VK_KHR_surface\0"]
            .iter()
            .map(|&s| s.as_ptr() as *const i8)
            .collect();

        let app_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_0,
            p_application_name: "Test Application Alovak".as_ptr() as *const i8,
            application_version: vk::make_api_version(0, 0, 0, 1),
            p_engine_name: "Alovak".as_ptr() as *const i8,
            engine_version: vk::make_api_version(0, 0, 0, 1),
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extension);
        // println!("{:#?}", layers.as_ptr());
        instance = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(instance) => instance,
            Err(error) => return Err(error.to_string()),
        };

        // pick physical device

        if let Ok(physical_devices) = unsafe { instance.enumerate_physical_devices() } {
            for pd in physical_devices {
                let physical_device_properties =
                    unsafe { instance.get_physical_device_properties(pd) };

                let mut queue_family_graphic: Option<u32> = None;
                let mut queue_family_present: Option<u32> = None;
                // println!("\n\n {:#?} \n\n", physical_device_properties);
                let queue_family_properties = unsafe {
                    instance.get_physical_device_queue_family_properties(pd)
                };

                for (index, queue) in queue_family_properties.iter().enumerate() {
                    // println!("\n {:#?} \n", queue);
                    if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        queue_family_graphic = Some(index as u32);
                    }
                    // let a = 0;
                    // let present_support: vk::Bool32 = 0;
                    // vk::PFN_vkGetPhysicalDeviceSurfaceSupportKHR(
                    //     physical_device,
                    //     index as u32,
                    //     a,
                    //     present_support,
                    // );
                }

                if let Some(qfg) = queue_family_graphic{
                    if let Some(qfp) = queue_family_present{
                        queue_family_graphic_index = qfp;
                    }
                }
            }
        }

        return Ok(Self {});
    }
}

impl Drop for Render {
    fn drop(&mut self) {
        unsafe {
            //self.instance.destroy_instance(None);
        }
    }
}
