use render::vulkan::Vulkan;
use casopis::Casopis;
use log::Level;
use window::{win32::WindowWin32, Window};

mod render;
mod window;

fn main() {
    Casopis::init(Level::Trace);
    let win: WindowWin32 = WindowWin32::create("alovak\0").unwrap();
    let vulkan = Vulkan::init(&win);
}