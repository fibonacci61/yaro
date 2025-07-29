#![no_std]
#![no_main]
#![allow(clippy::unusual_byte_groupings)]

mod asm;
mod boot;
mod int;
mod io;
mod mem;
mod sbi;
mod sched;

extern crate alloc;

use core::panic::PanicInfo;

use talc::Span;

use crate::{
    boot::{PHEAP_LEN, PHYS_PHEAP},
    io::serial::print,
    mem::alloc::{Block, HEAP_ALLOCATOR, PAGE_ALLOCATOR},
};

fn shutdown() -> ! {
    const SHUTDOWN_EID: usize = 0x53525354;
    let _ = unsafe { sbi::SbiCall::default().with_eid(SHUTDOWN_EID).call() };

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("Kernel panic in ");

    if let Some(location) = info.location() {
        print!("[{}:{}]: ", location.file(), location.line());
    } else {
        print!("[Location unavailable]: ");
    }

    print!("{}", info.message());

    shutdown();
}

unsafe extern "C" {
    static mut _sbss: usize;
    static mut _ebss: usize;
}

unsafe fn zero_bss() {
    let mut ptr = &raw mut _sbss;
    while ptr < &raw mut _ebss {
        unsafe {
            ptr.write(0);
            ptr = ptr.add(1)
        }
    }
}

unsafe extern "C" fn kmain(_hart_id: usize, _dtb_addr: usize) -> ! {
    crate::io::serial::println!("Hello World!");

    unsafe {
        zero_bss();
        int::set_kernel_entry();

        let mut page_alloc = PAGE_ALLOCATOR.lock();

        let pheap_block = Block::from_range(PHYS_PHEAP, PHYS_PHEAP + PHEAP_LEN).unwrap();
        page_alloc.claim(pheap_block);

        const HEAP_ORDER: usize = 3;
        let heap_allocation = page_alloc.alloc(HEAP_ORDER).unwrap();
        HEAP_ALLOCATOR
            .lock()
            .claim(Span::new(
                heap_allocation.start().as_ptr(),
                heap_allocation.end().as_ptr(),
            ))
            .unwrap();
    }

    // Kernel heap test
    let mut vec = alloc::vec::Vec::new();
    for i in 0..100 {
        vec.push(i);
    }

    crate::io::serial::println!("{vec:?}");

    shutdown();
}
