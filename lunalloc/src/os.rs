use core::num::NonZeroUsize;
use core::ptr::NonNull;

#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod posix;




pub(crate) unsafe fn reserve(size: NonZeroUsize) -> Option<NonNull<u8>> {
    todo!()
}

pub(crate) unsafe fn commit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    todo!()
}

pub(crate) unsafe fn decommit(ptr: NonNull<u8>, size: NonZeroUsize) -> bool {
    todo!()
}

pub(crate) unsafe fn release(ptr: NonNull<u8>) -> bool {
    todo!()
}

pub(crate) unsafe fn alloc(size: NonZeroUsize) -> Option<NonNull<u8>> {
    todo!()
}

pub(crate) unsafe fn free(ptr: NonNull<u8>) -> bool {
    todo!()
}