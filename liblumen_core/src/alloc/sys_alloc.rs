use core::cmp;
use core::ptr::{self, NonNull};

use crate::alloc::alloc_handle::{AsAllocHandle, Global};
use crate::alloc::{
    AllocErr, AllocInit, AllocRef, GlobalAlloc, Layout, MemoryBlock, ReallocPlacement,
};
use crate::sys::alloc as sys_alloc;

use super::StaticAlloc;

/// This allocator acts as the system allocator, depending
/// on the target, that may be the actual system allocator,
/// or our own implementation.
#[derive(Debug, Copy, Clone)]
pub struct SysAlloc;

unsafe impl AllocRef for SysAlloc {
    #[inline]
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        unsafe { sys_alloc::alloc(layout, init) }
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        sys_alloc::free(ptr.as_ptr(), layout);
    }

    #[inline]
    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        sys_alloc::grow(ptr, layout, new_size, placement, init)
    }

    #[inline]
    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        sys_alloc::shrink(ptr, layout, new_size, placement)
    }
}

unsafe impl GlobalAlloc for SysAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        sys_alloc::alloc(layout, AllocInit::Uninitialized)
            .map(|memory_block| memory_block.ptr.as_ptr())
            .unwrap_or(ptr::null_mut())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        sys_alloc::alloc(layout, AllocInit::Zeroed)
            .map(|memory_block| memory_block.ptr.as_ptr())
            .unwrap_or(ptr::null_mut())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        sys_alloc::free(ptr, layout);
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        sys_alloc::realloc(NonNull::new(ptr).unwrap(), layout, new_size)
            .map(|memory_block| memory_block.ptr.as_ptr())
            .unwrap_or(ptr::null_mut())
    }
}

// Used by the StaticAlloc impl
static mut SYS_ALLOC: SysAlloc = SysAlloc;

unsafe impl StaticAlloc for SysAlloc {
    #[inline]
    unsafe fn static_ref() -> &'static Self {
        &SYS_ALLOC
    }
    #[inline]
    unsafe fn static_mut() -> &'static mut Self {
        &mut SYS_ALLOC
    }
}

impl AsAllocHandle<'static> for SysAlloc {
    type Handle = Global<Self>;

    #[inline]
    fn as_alloc_handle(&'static self) -> Self::Handle {
        Global::new()
    }
}

/// Fallback for realloc that allocates a new region, copies old data
/// into the new region, and frees the old region.
#[inline]
pub unsafe fn realloc_fallback(
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_size: usize,
) -> Result<MemoryBlock, AllocErr> {
    use core::intrinsics::unlikely;

    let old_size = old_layout.size();

    if unlikely(old_size == new_size) {
        return Ok(MemoryBlock {
            ptr,
            size: new_size,
        });
    }

    let align = old_layout.align();
    let new_layout = Layout::from_size_align(new_size, align).expect("invalid layout");

    // Allocate new region, using mmap for allocations larger than page size
    let memory_block = sys_alloc::alloc(new_layout, AllocInit::Uninitialized)?;
    // Copy old region to new region
    ptr::copy_nonoverlapping(
        ptr.as_ptr(),
        memory_block.ptr.as_ptr(),
        cmp::min(old_size, new_size),
    );
    // Free old region
    sys_alloc::free(ptr.as_ptr(), old_layout);

    Ok(memory_block)
}
