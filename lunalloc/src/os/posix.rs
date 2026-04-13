use core::num::NonZeroUsize;
use core::ptr::NonNull;

pub(crate) unsafe fn reserve(size: NonZeroUsize) -> Option<NonNull<u8>> {
    let ptr = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            size.get(),
            libc::PROT_NONE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED || ptr.is_null() {
        // core::hint::cold_path();
        return None;
    }

    unsafe {
        // Safety: we've checked above that the ptr is not null
        Some(NonNull::new_unchecked(ptr).cast())
    }
}

pub(crate) unsafe fn commit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    let res = unsafe {
        libc::mprotect(
            ptr.as_ptr() as *mut _,
            size.get(),
            libc::PROT_READ | libc::PROT_WRITE,
        )
    };

    res == 0
}

pub(crate) unsafe fn decommit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    let res = unsafe { libc::mprotect(ptr.as_ptr() as *mut _, size.get(), libc::PROT_NONE) };

    res == 0
}

pub(crate) unsafe fn release(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    let res = unsafe { libc::munmap(ptr.as_ptr() as *mut _, size.get()) };

    res == 0
}

pub(crate) unsafe fn alloc(size: NonZeroUsize) -> Option<NonNull<u8>> {
    let ptr = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            size.get(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED || ptr.is_null() {
        // core::hint::cold_path();
        return None;
    }

    unsafe {
        // Safety: we've checked above that the ptr is not null
        Some(NonNull::new_unchecked(ptr).cast())
    }
}

pub(crate) unsafe fn free(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    let res = unsafe { libc::munmap(ptr.as_ptr() as *mut _, size.get()) };

    res == 0
}
