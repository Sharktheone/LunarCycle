#[cfg(target_family = "windows")]
mod windows;
#[cfg(target_family = "unix")]
mod posix;



pub fn map(size: usize) -> Option<*mut u8> {
    #[cfg(target_family = "windows")]
    {
        windows::map(size)
    }
    #[cfg(target_family = "unix")]
    {
        posix::map(size)
    }

    #[cfg(not(any(target_family = "windows", target_family = "unix")))]
    unsafe {
        let ptr = malloc(size);

        if ptr.is_null() {
            // core::hint::cold_path();
            None
        } else {
            Some(ptr as *mut u8)
        }
    }
}

pub fn unmap(ptr: *mut u8, size: usize) -> bool {
    #[cfg(target_family = "windows")]
    {
        windows::unmap(ptr, size)
    }
    #[cfg(target_family = "unix")]
    {
        posix::unmap(ptr, size)
    }

    #[cfg(not(any(target_family = "windows", target_family = "unix")))]
    {
        if ptr.is_null() {
            // core::hint::cold_path();
            return false;
        }

        unsafe { libc::free(ptr as *mut _) }
        true
    }
}