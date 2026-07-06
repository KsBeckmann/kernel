use core::fmt::Write;

use spin::{Mutex, lazylock::LazyLock};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

#[allow(dead_code)]
pub static SERIAL1: LazyLock<Mutex<Uart16550Tty<PioBackend>>> = LazyLock::new(|| {
    let uart =
        unsafe { Uart16550Tty::new_port(0x3F8, Config::default()) }.expect("failed to init COM1");
    Mutex::new(uart)
});

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
