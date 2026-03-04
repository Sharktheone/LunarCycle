use core::{num::NonZeroUsize, ptr::NonNull};

use crate::{bitmap::Bitmap, os};

const POOL_SIZE: usize = 4 * 1024usize.pow(3);
pub const PAGE_SIZE: usize = 16384;
const NUM_PAGES: usize = POOL_SIZE / PAGE_SIZE;
const NUM_GROUPS: usize = 512;
const GROUP_SIZE: usize = NUM_PAGES / NUM_GROUPS;

const GROUPS_BITMAP_SIZE: usize = NUM_GROUPS.div_ceil(64);
const GROUP_BITMAP_SIZE: usize = GROUP_SIZE.div_ceil(64);

const DEFAULT_COMMIT_PAGES: usize = 16;

type Memory = [PageGroup; NUM_GROUPS];


pub struct OsPool {
    ptr: NonNull<Memory>,
    
    free: Bitmap<GROUPS_BITMAP_SIZE>,
    allocated: Bitmap<GROUPS_BITMAP_SIZE>,
    
    next: Option<NonNull<Self>>
}

impl OsPool {
    pub fn new() -> Option<Self> {
        let size = NonZeroUsize::new(POOL_SIZE)?;
        
        let ptr = unsafe { os::reserve(size) }?;
        
        Some(Self {
            ptr: ptr.cast(),
            free: Bitmap::all(),
            allocated: Bitmap::new(),
            next: None,
        })
    }
    
    pub fn is_empty(&self) -> bool {
        self.free.first_zero() >= NUM_GROUPS
    }
    
    pub fn has_allocated(&self) -> bool {
        self.allocated.first_zero() >= NUM_GROUPS
    }
    
    pub fn group(&self, group: usize) -> Option<NonNull<PageGroup>> {
        if group >= NUM_GROUPS {
            // core::hint::cold_path();
            if let Some(next) = self.next {
                return unsafe { next.as_ref().group(group - NUM_GROUPS) };
            } else {
                // core::hint::cold_path();
                return None;
            }
        }
        
        Some(unsafe { self.ptr.cast::<PageGroup>().add(group) })
    }
    
    pub fn page(&self, group: usize, page: NonZeroUsize) -> Option<NonNull<Page>> {
        Some(self.page_stripped(group, page.get())?.cast::<Page>())
    }
    
    pub fn page_stripped(&self, group: usize, page: usize) -> Option<NonNull<u8>> {
        if page >= GROUP_SIZE {
            // core::hint::cold_path();
            return None;
        }
        
        let page_ptr = self.group(group)?.cast::<Page>();
        
        
        Some(unsafe { page_ptr.add(page).cast::<u8>() })
    }
    
    pub fn get_next_free_group(&mut self) -> Option<(NonNull<PageGroup>, usize)> {
        let group = self.free.first_zero();
        if group >= NUM_GROUPS {
            // core::hint::cold_path();
            if let Some(mut next) = self.next {
                return unsafe { next.as_mut().get_next_free_group() };
            } else {
                // core::hint::cold_path();
                return None;
            }
        }
        
        if !self.allocated.get(group) {
            // Safety: We've checked that the group is within bounds, and there are no other references to it
            // also the group is not allocated, so it's safe to commit it.
            unsafe { 
                self.commit_group(group, DEFAULT_COMMIT_PAGES)?
            };
        }
        
        Some((unsafe { self.ptr.cast::<PageGroup>().add(group) }, group))
    }
    
    pub fn get_next_free_page_on_group(&mut self, group_idx: usize) -> Option<(NonNull<u8>, usize)> {
        let group_ptr = self.group(group_idx)?;
        let group = unsafe { group_ptr.as_ref() };
        
        let page = group.header.free.first_zero();
        
        if page >= GROUP_SIZE {
            // core::hint::cold_path();
            return None;
        }
        
        if !group.header.allocated.get(page) {
            // Safety: We've checked that the page is within bounds, and there are no other references to it
            // also the page is not allocated, so it's safe to commit it.
            unsafe { 
                self.commit_page(group_idx, NonZeroUsize::new(page)?)
            }?;
        }
        
        Some((unsafe { group_ptr.cast::<Page>().add(page).cast::<u8>() }, page))
    }
    
