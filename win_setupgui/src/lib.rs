#[cfg(target_os = "windows")]
use windows::{core::*, Win32::UI::WindowsAndMessaging::*};

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn DllMain(_: usize, _: u32, _: usize) -> i32 {
    unsafe {
        MessageBoxW(None, w!("Wide"), w!("Caption"), MB_OK);
    }
    1
}
