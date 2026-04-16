use core::num::NonZeroUsize;
use core::ptr::NonNull;

use crate::ospool::{FirstPage, Page, PageCommitCB};
use crate::{
    bitmap::{Bitmap, BitmapRef},
    ospool::{OsPool, HEADER_SIZE, PAGE_SIZE},
};

pub struct ArenaAlloc<const SIZE: usize> {
    pool: OsPool,
}

impl<const SIZE: usize> ArenaAlloc<SIZE> {
    const _ELEMS_PER_PAGE_NAIVE: usize = PAGE_SIZE / SIZE;

    const BITMAP_SIZE: usize = Self::_ELEMS_PER_PAGE_NAIVE.div_ceil(64);
    const BITMAP_BYTES: usize = Self::BITMAP_SIZE * 8;
    const HEADER_BYTES: usize = Self::BITMAP_BYTES * 4;
    const PAGE_DATA_OFFSET: usize = Self::align_up(Self::HEADER_BYTES, SIZE);
    const ELEMS_PER_PAGE: usize = (PAGE_SIZE - Self::PAGE_DATA_OFFSET) / SIZE;

    const FIRST_PAGE_HEADER_BYTES: usize = HEADER_SIZE;
    const FIRST_PAGE_DATA_OFFSET: usize =
        Self::align_up (Self::FIRST_PAGE_HEADER_BYTES + Self::HEADER_BYTES, SIZE);
    const FIRST_PAGE_ELEMS: usize = (PAGE_SIZE - Self::FIRST_PAGE_DATA_OFFSET) / SIZE;
    const FIRST_PAGE_ELEMS_LESS: usize = Self::ELEMS_PER_PAGE - Self::FIRST_PAGE_ELEMS;

    pub fn new() -> Option<Self> {
        Some(Self {
            pool: OsPool::new()?,
        })
    }

    pub const unsafe fn from_pool(pool: OsPool) -> Self {
        Self { pool }
    }

    pub fn new_multiple<const N: usize>() -> Option<[Self; N]> {
        Some(OsPool::new_multiple::<N>()?.map(|p| unsafe { Self::from_pool(p) }))
    }



    pub const fn element_size() -> usize {
        SIZE
    }

    const fn align_up(value: usize, align: usize) -> usize {
        let rem = value % align;
        if rem == 0 {
            value
        } else {
            value + (align - rem)
        }
    }
    const fn header_ptr(page: NonNull<u8>, page_idx: usize) -> NonNull<u8> {
        unsafe {
            page.cast::<u8>()
                .add((page_idx == 0) as usize * Self::FIRST_PAGE_HEADER_BYTES)
        }
    }

    const fn first_page_header_ptr(page: NonNull<u8>) -> NonNull<u8> {
        unsafe { page.cast::<u8>().add(Self::FIRST_PAGE_HEADER_BYTES) }
    }

    const fn page_elements(page_idx: usize) -> usize {
        // non-branching way to return either FIRST_PAGE_ELEMS or ELEMS_PER_PAGE
        Self::ELEMS_PER_PAGE - (page_idx == 0) as usize * Self::FIRST_PAGE_ELEMS_LESS
    }
    const fn data_offset(page_idx: usize) -> usize {
        // non-branching seems to compile to the exact same asm, so:
        if page_idx == 0 {
            Self::FIRST_PAGE_DATA_OFFSET
        } else {
            Self::PAGE_DATA_OFFSET
        }
    }

    const fn data_ptr(page: NonNull<u8>, page_idx: usize) -> NonNull<u8> {
        unsafe { page.add(Self::data_offset(page_idx)) }
    }

    const unsafe fn bitmap_ref<const OFFSET: usize>(
        &mut self,
        page: NonNull<u8>,
        page_idx: usize,
    ) -> BitmapRef {
        let header = Self::header_ptr(page, page_idx);

        unsafe { Self::bitmap_ref_header::<OFFSET>(header) }
    }

