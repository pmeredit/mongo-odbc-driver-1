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
    #[nwg_resource(source_file: Some("./Banner.bmp"))]
     banner: nwg::Bitmap,
    #[nwg_control(size: (600, 500), position: (500, 500), title: "AtlasSQL ODBC Driver Source Configuration", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [BasicApp::close] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 10)]
    grid: nwg::GridLayout,

    #[nwg_control(bitmap: Some(&data.banner))]
    #[nwg_layout_item(layout: grid, row: 0, col: 0, col_span: 7, row_span: 2)]
    frame: nwg::ImageFrame,

    #[nwg_control(flags: "VISIBLE", text: "User")]
    #[nwg_layout_item(layout: grid, row: 2, col: 1, col_span: 1)]
    user_field: nwg::Label,
    
    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid, row: 2, col: 2, col_span: 5)]
    user_input: nwg::TextBox,
    
    #[nwg_control(flags: "VISIBLE", text: "Password")]
    #[nwg_layout_item(layout: grid, row: 3, col: 1, col_span: 1)]
    password_field: nwg::Label,
    
    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid, row: 3, col: 2, col_span: 5)]
    password_input: nwg::TextBox,

    #[nwg_control(flags: "VISIBLE", text: "Mongo URI:")]
    #[nwg_layout_item(layout: grid, row: 4, col: 0, col_span: 6)]
    #[nwg_events( OnButtonClick: [BasicApp::choose_uri] )]
    radio_uri: nwg::RadioButton,

    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid, row: 5, col: 1, col_span: 6)]
    uri_input: nwg::TextBox,

    #[nwg_control(flags: "VISIBLE", text: "Connection Properties:")]
    #[nwg_events( OnButtonClick: [BasicApp::choose_props] )]
    #[nwg_layout_item(layout: grid, row: 6, col: 0, col_span: 6)]
    radio_props: nwg::RadioButton,

    #[nwg_control(flags: "VISIBLE", text: "Server")]
    #[nwg_layout_item(layout: grid, row: 7, col: 1, col_span: 1)]
    server_field: nwg::Label,
    
    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid, row: 7, col: 2, col_span: 5)]
    server_input: nwg::TextBox,

    #[nwg_control(flags: "VISIBLE", text: "Database")]
    #[nwg_layout_item(layout: grid, row: 8, col: 1, col_span: 1)]
    database_field: nwg::Label,
    
    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid, row: 8, col: 2, col_span: 5)]
    database_input: nwg::TextBox,

    #[nwg_control(flags: "VISIBLE", text: "Test")]
    #[nwg_layout_item(layout: grid, row: 9, col: 2, col_span: 1)]
    test_button: nwg::Button,

    #[nwg_control(flags: "VISIBLE", text: "Ok")]
    #[nwg_layout_item(layout: grid, row: 9, col: 4, col_span: 1)]
    ok_button: nwg::Button,

    #[nwg_control(flags: "VISIBLE", text: "Cancel")]
    #[nwg_layout_item(layout: grid, row: 9, col: 5, col_span: 1)]
    cancel_button: nwg::Button,

    #[nwg_control(flags: "VISIBLE", text: "Help")]
    #[nwg_layout_item(layout: grid, row: 9, col: 6, col_span: 1)]
    help_button: nwg::Button,
}

impl BasicApp {
    fn choose_uri(&self) {
        self.radio_props.set_check_state(nwg::RadioButtonState::Unchecked);
    }

    fn choose_props(&self) {
        self.radio_uri.set_check_state(nwg::RadioButtonState::Unchecked);
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn init_gui() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
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
    init_gui();
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
