#[repr(C)]
pub struct ThreadFastMutex {
    owner_id: u32,
    depth: isize,
}

pub type ThreadId = usize;

#[repr(C)]
pub struct ThreadSpinRWLock {
    // TODO: https://github.com/perilouswithadollarsign/cstrike15_src/blob/f82112a2388b841d72cb62ca48ab1846dfcc11c8/public/tier0/threadtools.h#L1704
    lock_info: u32,
    writer_id: ThreadId,
    // TODO: Got some more stuff to fill here.
}
