//#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::{
        Foundation::{HINSTANCE, HWND},
        System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        UI::WindowsAndMessaging::*,
    },
};

//#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn DllMain(_: HINSTANCE, reason_for_call: u32, _: usize) -> i32 {
    unsafe {
        match reason_for_call {
            DLL_PROCESS_ATTACH => {
                MessageBoxW(None, w!("ATTACH"), w!("ATTACHS"), MB_OK);
            }
            DLL_PROCESS_DETACH => {
                MessageBoxW(None, w!("DETACH"), w!("DETACHS"), MB_OK);
            }
            _ => {
                MessageBoxW(None, w!("UNKNOWN"), w!("UNKNOWN"), MB_OK);
            }
        }
        //        CreateWindowExW(
        //            WS_EX_LAYERED,
        //            w!("AtlasSQL ODBC Data Source Setup"),
        //            w!("FOO"),
        //            WS_OVERLAPPEDWINDOW,
        //            200,
        //            200,
        //            400,
        //            300,
        //            None,
        //            None,
        //            None,
        //            None,
        //        );
    }
    1
}

#[no_mangle]
pub extern "system" fn ConfigDSNW(
    _: HWND,
    _request: u32,
    driver: PCWSTR,
    attributes: PCWSTR,
) -> bool {
    unsafe {
        MessageBoxW(None, w!("CONFIG"), w!("CONFIG"), MB_OK);
        MessageBoxW(None, driver, attributes, MB_OK);
    }
    true
}

#[no_mangle]
pub extern "system" fn Driver_Prompt(
    _: HWND,
    _: *const u16,
    _: u16,
    _: *mut u16,
    _: u16,
    _: *mut u16,
) -> bool {
    unsafe {
        MessageBoxW(None, w!("DRIVER PROMPT"), w!("DRIVER PROMPT"), MB_OK);
    }
    true
}
