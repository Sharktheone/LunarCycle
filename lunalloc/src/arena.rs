use core::ptr::NonNull;

use libc::F_FREEZE_FS;

use crate::{bitmap::{Bitmap, BitmapRef}, ospool::{OsPool, PAGE_SIZE}};




pub struct ArenaAlloc<const SIZE: usize> {
    pool: OsPool,
}

impl<const SIZE: usize> ArenaAlloc<SIZE> {
    const _ELEMS_PER_PAGE_NAIVE: usize = PAGE_SIZE / SIZE;
    
    const BITMAP_SIZE: usize = Self::_ELEMS_PER_PAGE_NAIVE.div_ceil(64);
    const BITMAP_BYTES: usize = Self::BITMAP_SIZE * 8;
    const HEADER_BYTES: usize = Self::BITMAP_BYTES * 4;
    
    const ELEMS_PER_PAGE: usize = (PAGE_SIZE-Self::HEADER_BYTES) / SIZE;
    
    const FIRST_PAGE_HEADER_BYTES: usize = 128;
    const FIRST_PAGE_ELEMS: usize = (PAGE_SIZE-(Self::FIRST_PAGE_HEADER_BYTES+Self::HEADER_BYTES)) / SIZE;
    const FIRST_PAGE_ELEMS_LESS: usize = Self::ELEMS_PER_PAGE-Self::FIRST_PAGE_ELEMS;
    
    
    
    fn new() -> Option<Self> {
        Some(Self { pool: OsPool::new()? })
    }
    
    const fn header_ptr(&mut self, page: NonNull<u8>, page_idx: usize) -> NonNull<u8> {
        unsafe {
            page.cast::<u8>()
            .add((page_idx == 0) as usize * Self::FIRST_PAGE_HEADER_BYTES)
        }
    }
    
    const fn page_elements(page_idx: usize) -> usize {
        Self::ELEMS_PER_PAGE - (page_idx == 0) as usize * Self::FIRST_PAGE_ELEMS_LESS
    }
    
    const unsafe fn bitmap_ref<const OFFSET: usize>(&mut self, page: NonNull<u8>, page_idx: usize) -> BitmapRef<'_> {
        let header = self.header_ptr(page, page_idx);
        
        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                header.as_ptr().add(OFFSET*Self::BITMAP_BYTES) as *mut u64,
                Self::BITMAP_SIZE
            )
        };
        
        BitmapRef::new(slice)
    }
    
    const unsafe fn free_bitmap(&mut self, page: NonNull<u8>, page_idx: usize) -> BitmapRef<'_> {
        unsafe {
            self.bitmap_ref::<0>(page, page_idx)
        }
    }
    
    const unsafe fn gc_new_bitmap(&mut self, page: NonNull<u8>, page_idx: usize) -> BitmapRef<'_> {
        unsafe {
            self.bitmap_ref::<1>(page, page_idx)
        }
    }
    
    const unsafe fn gc_marked_bitmap(&mut self, page: NonNull<u8>, page_idx: usize) -> BitmapRef<'_> {
        unsafe {
            self.bitmap_ref::<2>(page, page_idx)
        }
    }
    
    const unsafe fn gc_needs_drop_bitmap(&mut self, page: NonNull<u8>, page_idx: usize) -> BitmapRef<'_> {
        unsafe {
            self.bitmap_ref::<3>(page, page_idx)
        }
    }
    
    fn alloc(&mut self) -> Option<NonNull<u8>> {
        let ((page, page_idx), group) = self.pool.get_next_free_page()?;
        
        let mut free_bitmap = unsafe { self.free_bitmap(page, page_idx) };
        
        let slot = free_bitmap.first_one();
        free_bitmap.set(slot, false);
        
        if slot == Self::page_elements(page_idx)-1 {
            // This page is now full, mark it as such in the pool
            unsafe {
                self.pool.mark_page_full(group, page_idx);
            }
        }
        
        Some(unsafe { page.cast::<u8>().add(slot * SIZE) })
    }
    
    fn free(&mut self, ptr: NonNull<u8>) -> Option<()> {
        //TODO: this is WRONG, we need a different way to calculate the correct slot as the first page has a different layout
        let (group_idx, page_idx, slot) = self.pool.get_page_and_slot(ptr, Self::ELEMS_PER_PAGE)?;
        
        let page = self.pool.page_stripped(group_idx, page_idx)?;
        
        let mut free_bitmap = unsafe { self.free_bitmap(page, page_idx) };
        free_bitmap.set(slot, true);
        
        if free_bitmap.first_one() < Self::page_elements(page_idx) {
            // This page is no longer full, mark it as such in the pool
            unsafe {
                self.pool.mark_page_not_full(group_idx, page_idx);
            }
        }
        
        Some(())
    }
    
}


#[repr(C)]
struct PageHeader<const SIZE: usize> {
    free: Bitmap<SIZE>,
    gc_new: Bitmap<SIZE>,
    gc_marked: Bitmap<SIZE>,
    gc_needs_drop: Bitmap<SIZE>,
}