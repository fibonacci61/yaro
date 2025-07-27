macro_rules! read_csr {
    ($csr:literal) => {{
        let value: usize;
        ::core::arch::asm!(concat!("csrr {}, ", $csr), out(reg) value);
        value
    }};
}

macro_rules! write_csr {
    ($csr:literal, $value:expr) => {
        ::core::arch::asm!(concat!("csrw ", $csr, ", {}"), in(reg) $value);
    };
}

pub(crate) use read_csr;
pub(crate) use write_csr;
