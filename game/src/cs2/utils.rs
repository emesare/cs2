use std::ffi::{c_char, c_void};

use windows::Win32::{
    Foundation::{FARPROC, HINSTANCE},
    System::LibraryLoader::GetProcAddress,
};

use crate::{c_str, win_pcstr};

pub unsafe fn get_interface<T: Sized>(module: HINSTANCE, name: &str) -> Option<*mut T> {
    // Get create_interface from the specified module. (i.e. client.dll)
    let create_interface_ptr: FARPROC = GetProcAddress(module, win_pcstr!("CreateInterface"));
    let create_interface = std::mem::transmute::<
        _,
        unsafe extern "C" fn(name: *const c_char, return_code: i8) -> *const c_void,
    >(create_interface_ptr.unwrap());

    // Retrieve interface using `CreateInterface`.
    let interface = create_interface(c_str!("{}", name), 0);

    if interface.is_null() {
        None
    } else {
        Some(interface as *mut T)
    }
}
