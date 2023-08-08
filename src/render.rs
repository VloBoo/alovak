pub use ash::vk::PhysicalDevice;
pub use ash::{vk, Device, Entry, Instance};

pub struct Render {
    entry: Entry,
    instance: Instance,
    physical_device: PhysicalDevice,
}

impl Render {
    pub fn new() -> Result<Self, ()> {
        let entry = Entry::linked();
        let instance = Self::create_instance(&entry).unwrap();
        let physical_device = Self::pick_physical_device(&instance).unwrap();
        let render = Render {
            entry: entry,
            instance: instance,
            physical_device: physical_device,
        };
        return Ok(render);
    }

    fn create_instance(entry: &Entry) -> Result<Instance, vk::Result> {
        let app_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_0,
            p_application_name: "Test Application Alovak".as_ptr() as *const i8,
            application_version: vk::make_api_version(0, 0, 0, 1),
            p_engine_name: "Alovak".as_ptr() as *const i8,
            engine_version: vk::make_api_version(0, 0, 0, 1),
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None) }?;
        return Ok(instance);
    }

    fn create_surface(entry: &Entry) -> Result<vk::SurfaceKHR, vk::Result> {
        return Err(vk::Result::ERROR_UNKNOWN);
    }

    fn pick_physical_device(instance: &Instance) -> Option<PhysicalDevice> {
        let physical_devices = match unsafe { instance.enumerate_physical_devices() } {
            Ok(var) => var,
            Err(_) => return None,
        };

        for physical_device in physical_devices {
            let physical_device_properties =
                unsafe { instance.get_physical_device_properties(physical_device) };

            let mut queuu_family_graphic: Option<u32> = None;
            //let mut queuu_family_present: Option<u32> = None;
            println!("\n\n {:#?} \n\n", physical_device_properties);
            let queue_family_properties =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

            for (index, queue) in queue_family_properties.iter().enumerate() {
                println!("\n {:#?} \n", queue);
                if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    queuu_family_graphic = Some(index as u32);
                }
            }
            return Some(physical_device);
        }

        return None;
    }
}

impl Drop for Render {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
