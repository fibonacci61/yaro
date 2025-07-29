#![no_std]
#![no_main]
#![allow(clippy::unusual_byte_groupings)]

mod asm;
mod int;
mod io;
mod mem;
mod sbi;
mod sched;

use core::{arch::naked_asm, panic::PanicInfo};

use crate::{
    io::serial::print,
    mem::{
        addr::{PhysAddr, VirtAddr},
        alloc::BiBuddy,
        paging::table::{ENTRY_COUNT, RawTable},
    },
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

const PHYS_RAM_START: PhysAddr = PhysAddr(0x80000000);
const PHYS_STACK: PhysAddr = PhysAddr(0x83000000);
const STACK_TOP: VirtAddr = VirtAddr::new(0xffffffffc0000000);

#[unsafe(link_section = ".boot.data")]
static STACK_PT: RawTable = {
    let mut table = [0; ENTRY_COUNT];
    // valid + rw + non-user + global + accessed + dirty
    let pte = (PHYS_STACK.ppn() << 10) | 0xe7;
    table[511] = pte;
    RawTable(table)
};

const KERNEL_PTE: usize = {
    // valid + rwx + non-user + global + accessed + dirty
    let pte = (PHYS_RAM_START.ppn() << 10) | 0xef;
    pte
};

// This is the primary kernel page table.
// The kernel resides at 0x82000000 in physical memory, part of the gigapage [0x80000000, 0xc0000000).
// The last GiB of virtual memory is mapped to this gigapage, so that 0xffffffffc2000000 corresponds to
// 0x82000000. Also, it's identity mapped, so that a page fault doesn't happen right after enabling
// paging.
//
// We can't add a PTE for STACK_PT at compile time, so that needs to be done at runtime in `_boot`.
#[unsafe(link_section = ".boot.data")]
static mut KERNEL_PT: RawTable = {
    let mut table = [0; ENTRY_COUNT];

    // identity map 0x8000000
    table[VirtAddr::new(PHYS_RAM_START.as_usize()).vpn2()] = KERNEL_PTE;

    // map high memory (last virtual GiB + 2 MiB)
    table[511] = KERNEL_PTE;
    RawTable(table)
};

#[unsafe(link_section = ".boot.start")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _boot() -> ! {
    #[allow(unused_unsafe)]
    unsafe {
        naked_asm!(
            // t0 = KERNEL_PT
            "la t0, {kpt}",

            // t1 is our pte register
            // t1 = &STACK_PT
            "la t1, {spt}",

            // spte = (spt >> 12) & 0xfffffffffff << 10 | 0xe1
            // t1 >> 12
            "srli t1, t1, 12",
            // t1 & 0xfffffffffff
            "li t2, 0xfffffffffff",
            "and t1, t1, t2",
            // t1 << 10
            "slli t1, t1, 10",
            // valid + no perms + non-user + global
            // NOTE: non-leaf PTEs mustn't have the Dirty or Accessed bits set.
            "ori t1, t1, 0x21",

            // t0[510] = t1
            "li t2, 4080",
            // t2 = t0 + 4080
            "add t2, t0, t2",
            // *t2 = t1
            "sd t1, 0(t2)",

            // set up satp value and load page table
            // t0 >> 12
            "srli t0, t0, 12",
            // t0 & 0xfffffffffff
            "li t2, 0xfffffffffff",
            "and t0, t0, t2",
            // t0 | (8 << 60)
            "li t2, 8 << 60",
            "or t0, t0, t2",

            // write to satp
            "csrw satp, t0",
            // flush tlb
            "sfence.vma",

            "li sp, {stack_top}",

            // call kmain
            "lui t0, %hi({kmain})",
            "addi t0, t0, %lo({kmain})",
            "jr t0",
            kpt = sym KERNEL_PT,
            spt = sym STACK_PT,
            stack_top = const STACK_TOP.as_usize(),
            kmain = sym kmain,
        )
    }
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
    }

    let mut _alloc = BiBuddy::new();
    // TOOD: Find free memory regions from the dtb and add them to the allocator.
    // Also initialize the kernel heap. A block of order 3 should be enough for now (128 KiB).

    shutdown();
}
