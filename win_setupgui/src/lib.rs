//#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::{Foundation::HINSTANCE, UI::WindowsAndMessaging::*},
};

//#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn DllMain(hinstance: HINSTANCE, _: u32, _: usize) -> i32 {
    unsafe {
        CreateWindowExW(
            WS_EX_LAYERED,
            w!("AtlasSQL ODBC Data Source Setup"),
            w!("FOO"),
            WS_OVERLAPPEDWINDOW,
            200,
            200,
            400,
            300,
            None,
            None,
            hinstance,
            None,
        );
    }
    1
}
