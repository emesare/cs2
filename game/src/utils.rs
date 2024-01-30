use std::{fs::OpenOptions, os::windows::prelude::AsRawHandle};

use windows::Win32::{
    Foundation::{BOOL, HANDLE, HINSTANCE, HWND, LPARAM, RECT},
    System::{
        Console::{
            AllocConsole, GetConsoleWindow, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
        },
        Diagnostics::Debug::IMAGE_NT_HEADERS32,
        LibraryLoader::GetModuleHandleA,
        SystemServices::IMAGE_DOS_HEADER,
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowLongA, GetWindowRect, GetWindowThreadProcessId,
        GWL_HINSTANCE, GW_OWNER,
    },
};

#[macro_export]
macro_rules! c_str {
    ($string:expr) => {
        concat!($string, "\0").as_ptr() as *const core::ffi::c_char
    };
    ($fmt:expr, $($arg:tt)*) => (format!(concat!($fmt, "\0"), $($arg)*).as_ptr() as *const core::ffi::c_char);
}

#[macro_export]
macro_rules! win_pcstr {
    ($string:expr) => {
        windows::core::PCSTR(concat!($string, "\0").as_ptr())
    };
    ($fmt:expr, $($arg:tt)*) => (windows::core::PCSTR(format!(concat!($fmt, "\0"), $($arg)*).as_ptr()));
}

pub unsafe fn alloc_console() {
    AllocConsole();

    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .open("CONOUT$")
        .expect("should be able to open CONOUT$");

    SetStdHandle(STD_OUTPUT_HANDLE, HANDLE(file.as_raw_handle() as _))
        .expect("`STD_OUTPUT_HANDLE` should be set to CONOUT$ handle");
    SetStdHandle(STD_ERROR_HANDLE, HANDLE(file.as_raw_handle() as _))
        .expect("`STD_ERROR_HANDLE` should be set to CONOUT$ handle");
    std::mem::forget(file);
}

pub unsafe fn get_window_hinstance(hwnd: HWND) -> HINSTANCE {
    HINSTANCE(GetWindowLongA(hwnd, GWL_HINSTANCE) as isize)
}

pub fn get_window_size(hwnd: HWND) -> Option<(i32, i32)> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect).ok()? };
    Some((rect.right - rect.left, rect.bottom - rect.top))
}

/// Gets the first enumerated window that belongs to the process.
pub fn get_window_hwnd() -> Option<HWND> {
    /// Enumerate window handles and store our window [HWND] in `l_param`.
    ///
    /// Safety: TODO
    unsafe extern "system" fn enum_windows_cb(hwnd: HWND, l_param: LPARAM) -> BOOL {
        let mut wnd_proc_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut wnd_proc_id));
        match std::process::id() != wnd_proc_id {
            true => true.into(),
            false => {
                if GetWindow(hwnd, GW_OWNER).0 == 0 && GetConsoleWindow() != hwnd {
                    *(l_param.0 as *mut HWND) = hwnd;
                    return false.into();
                }
                true.into()
            }
        }
    }

    let mut output = HWND(0);
    // TODO: Safety message
    unsafe {
        EnumWindows(
            Some(enum_windows_cb),
            std::mem::transmute::<_, LPARAM>(&mut output),
        );
    };

    match output.0 == 0 {
        true => None,
        false => Some(output),
    }
}

pub fn get_module(module_name: &str) -> Option<HINSTANCE> {
    unsafe { Some(GetModuleHandleA(win_pcstr!("{}", module_name)).ok()?.into()) }
}

pub fn module_addr(module: HINSTANCE) -> usize {
    unsafe { std::mem::transmute(module) }
}

// TODO: Make this unsafe?
// TODO: Use some pe header library to get rid of this unholy code.
pub fn module_to_bytes(module: HINSTANCE) -> &'static [u8] {
    let module_addr = unsafe { std::mem::transmute::<_, usize>(module) };
    let dos_headers = module_addr as *const IMAGE_DOS_HEADER;
    let e_lfanew = (unsafe { *dos_headers }).e_lfanew as i32;
    let nt_headers = (module_addr + e_lfanew as usize) as *mut IMAGE_NT_HEADERS32;
    let size_of_image = (unsafe { *nt_headers }).OptionalHeader.SizeOfImage as usize;
    let bytes = module_addr as *mut u8;
    let module_bytes = unsafe { std::slice::from_raw_parts(bytes, size_of_image) };
    module_bytes
}
