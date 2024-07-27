use windows::{
    core::*,
    Win32::{
        self, Foundation::*, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
    },
};

use crate::error::{Error, Result};

use super::{Handle, Window};

pub struct WindowWin32 {
    pub hwnd: HWND,
}

impl WindowWin32 {
    pub extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => {
                    log::trace!("WM_PAINT");

                    _ = ValidateRect(window, None);
                    LRESULT(0)
                }
                WM_DESTROY => {
                    log::trace!("WM_DESTROY");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => {
                    //log::trace!("{:?}", message);
                    DefWindowProcA(window, message, wparam, lparam)
                }
            }
        }
    }

    pub fn create(title: &str) -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleA(None).unwrap();
            let window_class = PCSTR::from_raw(title.as_bytes().as_ptr());

            let wc = WNDCLASSA {
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hInstance: instance.into(),
                lpszClassName: window_class,

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

            let hwnd = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                window_class,
                window_class,
                WS_OVERLAPPED | WS_VISIBLE | WS_SYSMENU | WS_MINIMIZEBOX,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                None,
            )
            .unwrap();

            tokio::spawn(async {
                log::warn!("TICK START");
                let mut message = MSG::default();
                while GetMessageA(&mut message, None, 0, 0).into() {
                    log::trace!("{:?}", message);
                    DispatchMessageA(&message);
                }
                log::warn!("TICK END");
            });

            Ok(WindowWin32 { hwnd })
        }
    }
}

impl Window for WindowWin32 {
    fn handle(self) -> Result<Handle> {
        Ok(Handle::Win32(self.hwnd))
    }
}
