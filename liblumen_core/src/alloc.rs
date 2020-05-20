pub mod alloc_handle;
pub mod boxed;
pub mod mmap;
pub mod raw_vec;
mod region;
pub mod size_classes;
mod static_alloc;
mod sys_alloc;
pub mod utils;
pub mod vec;

pub use self::region::Region;
pub use self::static_alloc::StaticAlloc;
pub use self::sys_alloc::*;

// Re-export core alloc types
pub use core::alloc::{
    AllocErr, AllocInit, AllocRef, GlobalAlloc, Layout, LayoutErr, MemoryBlock, ReallocPlacement,
};