    const unsafe fn bitmap_ref_header<'a, const OFFSET: usize>(
        header: NonNull<u8>,
    ) -> BitmapRef<'a> {
        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                header.as_ptr().add(OFFSET * Self::BITMAP_BYTES) as *mut u64,
                Self::BITMAP_SIZE,
            )
        };

        BitmapRef::new(slice)
    }

    const unsafe fn free_bitmap(&mut self, page: NonNull<u8>, page_idx: usize) -> BitmapRef<'_> {
        unsafe { self.bitmap_ref::<0>(page, page_idx) }
    }

    const unsafe fn gc_new_bitmap(&mut self, page: NonNull<u8>, page_idx: usize) -> BitmapRef<'_> {
        unsafe { self.bitmap_ref::<1>(page, page_idx) }
    }

    const unsafe fn gc_marked_bitmap(
        &mut self,
        page: NonNull<u8>,
        page_idx: usize,
    ) -> BitmapRef<'_> {
        unsafe { self.bitmap_ref::<2>(page, page_idx) }
    }

    const unsafe fn gc_needs_drop_bitmap(
        &mut self,
        page: NonNull<u8>,
        page_idx: usize,
    ) -> BitmapRef<'_> {
        unsafe { self.bitmap_ref::<3>(page, page_idx) }
    }

    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        let ((page, page_idx), group) = self.pool.get_next_free_page::<PageCommitter<SIZE>>()?;

        let mut free_bitmap = unsafe { self.free_bitmap(page, page_idx) };

        let slot = free_bitmap.first_one();

        #[cfg(debug_assertions)]
        if slot >= Self::page_elements(page_idx) {
            panic!("Pool gave us a page with no free slots");
        }

        free_bitmap.set(slot, false);

        if free_bitmap.first_one() >= Self::page_elements(page_idx)  {
            // This page is now full, mark it as such in the pool
            unsafe {
                self.pool.mark_page_full(group, page_idx);
            }
        }

        Some(unsafe { Self::data_ptr(page, page_idx).add(slot * SIZE) })
    }

    pub fn free(&mut self, ptr: NonNull<u8>) -> Option<()> {
        let (group_idx, page_idx, page_offset) = self.pool.get_page_offset(ptr)?;
        let data_offset = Self::data_offset(page_idx);
        let page_elements = Self::page_elements(page_idx);

        if page_offset < data_offset {
            return None; // we could say that this is an invalid input and just not check for it and make it the callers fault!
        }

        let slot_offset = page_offset - data_offset;
        if slot_offset % SIZE != 0 {
            return None; // we could say that this is an invalid input and just not check for it and make it the callers fault!
        }

        let slot = slot_offset / SIZE;
        if slot >= page_elements {
            return None; // we could say that this is an invalid input and just not check for it and make it the callers fault!
        }
        let page = self.pool.page_stripped(group_idx, page_idx)?;

        let mut free_bitmap = unsafe { self.free_bitmap(page, page_idx) };

        if free_bitmap.get(slot) {
            return None; // This slot is already free, so this is either a double free or an invalid pointer
        }

        free_bitmap.set(slot, true);

        let free_slots = free_bitmap.first_one();

        if free_slots < page_elements {
            // This page is no longer full, mark it as such in the pool
            unsafe {
                self.pool.mark_page_not_full(group_idx, page_idx);
            }
        }

        if free_slots >= page_elements {
            //TODO: we should group operations like that
            if let Some(page_idx) = NonZeroUsize::new(page_idx) {
                // This page is now completely free, we can return it to the pool
                unsafe {
                    self.pool.decommit_page(group_idx, page_idx);
                }
            }

            //TODO: check if the full group is now free and decommit it if so
        }

        Some(())
    }
}

struct PageCommitter<const SIZE: usize>;

impl<const SIZE: usize> PageCommitCB for PageCommitter<SIZE> {
    fn commit_page(page: NonNull<Page>) -> Option<()> {
        let mut free_bitmap = unsafe { ArenaAlloc::<SIZE>::bitmap_ref_header::<0>(page.cast()) };

        let num_elements = ArenaAlloc::<SIZE>::ELEMS_PER_PAGE;

        free_bitmap.set_bits(0, num_elements, true);

        Some(())
    }

    fn commit_pages(page: NonNull<Page>, count: NonZeroUsize) -> Option<()> {
        for i in 0..count.get() {
            let page = unsafe { page.add(i) };
            Self::commit_page(page)?;
        }

        Some(())
    }

    fn commit_first_page(page: NonNull<FirstPage>) -> Option<()> {
        let header = ArenaAlloc::<SIZE>::first_page_header_ptr(page.cast());
        let mut free_bitmap = unsafe { ArenaAlloc::<SIZE>::bitmap_ref_header::<0>(header) };

        let num_elements = ArenaAlloc::<SIZE>::FIRST_PAGE_ELEMS;

        free_bitmap.set_bits(0, num_elements, true);

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
