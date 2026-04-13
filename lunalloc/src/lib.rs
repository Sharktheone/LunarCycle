#![no_std]
#[cfg(not(any(windows, unix)))]
extern crate alloc;

mod arena;
mod bitmap;
#[cfg(any(windows, unix))]
pub mod os;
mod ospool;
