use std::ffi::c_void;

use super::mempool::MemoryPoolMT;

type KeyType = u64;

/// Threadsafe Hash
///
/// Number of buckets must be a power of 2.
/// Key must be intp sized (32-bits on x32, 64-bits on x64)
/// Designed for a usage pattern where the data is semi-static, and there
/// is a well-defined point where we are guaranteed no queries are occurring.
///
/// Insertions are added into a thread-safe list, and when Commit() is called,
/// the insertions are moved into a lock-free list
///
/// Elements are never individually removed; clears must occur at a time
/// where we guaranteed no queries are occurring
#[repr(C)]
pub struct TSHashMap<T: Sized> {
    memory_pool: MemoryPoolMT,
    buckets: HashMapBucket<T>,
    needs_commit: bool,
}

pub struct TSHashMapUnallocIter<T: Sized> {
    bucket_ptr: *mut HashMapUnallocatedData<T>,
}

impl<T: Sized> Iterator for TSHashMapUnallocIter<T> {
    type Item = HashMapUnallocatedData<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bucket_ptr.is_null() {
            return None;
        }

        // TODO: Safety docs
        let bucket = unsafe { self.bucket_ptr.read() };
        self.bucket_ptr = bucket.next;
        Some(bucket)
    }
}

impl<T: Sized> TSHashMap<T> {
    pub fn unalloc_iter(&self) -> TSHashMapUnallocIter<T> {
        TSHashMapUnallocIter {
            bucket_ptr: self.buckets.unallocated_data,
        }
    }
}

#[repr(C)]
pub struct HashMapFixedData<T: Sized> {
    key: KeyType,
    next: *mut HashMapFixedData<T>,
    data: T,
}

// TODO: Insure T is sized to 0x10 bytes????
#[repr(C)]
pub struct HashMapFixedStructData<T: Sized> {
    data: T,
    key: KeyType,
    _pad_0x20: [u8; 0x8],
}

#[repr(C)]
pub struct HashMapStructData<T: Sized> {
    _pad_0x0: [u8; 0x10],
    list: [HashMapFixedStructData<T>; 256],
}

#[repr(C)]
pub struct HashMapAllocatedData<T: Sized> {
    _pad_0x0: [u8; 0x18],
    list: [HashMapFixedData<T>; 128],
}

#[repr(C)]
pub struct HashMapBucketData<T: Sized> {
    data: T,
    next: *mut HashMapFixedData<T>,
    key: KeyType,
}

#[repr(C)]
pub struct HashMapUnallocatedData<T: Sized> {
    next: *mut HashMapUnallocatedData<T>,
    key: KeyType,
    ui_key: KeyType,
    i_key: KeyType,
    // TODO: Make this a variable array. (memory pool)
    current_block_list: [HashMapBucketData<T>; 256],
}

impl<T: Sized> HashMapUnallocatedData<T> {
    // TODO: Get from key
    // TODO: Add hashmap functionality to this...
}

#[repr(C)]
pub struct HashMapBucket<T: Sized> {
    struct_data: *mut HashMapStructData<T>,
    // TODO: Type this as fastmutex array?
    mutex_list: *mut c_void,
    allocated_data: *mut HashMapAllocatedData<T>,
    unallocated_data: *mut HashMapUnallocatedData<T>,
}
