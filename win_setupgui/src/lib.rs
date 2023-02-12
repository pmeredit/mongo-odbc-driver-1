//#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::{
        Foundation::HINSTANCE,
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
