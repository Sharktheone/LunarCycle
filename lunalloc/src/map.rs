
#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod posix;



pub fn map(size: usize) -> Option<*mut u8> {
    #[cfg(windows)]
    {
        windows::map(size)
    }
    #[cfg(unix)]
    {
        posix::map(size)
    }

    #[cfg(not(any(windows, unix)))]
    unsafe {
        use alloc::alloc;
        let layout = alloc::Layout::from_size_align(size, 1).ok()?;

        let ptr = alloc::alloc_zeroed(layout);

        if ptr.is_null() {
            // core::hint::cold_path();
            None
        } else {
            Some(ptr)
        }
    }
}

pub unsafe fn unmap(ptr: *mut u8, size: usize) -> bool {
    #[cfg(windows)]
    unsafe {
        windows::unmap(ptr, size)
    }
    #[cfg(unix)]
    unsafe {
        posix::unmap(ptr, size)
    }

    #[cfg(not(any(windows, unix)))]
    {
        if ptr.is_null() {
            // core::hint::cold_path();
            return false;
        }

        unsafe { libc::free(ptr as *mut _) }
        true
    }
}