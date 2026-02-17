pub const FALLBACK_PAGE_SIZE: usize = 4096;
static mut PAGE_SIZE: usize = 0;

pub fn get() -> usize {
    unsafe {
        if PAGE_SIZE == 0 {
            // core::hint::cold_path();
            PAGE_SIZE = platform::get_page_size();
        }
        PAGE_SIZE
    }
}

pub mod platform {
    #[cfg(unix)]
    pub use super::unix::get_page_size;

    #[cfg(windows)]
    pub use super::windows::get_page_size;

    #[cfg(not(any(windows, unix)))]
    pub fn get_page_size() -> usize {
        super::FALLBACK_PAGE_SIZE
    }
}


#[cfg(unix)]
pub mod unix {
    #[cfg(unix)]
    #[cold]
    pub fn get_page_size() -> usize {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }
}

#[cfg(windows)]
pub mod windows {
    #[cold]
    pub fn get_page_size() -> usize {
        use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};

        unsafe {
            let mut sys_info: SYSTEM_INFO = core::mem::zeroed();
            GetSystemInfo(&raw mut sys_info);
            sys_info.dwPageSize as usize
        }
    }
}