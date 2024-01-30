use std::ffi::{c_char, c_void};

use super::threadtools::ThreadFastMutex;

/// Ways a [MemoryPool] can grow when it needs to make a new [Blob].
#[repr(C)]
pub enum GrowMode {
    /// Don't allow new blobs.
    NONE = 0,
    /// Increase the [Blob::size] every allocation.
    FAST = 1,
    /// Constant [Blob::size] every allocation.
    SLOW = 2,
}

#[repr(C)]
pub struct MemoryPool {
    block_size: i32,
    blocks_per_blob: i32,
    grow_mode: GrowMode,
    blocks_allocated: i32,
    block_allocated_size: i32,
    peak_alloc: i32,
    // TODO: Do these even exist?
    alignment: u16,
    blob_num: u16,
    free_list_head: *mut c_void,
    alloc_owner: *const c_char,
    blob_head: Blob,
}

impl MemoryPool {}

#[repr(C)]
pub struct MemoryPoolMT {
    pub mp: MemoryPool,
    mutex: ThreadFastMutex,
}

#[repr(C)]
struct Blob {
    prev: *mut Blob,
    next: *mut Blob,
    /// Size (in number of bytes) of [Self::data].
    size: isize,
    data: [u8; 1],
    _align_pad: [u8; 1],
}
