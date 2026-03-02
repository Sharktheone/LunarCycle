use core::num::NonZeroUsize;
use core::ptr::NonNull;

#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod posix;




pub(crate) unsafe fn reserve(size: NonZeroUsize) -> Option<NonNull<u8>> {
    #[cfg(windows)]
    unsafe {
        windows::reserve(size)
    }
    #[cfg(unix)]
    unsafe {
        posix::reserve(size)
    }
}

pub(crate) unsafe fn commit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    #[cfg(windows)]
    unsafe {
        windows::commit(ptr, size)
    }
    #[cfg(unix)]
    unsafe {
        posix::commit(ptr, size)
    }
}

pub(crate) unsafe fn decommit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    #[cfg(windows)]
    unsafe {
        windows::decommit(ptr, size)
    }
    #[cfg(unix)]
    unsafe {
        posix::decommit(ptr, size)
    }
}
pub(crate) unsafe fn release(ptr: NonNull<u8>, #[cfg(unix)] size: NonZeroUsize) -> bool {
    #[cfg(windows)]
    unsafe {
        windows::release(ptr)
    }
    #[cfg(unix)]
    unsafe {
        posix::release(ptr, size)
    }
}

pub(crate) unsafe fn alloc(size: NonZeroUsize) -> Option<NonNull<u8>> {
    #[cfg(windows)]
    unsafe {
        windows::alloc(size)
    }
    #[cfg(unix)]
    unsafe {
        posix::alloc(size)
    }
}

pub(crate) unsafe fn free(ptr: NonNull<u8>, #[cfg(unix)] size: NonZeroUsize) -> bool {
    #[cfg(windows)]
    unsafe {
        windows::free(ptr)
    }
    #[cfg(unix)]
    unsafe {
        posix::free(ptr, size)
    }
}