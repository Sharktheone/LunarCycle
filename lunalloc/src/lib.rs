#![no_std]
#[cfg(not(any(windows, unix)))]
extern crate alloc;

#[cfg(any(windows, unix))]
pub mod os;
mod page;
mod page_size;