    pub fn get_next_free_page(&mut self) -> Option<((NonNull<u8>, usize), usize)> {
        let (_, group) = self.get_next_free_group()?;
        
        Some((self.get_next_free_page_on_group(group)?, group))
    }
    
    pub fn get_page_and_slot(&self, ptr: NonNull<u8>, nslots: usize) -> Option<(usize, usize, usize)> {
        let base_ptr = self.ptr.cast::<u8>();
        let offset = ptr.as_ptr() as usize - base_ptr.as_ptr() as usize;
        
        if offset >= POOL_SIZE {
            // core::hint::cold_path();
            return None;
        }
        
        //TODO: check the maths is correct here
        let group = offset / (GROUP_SIZE * PAGE_SIZE);
        let page_offset = offset % (GROUP_SIZE * PAGE_SIZE);
        let page = page_offset / PAGE_SIZE;
        let slot = page_offset % PAGE_SIZE / (PAGE_SIZE / nslots); // Assuming 64 slots per page
        
        Some((group, page, slot))
    }
    
    pub unsafe fn mark_page_full(&mut self, group_idx: usize, page: usize) -> Option<()> {
        let mut group_ptr = self.group(group_idx)?;
        // Safety: the ptr is aligned and non-null, plus there are no other references.
        let group = unsafe { group_ptr.as_mut() };
        
        if page >= GROUP_SIZE {
            // core::hint::cold_path();
            return None;
        }
        
        group.header.free.set(page, false);
        
        if group.header.free.all_clear() {
            self.free.set(group_idx, false);
        }
        
        Some(())
    }
   
   pub unsafe fn mark_page_not_full(&mut self, group_idx: usize, page: usize) -> Option<()> {
        let mut group_ptr = self.group(group_idx)?;
        // Safety: the ptr is aligned and non-null, plus there are no other references.
        let group = unsafe { group_ptr.as_mut() };
        
        if page >= GROUP_SIZE {
            // core::hint::cold_path();
            return None;
        }
        
        group.header.free.set(page, true);
        self.free.set(group_idx, true);
        
        Some(())
    } 
    
    pub unsafe fn commit_group(&mut self, group: usize, pages: usize) -> Option<()> {
        let commit_size = pages.max(1) * PAGE_SIZE;
        
        let mut group_ptr = self.group(group)?;
        
        
        
        let success = unsafe { 
            // Safety: commit_size is guaranteed to be non-zero
            let commit_size = NonZeroUsize::new_unchecked(commit_size);
            
            os::commit(group_ptr.cast(), commit_size)
        };
        
        // Safety: the ptr is aligned and non-null, plus there are no other references.
        let pg = unsafe { group_ptr.as_mut() };
        pg.header.free.set_all();
        pg.header.allocated.set_bits(0, pages, true);
        
        
        
        self.allocated.set(group, true);
        
        if !success {
            // core::hint::cold_path();
            return None;
        }
        
        
        
        Some(())
    }
    
    pub unsafe fn decommit_group(&mut self, group: usize) -> Option<()> {
        let group_ptr = self.group(group)?;
        
        let size = NonZeroUsize::new(size_of::<PageGroup>())?;
        
        let success = unsafe { os::decommit(group_ptr.cast(), size) };
        
        if !success {
            return None;
        }
        
        self.allocated.set(group, false);
        
        
        Some(())
    }
    
    
    pub unsafe fn commit_page(&mut self, group: usize, page: NonZeroUsize) -> Option<()> {
        let mut group_ptr = self.group(group)?;
        let page_ptr = self.page(group, page)?;
        
        let size = NonZeroUsize::new(PAGE_SIZE)?;
        
        let success = unsafe { os::commit(page_ptr.cast(), size) };
        
        if !success {
            // core::hint::cold_path();
            return None;
        }
        
        // Safety: the ptr is aligned and non-null, plus there are no other references.
        let header = unsafe { &mut group_ptr.as_mut().header };
        header.free.set(page.get(), true);
        header.allocated.set(page.get(), true);
        
        
        Some(())
    }
    
    pub unsafe fn commit_pages(&mut self, group: usize, page: NonZeroUsize, count: NonZeroUsize) -> Option<()> {
        let mut group_ptr = self.group(group)?;
        let page_ptr = self.page(group, page)?;
        
        let size = NonZeroUsize::new(count.get() * PAGE_SIZE)?;
        
        let success = unsafe { os::commit(page_ptr.cast(), size) };
        
        if !success {
            // core::hint::cold_path();
            return None;
        }
        
        // Safety: the ptr is aligned and non-null, plus there are no other references.
        let header = unsafe { &mut group_ptr.as_mut().header };
        header.free.set_bits(page.get(), count.get(), true);
        header.allocated.set_bits(page.get(), count.get(), true);
        
        Some(())
    }
    
