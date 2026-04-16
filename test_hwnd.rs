use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG};

fn main() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        let _ = GetMessageW(&mut msg, 0 as HWND, 0, 0);
    }
}
