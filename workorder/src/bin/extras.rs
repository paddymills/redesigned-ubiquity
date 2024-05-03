
use std::ffi::CString;
use std::ptr::null_mut;
use winapi::um::winuser::{FindWindowA, SendMessageA, WM_COMMAND};

fn main() {
    // Find the Excel window by its class name
    let class_name = CString::new("XLMAIN").expect("CString::new failed");
    let window_handle = unsafe { FindWindowA(class_name.as_ptr(), null_mut()) };

    if window_handle == null_mut() {
        println!("Excel window not found");
        return;
    }

    let macro_name = CString::new("Book1.xlsm!export").expect("CString::new failed");

    // Call the Excel macro using WM_COMMAND message
    let macro_id = 0; // ID of the macro (1 is just an example, replace with the actual ID)

    unsafe {
        SendMessageA(window_handle, WM_COMMAND, macro_id as usize, macro_name.as_ptr() as isize);
    }

    println!("Excel macro called successfully");
}