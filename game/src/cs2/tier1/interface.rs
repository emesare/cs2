use std::ffi::{c_char, c_void};

// TODO: Interface might have been moved to tier0 in source2
// See: https://github.com/perilouswithadollarsign/cstrike15_src/blob/f82112a2388b841d72cb62ca48ab1846dfcc11c8/public/tier1/interface.h#L39C9-L39C9

#[repr(C)]
pub enum InterfaceReturnStatus {
    Ok = 0,
    // TODO: Way to say n != 0 is always Failed? Answer: Encode in the `from` function.
    Failed,
}

pub type CreateInterfaceFn =
    extern "C" fn(name: *const c_char, return_code: *mut InterfaceReturnStatus) -> *const c_void;
pub type InstantiateInterfaceFn = extern "C" fn() -> *const c_void;

#[repr(C)]
pub struct InterfaceReg {
    create_fn: InstantiateInterfaceFn,
    name: *const c_char,
    next: *mut InterfaceReg,
}
