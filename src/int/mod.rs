use core::arch::naked_asm;

use crate::asm::{read_csr, write_csr};

#[unsafe(naked)]
unsafe extern "C" fn kernel_entry() -> ! {
    naked_asm!(
        "csrw sscratch, sp",
        "addi sp, sp, -8 * 31",
        "sd ra,  8 * 0(sp)",
        "sd gp,  8 * 1(sp)",
        "sd tp,  8 * 2(sp)",
        "sd t0,  8 * 3(sp)",
        "sd t1,  8 * 4(sp)",
        "sd t2,  8 * 5(sp)",
        "sd t3,  8 * 6(sp)",
        "sd t4,  8 * 7(sp)",
        "sd t5,  8 * 8(sp)",
        "sd t6,  8 * 9(sp)",
        "sd a0,  8 * 10(sp)",
        "sd a1,  8 * 11(sp)",
        "sd a2,  8 * 12(sp)",
        "sd a3,  8 * 13(sp)",
        "sd a4,  8 * 14(sp)",
        "sd a5,  8 * 15(sp)",
        "sd a6,  8 * 16(sp)",
        "sd a7,  8 * 17(sp)",
        "sd s0,  8 * 18(sp)",
        "sd s1,  8 * 19(sp)",
        "sd s2,  8 * 20(sp)",
        "sd s3,  8 * 21(sp)",
        "sd s4,  8 * 22(sp)",
        "sd s5,  8 * 23(sp)",
        "sd s6,  8 * 24(sp)",
        "sd s7,  8 * 25(sp)",
        "sd s8,  8 * 26(sp)",
        "sd s9,  8 * 27(sp)",
        "sd s10, 8 * 28(sp)",
        "sd s11, 8 * 29(sp)",
        "csrr a0, sscratch",
        "sd a0, 8 * 30(sp)",
        "mv a0, sp",
        "call {trap_handler}",
        "ld ra,  8 * 0(sp)",
        "ld gp,  8 * 1(sp)",
        "ld tp,  8 * 2(sp)",
        "ld t0,  8 * 3(sp)",
        "ld t1,  8 * 4(sp)",
        "ld t2,  8 * 5(sp)",
        "ld t3,  8 * 6(sp)",
        "ld t4,  8 * 7(sp)",
        "ld t5,  8 * 8(sp)",
        "ld t6,  8 * 9(sp)",
        "ld a0,  8 * 10(sp)",
        "ld a1,  8 * 11(sp)",
        "ld a2,  8 * 12(sp)",
        "ld a3,  8 * 13(sp)",
        "ld a4,  8 * 14(sp)",
        "ld a5,  8 * 15(sp)",
        "ld a6,  8 * 16(sp)",
        "ld a7,  8 * 17(sp)",
        "ld s0,  8 * 18(sp)",
        "ld s1,  8 * 19(sp)",
        "ld s2,  8 * 20(sp)",
        "ld s3,  8 * 21(sp)",
        "ld s4,  8 * 22(sp)",
        "ld s5,  8 * 23(sp)",
        "ld s6,  8 * 24(sp)",
        "ld s7,  8 * 25(sp)",
        "ld s8,  8 * 26(sp)",
        "ld s9,  8 * 27(sp)",
        "ld s10, 8 * 28(sp)",
        "ld s11, 8 * 29(sp)",
        "ld sp,  8 * 30(sp)",
        "sret",
        trap_handler = sym trap_handler,
    );
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
struct TrapFrame {
    ra: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    sp: usize,
}

#[allow(dead_code)]
unsafe extern "C" fn trap_handler(_trap_frame: &mut TrapFrame) {
    let scause = unsafe { read_csr!("scause") };
    let stval = unsafe { read_csr!("stval") };
    let user_pc = unsafe { read_csr!("sepc") };

    panic!("unexpected trap scause={scause}, stval={stval}, user_pc={user_pc}");
}

pub unsafe fn set_kernel_entry() {
    unsafe {
        write_csr!("stvec", kernel_entry as usize);
    }
}
