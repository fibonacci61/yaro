//! RISC V has a maximum of three privilege levels or "modes": M-mode, S-mode, and U-mode.
//! The kernel operates under S-mode while SBI is above it at M-mode and runs concurrently
//! with the kernel. It provides abstractions over the machine's hardware that are accessible
//! through the `ecall` instruction, functionally behaving like a system call from a user
//! program to its operating system.

use core::{arch::asm, mem::MaybeUninit};

pub const SBI_SUCCESS: isize = 0;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(isize)]
pub enum SbiError {
    Failed = -1,
    NotSupported = -2,
    InvalidParam = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
    AlreadyStarted = -7,
    AlreadyStopped = -8,
}

#[must_use]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SbiRet {
    // Invariant: `error` is always SBI_SUCCESS or a variant of SbiError
    error: isize,
    value: MaybeUninit<usize>,
}

impl SbiRet {
    pub fn is_success(&self) -> bool {
        self.error == SBI_SUCCESS
    }

    pub fn error(&self) -> Option<SbiError> {
        if self.error != SBI_SUCCESS {
            Some(unsafe { core::mem::transmute::<isize, SbiError>(self.error) })
        } else {
            None
        }
    }

    pub fn value(&self) -> Option<usize> {
        if self.error == SBI_SUCCESS {
            Some(unsafe { self.value.assume_init() })
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SbiCall {
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    fid: usize,
    eid: usize,
}

#[allow(dead_code)]
impl SbiCall {
    pub const fn new() -> Self {
        Self {
            arg0: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
            fid: 0,
            eid: 0,
        }
    }

    pub unsafe fn call(&self) -> SbiRet {
        let error;
        let value;

        unsafe {
            asm!(
                "ecall",
                in("a0") self.arg0,
                in("a1") self.arg1,
                in("a2") self.arg2,
                in("a3") self.arg3,
                in("a4") self.arg4,
                in("a5") self.arg5,
                in("a6") self.fid,
                in("a7") self.eid,
                lateout("a0") error,
                lateout("a1") value,
            );
        }

        SbiRet { error, value }
    }

    pub fn with_arg0(&mut self, value: usize) -> &mut Self {
        self.arg0 = value;
        self
    }

    pub fn with_arg1(&mut self, value: usize) -> &mut Self {
        self.arg1 = value;
        self
    }

    pub fn with_arg2(&mut self, value: usize) -> &mut Self {
        self.arg2 = value;
        self
    }

    pub fn with_arg3(&mut self, value: usize) -> &mut Self {
        self.arg3 = value;
        self
    }

    pub fn with_arg4(&mut self, value: usize) -> &mut Self {
        self.arg4 = value;
        self
    }

    pub fn with_arg5(&mut self, value: usize) -> &mut Self {
        self.arg5 = value;
        self
    }

    pub fn with_fid(&mut self, value: usize) -> &mut Self {
        self.fid = value;
        self
    }

    pub fn with_eid(&mut self, value: usize) -> &mut Self {
        self.eid = value;
        self
    }
}
