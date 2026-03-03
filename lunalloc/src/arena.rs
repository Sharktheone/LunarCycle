use crate::ospool::{OsPool, PAGE_SIZE};




pub struct Alloc<const SIZE: usize> {
    pool: OsPool,
}

impl<const SIZE: usize> Alloc<SIZE> {
    const _ELEMS_PER_PAGE_NAIVE: usize = PAGE_SIZE / SIZE;
    
    const BITMAP_SIZE: usize = Self::_ELEMS_PER_PAGE_NAIVE.div_ceil(64);
    const BITMAP_BYTES: usize = Self::BITMAP_SIZE * 8;
    const HEADER_BYTES: usize = Self::BITMAP_BYTES * 4;
    
    const ELEMS_PER_PAGE: usize = (PAGE_SIZE-Self::HEADER_BYTES) / SIZE;
}