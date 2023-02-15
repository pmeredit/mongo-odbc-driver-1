#![windows_subsystem = "windows"]
/*!
    A very simple application that shows your name in a message box.
    Unlike `basic_d`, this example uses layout to position the controls in the window
*/

//#[cfg(target_os = "windows")]
use widechar::to_widechar_vec;
use windows::{
    core::{w, PCSTR, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND},
        System::SystemServices::{
            DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
        },
        UI::WindowsAndMessaging::{MessageBoxA, MessageBoxW, MB_OK},
    },
};

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_control(size: (300, 115), position: (300, 300), title: "Basic example", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [BasicApp::say_goodbye] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_resource(source_file: Some("./Banner.bmp"), size: Some((15, 15)))]
    #[nwg_layout_item(layout: grid, row: 0, col: 0, col_span: 2)]
    banner: nwg::Bitmap,

    #[nwg_control(text: "", focus: true)]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    driver_name: nwg::TextInput,

    #[nwg_control(text: "Description", focus: false)]
    #[nwg_layout_item(layout: grid, row: 1, col: 1)]
    description: nwg::TextInput,

    #[nwg_control(text: "Say my name")]
    #[nwg_layout_item(layout: grid, col: 0, row: 2, row_span: 2, col_span: 2)]
    #[nwg_events( OnButtonClick: [BasicApp::say_hello] )]
    hello_button: nwg::Button,
}

impl BasicApp {
    fn say_hello(&self) {
        nwg::modal_info_message(
            &self.window,
            "Hello",
            &format!("Hello {}", self.driver_name.text()),
        );
    }

    fn say_goodbye(&self) {
        nwg::modal_info_message(
            &self.window,
            "Goodbye",
            &format!("Goodbye {}", self.driver_name.text()),
        );
        nwg::stop_thread_dispatch();
    }
}

fn init_gui(driver_name: &str) {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let mut app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    app.driver_name.set_text(driver_name);
    nwg::dispatch_thread_events();
}

//#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn DllMain(_: HINSTANCE, reason_for_call: u32, _: usize) -> i32 {
    unsafe {
        match reason_for_call {
            DLL_PROCESS_ATTACH => {
                //MessageBoxW(None, w!("ATTACH1"), w!("ATTACH2"), MB_OK);
            }
            DLL_PROCESS_DETACH => {
                //MessageBoxW(None, w!("DETACH1"), w!("DETACH2"), MB_OK);
            }
            DLL_THREAD_ATTACH => {
                //MessageBoxW(None, w!("TA1"), w!("TA2"), MB_OK);
            }
            DLL_THREAD_DETACH => {
                //MessageBoxW(None, w!("TD1"), w!("TD2"), MB_OK);
            }
            _ => {
                //MessageBoxW(None, w!("U"), w!("U"), MB_OK);
            }
        }
    }
    1
}

#[no_mangle]
pub extern "system" fn ConfigDSNW(
    _: HWND,
    request: u32,
    driver: PCWSTR,
    attributes: PCWSTR,
) -> bool {
    unsafe {
        let driver_name = format!("{:?}", driver.to_string());
        init_gui(&driver_name);
    }
    true
}

#[no_mangle]
pub extern "system" fn ConfigDSN(_: HWND, request: u32, driver: PCSTR, attributes: PCSTR) -> bool {
    unsafe {
        MessageBoxW(None, w!("CONFIG1A"), w!("CONFIG2A"), MB_OK);
        MessageBoxA(None, driver, attributes, MB_OK);
        let o = to_widechar_vec(&request.to_string());
        let o = PCWSTR::from_raw(o.as_ptr());
        MessageBoxW(None, o, w!("REQUEST"), MB_OK);
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
