use core::{alloc::Layout, cell::RefCell, ptr::NonNull};

use cortex_m::interrupt::Mutex;
use rlsf::Tlsf;

extern crate alloc;
pub struct MyAllocator(Mutex<RefCell<Tlsf<'static, u8, u8, 8, 8>>>);

impl MyAllocator {
    pub const INIT: MyAllocator = MyAllocator(Mutex::new(RefCell::new(Tlsf::INIT)));

    pub unsafe fn init(&self, pool: *mut u8, len: usize) -> usize {
        cortex_m::interrupt::free(|cs| {
            let mut tlsf = self.0.borrow(cs).borrow_mut();

            len - tlsf.append_free_block_ptr(
                NonNull::new(core::ptr::slice_from_raw_parts_mut(pool, len))
                    .expect("a null pointer was supplied"),
            )
        })
    }
}

unsafe impl alloc::alloc::GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        cortex_m::interrupt::free(|cs| {
            let mut tlsf = self.0.borrow(cs).borrow_mut();
            tlsf.allocate(layout)
                .map_or(core::ptr::null_mut(), |p| p.as_ptr())
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr);
        if ptr.is_none() {
            return;
        }
        cortex_m::interrupt::free(|cs| {
            let mut tlsf = self.0.borrow(cs).borrow_mut();
            tlsf.deallocate(ptr.unwrap(), layout.align())
        })
    }
}
