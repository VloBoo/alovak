use windows::Win32::Foundation::HWND;

use crate::error::Result;

pub mod win32;

pub trait Window {
    fn handle(self) -> Result<Handle>;
}


pub enum Handle{
    Win32(HWND),
}