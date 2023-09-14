use std::collections::HashSet;

use ash::vk::Bool32;
pub use ash::vk::PhysicalDevice;
pub use ash::{vk, vk::Queue, Device, Entry, Instance};

use crate::Render;

pub struct RenderBuilder {
    entry: Option<Entry>,
    instance: Option<Instance>,
    surface: Option<vk::SurfaceKHR>,
    surface_format: Option<vk::SurfaceFormatKHR>,
    present_mode: Option<vk::PresentModeKHR>,
    queue_family_graphic: Option<u32>,
    queue_family_present: Option<u32>,
    physical_device: Option<PhysicalDevice>,
    logical_device: Option<Device>,
}

impl RenderBuilder {
    pub fn build() -> Result<Render, vk::Result> {
        let mut render_builder = RenderBuilder {
            entry: Some(Entry::linked()),
            instance: None,
            surface: None,
            surface_format: None,
            present_mode: None,
            queue_family_graphic: None,
            queue_family_present: None,
            physical_device: None,
            logical_device: None,
        };
        render_builder
            .create_instance()
            .create_surface()
            .pick_physical_device()
            .create_device();
        match render_builder.logical_device {
            Some(_) => Ok(Render {}),
            None => Err(vk::Result::ERROR_UNKNOWN),
        }
    }

    fn create_instance(&mut self) -> &mut Self {
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
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extension)
            .build();
        // println!("{:#?}", layers.as_ptr());
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
                let mut _physical_device: Option<PhysicalDevice> = None;
                if let Ok(physical_devices) = unsafe { instance.enumerate_physical_devices() } {
                    for physical_device in physical_devices {
                        let physical_device_properties =
                            unsafe { instance.get_physical_device_properties(physical_device) };

                        let mut queue_family_graphic: Option<u32> = None;
                        let mut queue_family_present: Option<u32> = Some(1);
                        // println!("\n\n {:#?} \n\n", physical_device_properties);
                        let queue_family_properties = unsafe {
                            instance.get_physical_device_queue_family_properties(physical_device)
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
                        self.queue_family_graphic = queue_family_graphic;
                        self.queue_family_present = queue_family_graphic;
                        if queue_family_graphic.is_some()
                        /*|| queue_family_present.is_some()*/
                        {
                            _physical_device = Some(physical_device);
                            break;
                        }
                    }
                }
                _physical_device
            }
            None => None,
        };
        return self;
    }

    fn create_device(&mut self) -> &mut Self {
        println!("\n {} \n", self.physical_device.is_some());
        self.logical_device = match self.queue_family_graphic {
            Some(queue_graphic) => match self.queue_family_present {
                Some(queue_present) => {
                    let mut queues_index: HashSet<u32> = HashSet::new();
                    queues_index.insert(queue_graphic);
                    queues_index.insert(queue_present);
                    let prior: f32 = 1.0;
                    let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = queues_index
                        .iter()
                        .map(|&i| vk::DeviceQueueCreateInfo {
                            queue_family_index: i,
                            queue_count: 1,
                            p_queue_priorities: &prior,
                            ..Default::default()
                        })
                        .collect();

                    let device_create_info = vk::DeviceCreateInfo {
                        queue_create_info_count: queue_create_infos.len() as u32,
                        p_queue_create_infos: queue_create_infos.as_ptr(),
                        ..Default::default()
                    };

                    match &self.instance {
                        Some(instance) => match &self.physical_device {
                            Some(physical_device) => unsafe {
                                instance.create_device(*physical_device, &device_create_info, None)
                            }
                            .ok(),
                            None => None,
                        },
                        None => None,
                    }
                }
                None => None,
            },
            None => None,
        };
        return self;
    }
}
