use alovak::{vulkan::Vulkan, win32::WindowWin32, Window};
use casopis::Casopis;
use log::Level;

#[tokio::main]
async fn main() {
    Casopis::init(Level::Trace).unwrap();

    let win: WindowWin32 = WindowWin32::create("alovak\0").unwrap();

    let vulkan = Vulkan::init(win.handle().unwrap()).unwrap();
}
