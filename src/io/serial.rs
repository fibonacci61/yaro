use core::fmt::Write;

use spin::Mutex;

use crate::sbi::SbiCall;

pub struct Serial(());

pub static SERIAL: Mutex<Serial> = Mutex::new(Serial(()));

impl Write for Serial {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let ret = unsafe { SbiCall::new().with_arg0(c as usize).with_eid(1).call() };
        if ret.is_success() {
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn print_inner(args: core::fmt::Arguments) {
    let _ = SERIAL.lock().write_fmt(args);
}

macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        $crate::io::serial::print_inner(format_args!($($arg)*));
    };
}

macro_rules! println {
    () => {
        $crate::io::serial::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::io::serial::print!("{}\n", format_args!($($arg)*));
    }
}

pub(crate) use print;
pub(crate) use println;
