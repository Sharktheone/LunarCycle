



pub fn map(size: usize) -> Option<*mut u8> {
    unsafe {
        let ptr = libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        );
        if ptr == libc::MAP_FAILED {
            // core::hint::cold_path();
            None
        } else {
            Some(ptr as *mut u8)
        }
    }
}

pub fn unmap(ptr: *mut u8, size: usize) -> bool {
    if ptr.is_null() {
        // core::hint::cold_path();
        return false;
    }

    unsafe { libc::munmap(ptr as *mut _, size) == 0 }
}