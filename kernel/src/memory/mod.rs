pub mod frame;
pub mod heap;
pub mod paging;

pub use frame::BootInfoFrameAllocator;
pub use heap::init_heap;
pub use paging::init_paging;
