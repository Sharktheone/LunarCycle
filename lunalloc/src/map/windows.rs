use std::ptr::null_mut;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};


pub fn map(size: usize) -> Option<*mut u8> {
    unsafe {
        let ptr = VirtualAlloc(
            null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if ptr.is_null() {
            // core::hint::cold_path();
            None
        } else {
            Some(ptr as *mut u8)
        }
    }
}

pub unsafe fn unmap(ptr: *mut u8, size: usize) -> bool {
    if ptr.is_null() {
        // core::hint::cold_path();
        return false;
    }

    unsafe {
        VirtualFree(ptr as *mut _, 0, winapi::um::winnt::MEM_RELEASE);
    }
}