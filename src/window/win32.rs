use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};

use super::Window;

pub struct WindowWin32 {
    pub hwnd: HWND
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
}

impl Window for WindowWin32 {
     fn create(title: &str) -> std::result::Result<WindowWin32, &str> {
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

            log::warn!("{:?}",hwnd);

            let mut message = MSG::default();

            while GetMessageA(&mut message, None, 0, 0).into() {
                DispatchMessageA(&message);
            }
            Ok(WindowWin32 {hwnd})
        }
    }
}
