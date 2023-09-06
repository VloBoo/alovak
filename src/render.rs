pub use ash::vk::PhysicalDevice;
pub use ash::{vk, Device, Entry, Instance};

mod render_builder;
use render_builder::RenderBuilder;

pub struct Render {}

impl Render {
    pub fn new() -> Result<Self, ()> {
        RenderBuilder::build();
        return Err(());
    }
}

impl Drop for Render {
    fn drop(&mut self) {
        unsafe {
            //self.instance.destroy_instance(None);
        }
    }
}