    pub unsafe fn decommit_page(&mut self, group: usize, page: NonZeroUsize) -> Option<()> {
        let page_ptr = self.page(group, page)?;
        
        let size = NonZeroUsize::new(PAGE_SIZE)?;
        
        let success = unsafe { os::decommit(page_ptr.cast(), size) };
        
        if !success {
            // core::hint::cold_path();
            return None;
        }
        
        // Safety: the ptr is aligned and non-null, plus there are no other references.
        let header = unsafe { &mut self.group(group)?.as_mut().header };
        header.free.set(page.get(), true);
        header.allocated.set(page.get(), false);
        
        Some(())
    }
    
    pub unsafe fn decommit_pages(&mut self, group: usize, page: NonZeroUsize, count: NonZeroUsize) -> Option<()> {
        let page_ptr = self.page(group, page)?;
        
        let size = NonZeroUsize::new(count.get() * PAGE_SIZE)?;
        
        let success = unsafe { os::decommit(page_ptr.cast(), size) };
        
        if !success {
            // core::hint::cold_path();
            return None;
        }
        
        // Safety: the ptr is aligned and non-null, plus there are no other references.
        let header = unsafe { &mut self.group(group)?.as_mut().header };
        header.free.set_bits(page.get(), count.get(), true);
        header.allocated.set_bits(page.get(), count.get(), false);
        
        Some(())
    }
    
    pub fn last_pool(&mut self) -> &mut Self {
        let mut current = self;
        
        while let Some(mut next) = current.next {
            current = unsafe { next.as_mut() };
        }
        
        current
    }
    
    pub unsafe fn release_all(&mut self) -> Option<()> {
        #[cfg(unix)]
        let size = NonZeroUsize::new(POOL_SIZE)?;
        
        let success = unsafe { os::release(self.ptr.cast(), #[cfg(unix)] size) };
        
        if !success {
            // core::hint::cold_path();
            return None;
        }
        
        Some(())
    }
    
    pub fn extend(&mut self, alloc: &mut impl ExtendAlloc) -> Option<NonNull<Self>> {
        let last = self.last_pool();
        let new_pool = Self::new()?;
        
        let ptr = alloc.alloc(new_pool)?;
        last.next = Some(ptr); 
        
        Some(ptr)
    }
    
    pub fn shrink(&mut self, alloc: &mut impl ExtendAlloc) -> Option<()> {
        let mut ptr = self;
        
        while let Some(mut next) = ptr.next {
            let next = unsafe { next.as_mut() };
            
            if next.is_empty() {
                unsafe { next.release_all() }?;
                
                let next_next = next.next;
                ptr.next = next_next;
                alloc.free(next.into())?;
            } else {
                ptr = next;
            }
            
        }
        
        Some(())
    }
    
}

pub trait ExtendAlloc {
    fn alloc(&mut self, extend: OsPool) -> Option<NonNull<OsPool>>;
    fn free(&mut self, ptr: NonNull<OsPool>) -> Option<()>;
}

#[repr(C)]
pub struct PageGroup {
    header: GroupHeader,
    first: FirstPage,
    pages: [Page; GROUP_SIZE-1],
    
}

const HEADER_SIZE: usize = size_of::<GroupHeader>();

pub struct GroupHeader {
    free: Bitmap<GROUP_BITMAP_SIZE>,
    allocated: Bitmap<GROUP_BITMAP_SIZE>,
}

pub struct Page {
    pub data: [u8; PAGE_SIZE]
}

pub struct FirstPage {
    pub data: [u8; PAGE_SIZE-HEADER_SIZE]
}


const _ASSERTIONS: () = const {
    assert!(size_of::<Page>() == PAGE_SIZE);
    assert!(size_of::<FirstPage>() + size_of::<GroupHeader>() == size_of::<Page>());
    assert!(size_of::<PageGroup>() == POOL_SIZE / NUM_GROUPS);
    assert!(size_of::<Memory>() == POOL_SIZE);
};