use crate::page_size;



#[derive(Debug, Clone, Copy)]
pub struct PageAllocator {
    /// The size of a page in bytes.
    /// This is a field for cache locality and to support a different page size if needed.
    page_size: usize,
}

impl PageAllocator {
    pub fn new() -> Self {
        Self {
            page_size: page_size::get(),
        }
    }

    pub const fn with_page_size(page_size: usize) -> Self {
        Self { page_size }
    }

    pub const fn page_size(&self) -> usize {
        self.page_size
    }

    /// Allocates a page of memory and returns a pointer to it.
    /// The memory is zero-initialized.
    pub fn alloc_page(&self) -> Option<*mut u8> {
        crate::os::map(self.page_size)
    }

    pub fn alloc_multi_page(&self, num_pages: usize) -> Option<*mut u8> {
        crate::os::map(self.page_size * num_pages)
    }

    pub fn alloc_n_pages(&self, ptrs: &mut [*mut u8]) -> Option<()> {
        let num = ptrs.len();
        let ptr = self.alloc_multi_page(num);
        if let Some(p) = ptr {
            for (i, ptr) in ptrs.iter_mut().enumerate() {
                *ptr = unsafe { p.add(i * self.page_size) };
            }
            Some(())
        } else {
            None
        }
    }

    pub fn alloc_multi_n_pages(&self, ptrs: &mut [*mut u8], num_pages: usize) -> usize {


        let mut allocated = 0;
        for ptr in ptrs.iter_mut() {
            if let Some(p) = self.alloc_multi_page(num_pages) {
                *ptr = p;
                allocated += 1;
            } else {
                break;
            }
        }
        allocated
    }

    pub fn alloc_fixed_pages<const N: usize>(&self) -> Option<[*mut u8; N]> {
        let mut ptrs = [core::ptr::null_mut(); N];
        let allocated = self.alloc_multi_n_pages(&mut ptrs, 1);
        if allocated == N {
            Some(ptrs)
        } else {
            // Deallocate any pages that were allocated before the failure
            for &ptr in ptrs.iter().take(allocated) {
                unsafe { self.dealloc_page(ptr) };
            }
            None
        }
    }

    /// Deallocates a page of memory at the given pointer.
    /// The pointer must have been returned by a previous call to `alloc_page`.
    /// Safety:
    /// - The caller must ensure that the pointer was allocated by this allocator and has not already been deallocated.
    /// - The caller must ensure that the pointer is not used after this call.
    pub unsafe fn dealloc_page(&self, ptr: *mut u8) -> bool {
        unsafe {
            crate::os::unmap(ptr, self.page_size)
        }
    }
}