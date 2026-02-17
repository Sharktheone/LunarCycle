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
}