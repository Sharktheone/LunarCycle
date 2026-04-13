use core::num::NonZeroUsize;
use core::ptr::{NonNull, null_mut};
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{
    MEM_COMMIT, MEM_DECOMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_NOACCESS, PAGE_READWRITE,
};

pub(crate) unsafe fn reserve(size: NonZeroUsize) -> Option<NonNull<u8>> {
    let ptr = unsafe { VirtualAlloc(null_mut(), size, MEM_RESERVE, PAGE_NOACCESS) };

    if ptr.is_null() {
        // core::hint::cold_path();
        return None;
    }

    unsafe {
        // Safety: we've checked above that the ptr is not null
        Some(NonNull::new_unchecked(ptr).cast())
    }
}

pub(crate) unsafe fn commit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    let res = unsafe { VirtualAlloc(ptr.as_ptr().cast(), size.get(), MEM_COMMIT, PAGE_READWRITE) };

    !res.is_null()
}

pub(crate) unsafe fn decommit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    let res = unsafe { VirtualFree(ptr.as_ptr().cast(), size.get(), MEM_DECOMMIT) };

    res != 0
}

pub(crate) unsafe fn release(ptr: NonNull<u8>) -> bool {
    let res = unsafe { VirtualFree(ptr.as_ptr().cast(), 0, MEM_RELEASE) };

    res != 0
}

pub(crate) unsafe fn alloc(size: NonZeroUsize) -> Option<NonNull<u8>> {
    let ptr = unsafe {
        VirtualAlloc(
            null_mut(),
            size.get(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    if ptr.is_null() {
        // core::hint::cold_path();
        return None;
    }

    unsafe {
        // Safety: we've checked above that the ptr is not null
        Some(NonNull::new_unchecked(ptr).cast())
    }
}

pub(crate) unsafe fn free(ptr: NonNull<u8>) -> bool {
    let res = unsafe { VirtualFree(ptr.as_ptr().cast(), 0, MEM_RELEASE) };

    res != 0
}
