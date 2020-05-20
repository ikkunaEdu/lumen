use core::alloc::{AllocErr, Layout};
use core::ptr::{self, NonNull};

use crate::alloc::{ReallocPlacement, AllocInit, MemoryBlock, realloc_fallback};
use crate::sys::sysconf::MIN_ALIGN;

#[inline]
pub unsafe fn alloc(layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
    let layout_size = layout.size();
    if layout.align() <= MIN_ALIGN && layout.align() <= layout_size {
        NonNull::new(libc::malloc(layout_size) as *mut u8)
            .ok_or(AllocErr)
            .map(|ptr| MemoryBlock { ptr, size: layout_size })
    } else {
        #[cfg(target_os = "macos")]
        {
            if layout.align() > (1 << 31) {
                return Err(AllocErr);
            }
        }
        aligned_alloc(&layout, init)
    }
}

#[inline]
pub unsafe fn grow(
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
    placement: ReallocPlacement,
    init: AllocInit,
) -> Result<MemoryBlock, AllocErr> {
    // `libc::realloc` can't guarantee that it will be in place
    if placement == ReallocPlacement::InPlace {
        return Err(AllocErr)
    }

    if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
        NonNull::new(libc::realloc(ptr.as_ptr() as *mut libc::c_void, new_size) as *mut u8)
            .ok_or(AllocErr)
            .map(|ptr| {
                let memory_block = MemoryBlock { ptr, size: new_size };

                if init == AllocInit::Zeroed {
                    let old_size = layout.size();
                    let added = new_size - old_size;
                    ptr::write_bytes(ptr.as_ptr().add(old_size), 0, added);
                }

                memory_block
            })
    } else {
        realloc_fallback(ptr, layout, new_size)
    }
}

#[inline]
pub unsafe fn shrink(ptr: NonNull<u8>,
                     layout: Layout,
                     new_size: usize,
                     placement: ReallocPlacement,
) -> Result<MemoryBlock, AllocErr> {
    if placement == ReallocPlacement::InPlace {
        return Err(AllocErr);
    }

    if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
        NonNull::new(libc::realloc(ptr.as_ptr() as *mut libc::c_void, new_size) as *mut u8)
            .ok_or(AllocErr)
            .map(|ptr| MemoryBlock { ptr, size: new_size })
    } else {
        realloc_fallback(ptr, layout, new_size)
    }
}

#[inline]
pub unsafe fn realloc(
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
) -> Result<MemoryBlock, AllocErr> {
    if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
        NonNull::new(libc::realloc(ptr.as_ptr() as *mut libc::c_void, new_size) as *mut u8)
            .ok_or(AllocErr)
            .map(|ptr| MemoryBlock { ptr, size: new_size })
    } else {
        realloc_fallback(ptr, layout, new_size)
    }
}

#[inline]
pub unsafe fn free(ptr: *mut u8, _layout: Layout) {
    libc::free(ptr as *mut libc::c_void)
}

#[inline]
unsafe fn aligned_alloc(layout: &Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
    let mut ptr = ptr::null_mut();
    let layout_size = layout.size();
    let result = libc::posix_memalign(&mut ptr, layout.align(), layout_size);
    if result != 0 {
        return Err(AllocErr);
    }
    let memory_block = MemoryBlock { ptr: NonNull::new_unchecked(ptr as *mut u8), size: layout_size};

    if init == AllocInit::Zeroed {
        ptr::write_bytes(memory_block.ptr.as_ptr(), 0, layout_size);
    }

    Ok(memory_block)
}
