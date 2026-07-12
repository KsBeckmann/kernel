#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::{halt, println, vga};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn _start() -> ! {
    vga::clear_screen();
    println!("x = {}, y = {}", 2 + 2, 3 + 3);
    halt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt();
}
