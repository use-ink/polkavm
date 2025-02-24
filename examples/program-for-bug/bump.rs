//! A simple bump allocator.

use core::alloc::{
    GlobalAlloc,
    Layout,
};

static mut INNER: Option<InnerAlloc> = None;

static mut RISCV_HEAP: [u8; 1024 * 1024] = [0; 1024 * 1024];

/// A bump allocator suitable for use in a Wasm environment.
pub struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    #[inline]
    #[allow(static_mut_refs)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if INNER.is_none() {
            INNER = Some(InnerAlloc::new());
        };
        match INNER
            .as_mut()
            .expect("We just set the value above; qed")
            .alloc(layout)
        {
            Some(start) => start as *mut u8,
            None => core::ptr::null_mut(),
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg_attr(feature = "std", derive(Debug, Copy, Clone))]
struct InnerAlloc {
    /// Points to the start of the next available allocation.
    next: usize,

    /// The address of the upper limit of our heap.
    upper_limit: usize,
}

impl InnerAlloc {
    fn new() -> Self {
        Self {
            next: Self::heap_start(),
            upper_limit: Self::heap_end(),
        }
    }

    fn heap_start() -> usize {
        #[allow(static_mut_refs)]
        unsafe {
            RISCV_HEAP.as_mut_ptr() as usize
        }
    }

    #[allow(static_mut_refs)]
    fn heap_end() -> usize {
        Self::heap_start() + unsafe { RISCV_HEAP.len() }
    }

    #[allow(dead_code)]
    fn request_pages(&mut self, _pages: usize) -> Option<usize> {
        core::panic!("no request possible");
    }

    /// Tries to allocate enough memory on the heap for the given `Layout`. If there is
    /// not enough room on the heap it'll try and grow it by a page.
    ///
    /// Note: This implementation results in internal fragmentation when allocating across
    /// pages.
    fn alloc(&mut self, layout: Layout) -> Option<usize> {
        let alloc_start = self.align_ptr(&layout);

        let aligned_size = layout.size();

        let alloc_end = alloc_start.checked_add(aligned_size)?;

        if alloc_end > self.upper_limit {
            panic!("exhausted heap limit");
        } else {
            self.next = alloc_end;
            Some(alloc_start)
        }
    }

    /// Aligns the start pointer of the next allocation.
    ///
    /// We inductively calculate the start index
    /// of a layout in the linear memory.
    /// - Initially `self.next` is `0`` and aligned
    /// - `layout.align() - 1` accounts for `0` as the first index.
    /// - the binary with the inverse of the align creates a bitmask that is used to zero
    ///   out bits, ensuring alignment according to type requirements and ensures that the
    ///   next allocated pointer address is of the power of 2.
    fn align_ptr(&self, layout: &Layout) -> usize {
        (self.next + layout.align() - 1) & !(layout.align() - 1)
    }
}
