#![no_std]
#[cfg(not(any(windows, unix)))]
extern crate alloc;

pub mod map;
