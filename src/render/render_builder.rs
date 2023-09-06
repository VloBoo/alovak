pub use ash::vk::PhysicalDevice;
pub use ash::{vk, Device, Entry, Instance};

use crate::Render;

pub struct RenderBuilder {
    entry: Option<Entry>,
    instance: Option<Instance>,
    surface: Option<vk::SurfaceKHR>,
    physical_device: Option<PhysicalDevice>,
    queue_family_graphic: Option<u32>,
    queue_family_present: Option<u32>,
}

impl RenderBuilder {
    pub fn build() -> Result<Render, vk::Result> {
        let mut render_builder = RenderBuilder {
            entry: Some(Entry::linked()),
            instance: None,
            surface: None,
            physical_device: None,
            queue_family_graphic: None,
            queue_family_present: None,
        };
        render_builder
            .create_instance()
            .create_surface()
            .pick_physical_device();
        return Err(vk::Result::INCOMPLETE);
    }

    fn create_instance(&mut self) -> &mut Self {
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

        self.instance = match &self.entry {
            Some(a) => unsafe { a.create_instance(&create_info, None) }.ok(),
            None => None,
        };
        return self;
    }

    fn create_surface(&mut self) -> &mut Self {
        return self;
    }

    fn pick_physical_device(&mut self) -> &mut Self {
        self.physical_device = match &self.instance {
            Some(instance) => {
                if let Ok(physical_devices) = unsafe { instance.enumerate_physical_devices() } {
                    for physical_device in physical_devices {
                        let physical_device_properties =
                            unsafe { instance.get_physical_device_properties(physical_device) };

                        let mut queue_family_graphic: Option<u32> = None;
                        let mut queue_family_present: Option<u32> = None;
                        println!("\n\n {:#?} \n\n", physical_device_properties);
                        let queue_family_properties = unsafe {
                            instance.get_physical_device_queue_family_properties(physical_device)
                        };

                        for (index, queue) in queue_family_properties.iter().enumerate() {
                            println!("\n {:#?} \n", queue);
                            if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                                queue_family_graphic = Some(index as u32);
                            }
                        }
                        self.queue_family_graphic = queue_family_graphic;
                        self.queue_family_present = queue_family_present;
                        Some(physical_device);
                    }
                }
                None
            }
            None => None,
        };
        return self;
    }
}
